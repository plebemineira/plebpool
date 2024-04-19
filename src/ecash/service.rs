use crate::ecash;

use std::collections::HashSet;
use std::convert::TryInto;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use futures::StreamExt;
use tracing::{debug, error, warn};

#[derive(Clone)]
struct MintState {
    ln: ln_rs::Ln,
    mint: Arc<tokio::sync::Mutex<cdk::mint::Mint>>,
}

async fn handle_paid_invoice(mint: Arc<cdk::mint::Mint>, request: &str) -> anyhow::Result<()> {
    let quotes: Vec<cdk::types::MintQuote> = mint.mint_quotes().await?;

    for quote in quotes {
        if quote.request.eq(request) {
            let q = cdk::types::MintQuote {
                id: quote.id,
                amount: quote.amount,
                unit: quote.unit,
                request: quote.request,
                paid: true,
                expiry: quote.expiry,
            };

            mint.update_mint_quote(q).await?;
        }
    }

    Ok(())
}

pub async fn mint_service(
    config_file_arg: String,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let ecash_config = match ecash::config::EcashConfig::new(&Some(config_file_arg)) {
        Ok(mint_settings) => mint_settings,
        Err(e) => {
            error!("{}. Halting.", e);
            std::process::exit(1);
        }
    };

    // Create db_path parent directory if it doesn't exist
    guarantee_path(&ecash_config.mint.db_path)?;
    let localstore = cdk::mint::RedbLocalStore::new(
        ecash_config
            .mint
            .db_path
            .to_str()
            .expect("PathBuf always converts to &str"),
    )?;

    let mint = cdk::mint::Mint::new(
        Arc::new(localstore),
        bip39::Mnemonic::from_str(&ecash_config.mint.mnemonic)?,
        HashSet::new(),
        cdk::amount::Amount::ZERO,
        0.0,
    )
    .await?;

    debug!("Mint created");

    guarantee_path(&ecash_config.ln.cln_path.clone().unwrap())?; // todo: remove unwrap
    let cln_socket = expand_path(
        ecash_config
            .ln
            .cln_path
            .clone()
            .ok_or(anyhow::anyhow!("cln socket not defined"))?
            .to_str()
            .ok_or(anyhow::anyhow!("cln socket not defined"))?,
    )
    .ok_or(anyhow::anyhow!("cln socket not defined"))?;

    let last_pay_path = PathBuf::from_str(&ecash_config.mint.last_pay_path.clone())?;

    // Create last_pay_path if it doesn't exist
    match fs::metadata(&last_pay_path) {
        Ok(_) => (),
        Err(_e) => {
            // Create the parent directory if it doesn't exist
            fs::create_dir_all(last_pay_path.parent().unwrap())?;

            // Attempt to create the file
            let mut fs = File::create(&last_pay_path).unwrap();
            fs.write_all(&0_u64.to_be_bytes()).unwrap();
        }
    }
    let last_pay = std::fs::read(&last_pay_path).unwrap();

    let last_pay_index =
        u64::from_be_bytes(last_pay.try_into().unwrap_or([0, 0, 0, 0, 0, 0, 0, 0]));

    let cln = ln_rs::Cln::new(cln_socket, Some(last_pay_index)).await?;

    let ln = ln_rs::Ln {
        ln_processor: Arc::new(cln.clone()),
    };

    let ln_clone = ln.clone();
    let mint_clone = Arc::new(mint.clone());

    let cln_handle = tokio::spawn(async move {
        loop {
            let mut stream = ln_clone.ln_processor.wait_invoice().await.unwrap();

            while let Some((invoice, pay_index)) = stream.next().await {
                if let Err(err) =
                    handle_paid_invoice(mint_clone.clone(), &invoice.to_string()).await
                {
                    warn!("{:?}", err);
                }
                if let Some(pay_index) = pay_index {
                    if let Err(err) = std::fs::write(&last_pay_path, pay_index.to_be_bytes()) {
                        warn!("Could not write last pay index {:?}", err);
                    }
                }
            }
        }
    });

    let state = MintState {
        ln,
        mint: Arc::new(tokio::sync::Mutex::new(mint)),
    };

    let mint_service = axum::Router::new()
        .route("/v1/keys", axum::routing::get(get_keys))
        .route("/v1/keysets", axum::routing::get(get_keysets))
        .route(
            "/v1/keys/:keyset_id",
            axum::routing::get(get_keyset_pubkeys),
        )
        .route("/v1/swap", axum::routing::post(post_swap))
        .route(
            "/v1/mint/quote/bolt11",
            axum::routing::post(get_mint_bolt11_quote),
        )
        .route(
            "/v1/mint/quote/bolt11/:quote_id",
            axum::routing::get(get_check_mint_bolt11_quote),
        )
        .route("/v1/mint/bolt11", axum::routing::post(post_mint_bolt11))
        .route(
            "/v1/melt/quote/bolt11",
            axum::routing::post(get_melt_bolt11_quote),
        )
        .route(
            "/v1/melt/quote/bolt11/:quote_id",
            axum::routing::get(get_check_melt_bolt11_quote),
        )
        .route("/v1/melt/bolt11", axum::routing::post(post_melt_bolt11))
        .route("/v1/checkstate", axum::routing::post(post_check))
        .route("/v1/info", axum::routing::get(get_mint_info))
        .route("/v1/restore", axum::routing::post(post_restore))
        .layer(tower_http::cors::CorsLayer::very_permissive().allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        ]))
        .with_state(state);

    let ip = Ipv4Addr::from_str(&ecash_config.mint.listen_host)?;

    let port = ecash_config.mint.listen_port;

    let listen_addr = SocketAddr::new(std::net::IpAddr::V4(ip), port);
    axum::Server::bind(&listen_addr)
        .serve(mint_service.into_make_service())
        .await?;

    // todo: spawn mint handle?

    Ok(cln_handle)
}

async fn get_keys(
    axum::extract::State(state): axum::extract::State<MintState>,
) -> Result<axum::extract::Json<cdk::nuts::KeysResponse>, axum::response::Response> {
    let pubkeys = state
        .mint
        .lock()
        .await
        .pubkeys()
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(pubkeys))
}

async fn get_keysets(
    axum::extract::State(state): axum::extract::State<MintState>,
) -> Result<axum::extract::Json<cdk::nuts::KeysetResponse>, axum::response::Response> {
    let mint = state
        .mint
        .lock()
        .await
        .keysets()
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(mint))
}

async fn get_keyset_pubkeys(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Path(keyset_id): axum::extract::Path<cdk::nuts::Id>,
) -> Result<axum::extract::Json<cdk::nuts::KeysResponse>, axum::response::Response> {
    let pubkeys = state
        .mint
        .lock()
        .await
        .keyset_pubkeys(&keyset_id)
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(pubkeys))
}

async fn post_swap(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::SwapRequest>,
) -> Result<axum::extract::Json<cdk::nuts::SwapResponse>, axum::response::Response> {
    let swap_response = state
        .mint
        .lock()
        .await
        .process_swap_request(payload)
        .await
        .map_err(ecash::error::into_response)?;
    Ok(axum::extract::Json(swap_response))
}

async fn get_mint_bolt11_quote(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::MintQuoteBolt11Request>,
) -> Result<axum::extract::Json<cdk::nuts::MintQuoteBolt11Response>, axum::response::Response> {
    let invoice = state
        .ln
        .ln_processor
        .create_invoice(
            ln_rs::Amount::from_sat(u64::from(payload.amount)),
            "".to_string(),
        )
        .await;

    let invoice = invoice.unwrap();

    let quote = state
        .mint
        .lock()
        .await
        .new_mint_quote(
            invoice.to_string(),
            payload.unit,
            payload.amount,
            unix_time() + 120,
        )
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(quote.into()))
}

async fn get_melt_bolt11_quote(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::MeltQuoteBolt11Request>,
) -> Result<axum::extract::Json<cdk::nuts::MeltQuoteBolt11Response>, axum::response::Response> {
    let amount = payload.request.amount_milli_satoshis().unwrap() / 1000;
    assert!(amount > 0);
    let quote = state
        .mint
        .lock()
        .await
        .new_melt_quote(
            payload.request.to_string(),
            payload.unit,
            cdk::amount::Amount::from(amount),
            cdk::amount::Amount::ZERO,
            unix_time() + 1800,
        )
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(quote.into()))
}

async fn get_check_mint_bolt11_quote(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Path(quote_id): axum::extract::Path<String>,
) -> Result<axum::extract::Json<cdk::nuts::MintQuoteBolt11Response>, axum::response::Response> {
    let quote = state
        .mint
        .lock()
        .await
        .check_mint_quote(&quote_id)
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(quote))
}

async fn post_mint_bolt11(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::MintBolt11Request>,
) -> Result<axum::extract::Json<cdk::nuts::MintBolt11Response>, axum::response::Response> {
    let res = state
        .mint
        .lock()
        .await
        .process_mint_request(payload)
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(res))
}

async fn get_check_melt_bolt11_quote(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Path(quote_id): axum::extract::Path<String>,
) -> Result<axum::extract::Json<cdk::nuts::MeltQuoteBolt11Response>, axum::http::StatusCode> {
    let quote = state
        .mint
        .lock()
        .await
        .check_melt_quote(&quote_id)
        .await
        .unwrap();

    Ok(axum::extract::Json(quote))
}

async fn post_melt_bolt11(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::MeltBolt11Request>,
) -> Result<axum::extract::Json<cdk::nuts::MeltBolt11Response>, axum::http::StatusCode> {
    let quote = state
        .mint
        .lock()
        .await
        .verify_melt_request(&payload)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_ACCEPTABLE)?;

    let pre = state
        .ln
        .ln_processor
        .pay_invoice(ln_rs::Bolt11Invoice::from_str(&quote.request).unwrap(), None)
        .await
        .unwrap();

    let preimage = pre.payment_preimage;
    let res = state
        .mint
        .lock()
        .await
        .process_melt_request(
            &payload,
            &preimage.unwrap(),
            cdk::amount::Amount::from(pre.total_spent.to_sat()),
        )
        .await
        .unwrap();

    Ok(axum::extract::Json(res))
}

async fn post_check(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::CheckStateRequest>,
) -> Result<axum::extract::Json<cdk::nuts::CheckStateResponse>, axum::response::Response> {
    let state = state
        .mint
        .lock()
        .await
        .check_state(&payload)
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(state))
}

async fn get_mint_info(
    axum::extract::State(state): axum::extract::State<MintState>,
) -> Result<axum::extract::Json<cdk::nuts::MintInfo>, axum::response::Response> {
    Ok(axum::extract::Json(
        state
            .mint
            .lock()
            .await
            .mint_info()
            .await
            .map_err(ecash::error::into_response)?,
    ))
}

async fn post_restore(
    axum::extract::State(state): axum::extract::State<MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::RestoreRequest>,
) -> Result<axum::extract::Json<cdk::nuts::RestoreResponse>, axum::response::Response> {
    let restore_response = state
        .mint
        .lock()
        .await
        .restore(payload)
        .await
        .map_err(ecash::error::into_response)?;

    Ok(axum::extract::Json(restore_response))
}

fn expand_path(path: &str) -> Option<PathBuf> {
    if path.starts_with('~') {
        if let Some(home_dir) = dirs::home_dir().as_mut() {
            let remainder = &path[2..];
            home_dir.push(remainder);
            let expanded_path = home_dir;
            Some(expanded_path.clone())
        } else {
            None
        }
    } else {
        Some(PathBuf::from(path))
    }
}

pub fn unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|x| x.as_secs())
        .unwrap_or(0)
}

fn guarantee_path(path: &PathBuf) -> Result<(), std::io::Error> {
    match std::fs::metadata(&path) {
        Ok(_) => Ok(()),
        Err(_e) => {
            std::fs::create_dir_all(path.parent().expect("PathBuf should have valid parent"))?;
            Ok(())
        }
    }
}

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

mod handlers;

#[derive(Clone)]
struct MintState {
    ln: ln_rs::Ln,
    mint: Arc<tokio::sync::Mutex<cdk::mint::Mint>>,
}

pub async fn mint_service(
    config_file_arg: String,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let ecash_config = match ecash::config::EcashConfig::new(&Some(config_file_arg)) {
        Ok(mint_settings) => mint_settings,
        Err(e) => {
            error!("{e:?}. Halting.");
            std::process::exit(1);
        }
    };

    // Create db_path parent directory if it doesn't exist
    guarantee_parent_path(&ecash_config.mint.db_path)?;
    let localstore = cdk::mint::RedbLocalStore::new(
        ecash_config
            .mint
            .db_path
            .to_str()
            .expect("PathBuf always converts to &str"),
    )?;

    let mint = cdk::mint::Mint::new(
        Arc::new(localstore),                                       // localstore
        bip39::Mnemonic::from_str(&ecash_config.mint.mnemonic)?,    // mnemonic
        HashSet::new(),                                             // keysets_info
        cdk::amount::Amount::ZERO,                                  // min_fee_reserve
        0.0,                                                        // percent_fee_reserve
    )
    .await?;

    debug!("Mint created");

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
            if let Some(parent_path) = last_pay_path.parent() {
                fs::create_dir_all(parent_path)?;
            } else {
                return Err(anyhow::anyhow!("Parent directory not found"));
            }


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
        .route("/v1/keys", axum::routing::get(handlers::get_keys))
        .route("/v1/keysets", axum::routing::get(handlers::get_keysets))
        .route(
            "/v1/keys/:keyset_id",
            axum::routing::get(handlers::get_keyset_pubkeys),
        )
        .route("/v1/swap", axum::routing::post(handlers::post_swap))
        .route(
            "/v1/mint/quote/bolt11",
            axum::routing::post(handlers::get_mint_bolt11_quote),
        )
        .route(
            "/v1/mint/quote/bolt11/:quote_id",
            axum::routing::get(handlers::get_check_mint_bolt11_quote),
        )
        .route("/v1/mint/bolt11", axum::routing::post(handlers::post_mint_bolt11))
        .route(
            "/v1/melt/quote/bolt11",
            axum::routing::post(handlers::get_melt_bolt11_quote),
        )
        .route(
            "/v1/melt/quote/bolt11/:quote_id",
            axum::routing::get(handlers::get_check_melt_bolt11_quote),
        )
        .route("/v1/melt/bolt11", axum::routing::post(handlers::post_melt_bolt11))
        .route("/v1/checkstate", axum::routing::post(handlers::post_check))
        .route("/v1/info", axum::routing::get(handlers::get_mint_info))
        .route("/v1/restore", axum::routing::post(handlers::post_restore))
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

// ----
// helper functions
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

fn guarantee_parent_path(path: &PathBuf) -> Result<(), std::io::Error> {
    match std::fs::metadata(&path) {
        Ok(_) => Ok(()),
        Err(_e) => {
            std::fs::create_dir_all(path.parent().expect("PathBuf should have valid parent"))?;
            Ok(())
        }
    }
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

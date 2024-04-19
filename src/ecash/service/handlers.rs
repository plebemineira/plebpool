use std::str::FromStr;

pub async fn get_keys(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
) -> Result<axum::extract::Json<cdk::nuts::KeysResponse>, axum::response::Response> {
    let pubkeys = state
        .mint
        .lock()
        .await
        .pubkeys()
        .await
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(pubkeys))
}

pub async fn get_keysets(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
) -> Result<axum::extract::Json<cdk::nuts::KeysetResponse>, axum::response::Response> {
    let mint = state
        .mint
        .lock()
        .await
        .keysets()
        .await
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(mint))
}

pub async fn get_keyset_pubkeys(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
    axum::extract::Path(keyset_id): axum::extract::Path<cdk::nuts::Id>,
) -> Result<axum::extract::Json<cdk::nuts::KeysResponse>, axum::response::Response> {
    let pubkeys = state
        .mint
        .lock()
        .await
        .keyset_pubkeys(&keyset_id)
        .await
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(pubkeys))
}

pub async fn post_swap(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::SwapRequest>,
) -> Result<axum::extract::Json<cdk::nuts::SwapResponse>, axum::response::Response> {
    let swap_response = state
        .mint
        .lock()
        .await
        .process_swap_request(payload)
        .await
        .map_err(crate::ecash::error::into_response)?;
    Ok(axum::extract::Json(swap_response))
}

pub async fn get_mint_bolt11_quote(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
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
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(quote.into()))
}

pub async fn get_melt_bolt11_quote(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
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
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(quote.into()))
}

pub async fn get_check_mint_bolt11_quote(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
    axum::extract::Path(quote_id): axum::extract::Path<String>,
) -> Result<axum::extract::Json<cdk::nuts::MintQuoteBolt11Response>, axum::response::Response> {
    let quote = state
        .mint
        .lock()
        .await
        .check_mint_quote(&quote_id)
        .await
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(quote))
}

pub async fn post_mint_bolt11(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::MintBolt11Request>,
) -> Result<axum::extract::Json<cdk::nuts::MintBolt11Response>, axum::response::Response> {
    let res = state
        .mint
        .lock()
        .await
        .process_mint_request(payload)
        .await
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(res))
}

pub async fn get_check_melt_bolt11_quote(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
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

pub async fn post_melt_bolt11(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
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

pub async fn post_check(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::CheckStateRequest>,
) -> Result<axum::extract::Json<cdk::nuts::CheckStateResponse>, axum::response::Response> {
    let state = state
        .mint
        .lock()
        .await
        .check_state(&payload)
        .await
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(state))
}

pub async fn get_mint_info(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
) -> Result<axum::extract::Json<cdk::nuts::MintInfo>, axum::response::Response> {
    Ok(axum::extract::Json(
        state
            .mint
            .lock()
            .await
            .mint_info()
            .await
            .map_err(crate::ecash::error::into_response)?,
    ))
}

pub async fn post_restore(
    axum::extract::State(state): axum::extract::State<crate::ecash::service::MintState>,
    axum::extract::Json(payload): axum::extract::Json<cdk::nuts::RestoreRequest>,
) -> Result<axum::extract::Json<cdk::nuts::RestoreResponse>, axum::response::Response> {
    let restore_response = state
        .mint
        .lock()
        .await
        .restore(payload)
        .await
        .map_err(crate::ecash::error::into_response)?;

    Ok(axum::extract::Json(restore_response))
}

pub fn unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|x| x.as_secs())
        .unwrap_or(0)
}
use crate::helper::types::{ AppState, AppError, PII };
use crate::apis::snusbase::SnusbaseHashLookupResponse;

use axum::{
    http::header::HeaderMap,
    extract::{State, Path},
    Json
};
use anyhow::{ Result, anyhow, Context };

pub async fn snusbase_hashing ( 
    State(app): State<AppState>,
    Path(pii_type): Path<PII>,
    headers: HeaderMap,
    pii: String
) -> Result<Json<SnusbaseHashLookupResponse>, AppError> {
    // Verify the API key
    app.verify_api_key_header(&headers)?;

    let cost = crate::COST_PER_HASHES_SNUSBASE;

    // Verify the user has enough balance
    app.verify_user_api_key_has_balance(
        &app,
        &headers, 
        cost
    ).await?;

    // Query Snusbase
    let response = match pii_type {
        PII::Password => {
            app.snusbase
                .lock().await
                .rehash(pii.clone())
                .await
        },
        PII::Hash => {
            app.snusbase
                .lock().await
                .dehash(pii.clone())
                .await
        },
        _ => Err(anyhow!("Invalid PII type!").into())
    }.context("Failed to get Hashing results from Snusbase!")?;

    // Deduct the cost from the user's balance
    app.deduct_cost_and_log(
        &app,
        &headers, 
        ("Hashing".to_string(), "Snusbase".to_string(), pii_type, pii, cost),
    ).await?;

    Ok(Json(response))
}
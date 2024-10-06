use crate::{ AppState, AppError, PII };
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
    // Get the API key in the `Authorization` header
    let api_key = headers.get("Authorization")
        .ok_or_else(|| anyhow!("Missing \"Authorization\" header!"))?
        .to_str()
        .map_err(|e| anyhow!("{e:?}"))?
        .to_owned();
    
    // Check it
    if !app.verify_api_key(api_key).context("Failed to verify API key!")? {
        return Err(anyhow!("Invalid API key!").into());
    }

    // Check if it's a rehash or dehash
    let response = match pii_type {
        PII::Password => {
            app.snusbase
                .lock().await
                .rehash(pii)
                .await
        },
        PII::Hash => {
            app.snusbase
                .lock().await
                .dehash(pii)
                .await
        },
        _ => Err(anyhow!("Invalid PII type!").into())
    }.context("Failed to get Hashing results from Snusbase!")?;

    Ok(Json(response))
}
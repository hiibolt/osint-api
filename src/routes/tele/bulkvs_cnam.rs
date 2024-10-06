use crate::{ AppState, AppError };

use axum::{
    http::header::HeaderMap,
    extract::State,
    Json
};
use anyhow::{ Result, anyhow, Context };

use crate::apis::bulkvs::BulkVSPhoneNumberResponse;

pub async fn bulkvs_cnam ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    pii: String
) -> Result<Json<BulkVSPhoneNumberResponse>, AppError> {
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

    // Get the response from BulkVS
    let response = app.bulkvs
        .lock().await
        .query_phone_number(&pii)
        .context("Failed to get CNAM! from BulkVS!")?;

    Ok(Json(response))
}
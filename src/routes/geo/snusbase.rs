use crate::{ AppState, AppError };
use crate::apis::snusbase::SnusbaseIPResponse;

use axum::{
    http::header::HeaderMap,
    extract::State,
    Json
};
use anyhow::{ Result, anyhow, Context };

pub async fn snusbase_geo ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    ip: String
) -> Result<Json<SnusbaseIPResponse>, AppError> {
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
    let response = app.snusbase
        .lock().await
        .whois_ip_query(vec!(ip)).await
        .context("Failed to get Geolocation results from Snusbase!")?;

    Ok(Json(response))
}
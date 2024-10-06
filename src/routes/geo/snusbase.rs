use crate::helper::types::{ AppState, AppError, PII };
use crate::apis::snusbase::SnusbaseIPResponse;

use axum::{
    http::header::HeaderMap,
    extract::State,
    Json
};
use anyhow::{ Result, Context };

pub async fn snusbase_geo ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    ip: String
) -> Result<Json<SnusbaseIPResponse>, AppError> {
    // Verify the API key
    app.verify_api_key_header(&headers)?;

    let cost = crate::COST_PER_GEO_SNUSBASE;

    // Verify the user has enough balance
    app.verify_user_api_key_has_balance(
        &app,
        &headers, 
        cost
    ).await?;

    // Get the response from BulkVS
    let response = app.snusbase
        .lock().await
        .whois_ip_query(vec!(ip.clone())).await
        .context("Failed to get Geolocation results from Snusbase!")?;

    // Deduct the cost from the user's balance
    app.deduct_cost_and_log(
        &app,
        &headers, 
        ("Geo".to_string(), "Snusbase".to_string(), PII::Ip, ip, cost),
    ).await?;

    Ok(Json(response))
}
use crate::helper::types::{ AppState, AppError, PII };
use crate::apis::sherlock::SherlockResponse;

use axum::{
    http::header::HeaderMap,
    extract::State,
    Json
};
use anyhow::{ Result, Context };

pub async fn sherlock ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    username: String
) -> Result<Json<SherlockResponse>, AppError> {
    // Verify the API key
    app.verify_api_key_header(&headers)?;

    let cost = crate::COST_PER_XREF_SHERLOCK;

    // Verify the user has enough balance
    app.verify_user_api_key_has_balance(
        &app,
        &headers, 
        cost
    ).await?;

    // Get the response from BulkVS
    let response = app.sherlock
        .lock().await
        .get_and_stringify_potential_profiles(username.clone(), true).await
        .context("Failed to get Sherlock! from Sherlock!")?;

    // Deduct the cost from the user's balance
    app.deduct_cost_and_log(
        &app,
        &headers, 
        ("Xref".to_string(), "Sherlock".to_string(), PII::Username, username, cost),
    ).await?;

    Ok(Json(response))
}
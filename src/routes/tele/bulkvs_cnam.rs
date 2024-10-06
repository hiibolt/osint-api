use crate::helper::types::{ AppState, AppError, PII };

use axum::{
    http::header::HeaderMap,
    extract::State,
    Json
};
use anyhow::{ Result, Context };

use crate::apis::bulkvs::BulkVSPhoneNumberResponse;

pub async fn bulkvs_cnam ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    pii: String
) -> Result<Json<BulkVSPhoneNumberResponse>, AppError> {
    // Verify the API key
    app.verify_api_key_header(&headers)?;

    let cost = crate::COST_PER_TELE_BULKVS;

    // Verify the user has enough balance
    app.verify_user_api_key_has_balance(
        &app,
        &headers, 
        cost
    ).await?;

    // Get the response from BulkVS
    let response = app.bulkvs
        .lock().await
        .query_phone_number(&pii)
        .context("Failed to get CNAM! from BulkVS!")?;

    // Deduct the cost from the user's balance
    app.deduct_cost_and_log(
        &app,
        &headers, 
        ("Tele".to_string(), "BulkVS_CNAM".to_string(), PII::Phone, pii, cost),
    ).await?;

    Ok(Json(response))
}
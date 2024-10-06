use crate::{ AppState, AppError };
use crate::apis::sherlock::SherlockResponse;

use axum::{
    http::header::HeaderMap,
    extract::State,
    Json
};
use anyhow::{ Result, anyhow, Context };

pub async fn sherlock ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    username: String
) -> Result<Json<SherlockResponse>, AppError> {
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
    let response = app.sherlock
        .lock().await
        .get_and_stringify_potential_profiles(username, true).await
        .context("Failed to get Sherlock! from Sherlock!")?;

    Ok(Json(response))
}
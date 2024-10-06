use crate::helper::types::{ AppState, AppError };
use crate::apis::database::User;

use std::ops::Deref;

use axum::{
    http::header::HeaderMap,
    extract::State,
    Json
};
use anyhow::{ anyhow, Result, Context };


pub async fn get_user ( 
    State(app): State<AppState>,
    headers: HeaderMap
) -> Result<Json<User>, AppError> {
    // Get the API key in the `Authorization` header
    let api_key = headers.get("Authorization")
        .ok_or_else(|| anyhow!("Missing \'Authorization\' header!"))?
        .to_str()
        .map_err(|e| anyhow!("{e:?}"))?
        .to_owned();
    
    // Check it
    if !app.verify_api_key(api_key).context("Failed to verify API key!")? {
        return Err(anyhow!("Invalid API key!").into());
    }

    // Get the user's API key in the `Authorization` header
    let user_api_key = headers.get("User-API-Key")
        .ok_or_else(|| anyhow!("Missing \'User-API-Key\' header!"))?
        .to_str()
        .map_err(|e| anyhow!("{e:?}"))?
        .to_owned();

    Ok(Json(app.database
        .lock().await
        .get_user(user_api_key)?))
}
pub async fn create_user ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    user: Json<User>
) -> Result<Json<User>, AppError> {
    // Get the API key in the `Authorization` header
    let api_key = headers.get("Authorization")
        .ok_or_else(|| anyhow!("Missing \'Authorization\' header!"))?
        .to_str()
        .map_err(|e| anyhow!("{e:?}"))?
        .to_owned();
    
    // Check it
    if !app.verify_api_key(api_key).context("Failed to verify API key!")? {
        return Err(anyhow!("Invalid API key!").into());
    }
    
    Ok(Json(app.database
        .lock().await
        .create_user(user.deref().clone())?))
}
pub async fn offset_balance ( 
    State(app): State<AppState>,
    headers: HeaderMap,
    amount: String
) -> Result<Json<User>, AppError> {
    // Get the API key in the `Authorization` header
    let api_key = headers.get("Authorization")
        .ok_or_else(|| anyhow!("Missing \'Authorization\' header!"))?
        .to_str()
        .map_err(|e| anyhow!("{e:?}"))?
        .to_owned();
    
    // Check it
    if !app.verify_api_key(api_key).context("Failed to verify API key!")? {
        return Err(anyhow!("Invalid API key!").into());
    }

    // Get the user's API key in the `Authorization` header
    let user_api_key = headers.get("User-API-Key")
        .ok_or_else(|| anyhow!("Missing \'User-API-Key\' header!"))?
        .to_str()
        .map_err(|e| anyhow!("{e:?}"))?
        .to_owned();
    
    // Convert the amount to a number
    let amount = amount.parse::<i32>()
        .context("Failed to parse amount!")?;

    Ok(Json(app.database
        .lock().await
        .offset_balance(user_api_key, amount)?))
}
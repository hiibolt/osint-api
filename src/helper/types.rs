use crate::apis::{
    Snusbase,
    Sherlock,
    BulkVS,
    NocoDB
};
use crate::apis::database::APIUsage;


use std::sync::Arc;
use axum::{
    http::StatusCode,
    http::HeaderMap,
    response::{
        IntoResponse,
        Response
    },
};
use tokio::sync::Mutex;
use anyhow::{ Result, anyhow, Context };
use serde::{ Serialize, Deserialize };

#[derive(Clone)]
pub struct AppState {
    pub sherlock: Arc<Mutex<Sherlock>>,
    pub snusbase: Arc<Mutex<Snusbase>>,
    pub bulkvs:   Arc<Mutex<BulkVS>>,
    pub database: Arc<Mutex<NocoDB>>
}
impl AppState {
    pub fn verify_api_key (
        &self,
        api_key: String
    ) -> Result<bool> {
        // Check if the API key is in the list of valid keys
        Ok(std::env::var("API_KEYS")
            .context("Missing API_KEYS env variable!")?
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .contains(&api_key))
    }
    pub fn verify_api_key_header (
        &self,
        headers: &HeaderMap,
    ) -> Result<()> {
        // Get the API key in the `Authorization` header
        let api_key = headers.get("Authorization")
            .ok_or_else(|| anyhow!("Missing \'Authorization\' header!"))?
            .to_str()
            .map_err(|e| anyhow!("{e:?}"))?
            .to_owned();
        
        // Check it
        if !self.verify_api_key(api_key).context("Failed to verify API key!")? {
            return Err(anyhow!("Invalid API key!").into());
        }

        Ok(())
    }
    pub async fn verify_user_api_key_has_balance (
        &self,
        app:     &AppState,
        headers: &HeaderMap,
        cost:    i32
    ) -> Result<()> {
        // Get the user's API key in the `Authorization` header
        let user_api_key = headers.get("User-API-Key")
            .ok_or_else(|| anyhow!("Missing \'User-API-Key\' header!"))?
            .to_str()
            .map_err(|e| anyhow!("{e:?}"))?
            .to_owned();
    
        // Get and print the user's balance
        let user = app.database
            .lock().await
            .get_user(user_api_key.clone())?;
        
        // Check if the user has enough balance
        if user.balance < cost {
            return Err(anyhow!("Balance {} is insufficient for cost {}!", user.balance, cost).into());
        }

        Ok(())
    }
    pub async fn deduct_cost_and_log(
        &self,
        app:     &AppState,
        headers: &HeaderMap,
        (category, service, pii_type, pii, cost): (String, String, PII, String, i32)
    ) -> Result<()> {
        let user_api_key = headers.get("User-API-Key")
            .ok_or_else(|| anyhow!("Missing \'User-API-Key\' header!"))?
            .to_str()
            .map_err(|e| anyhow!("{e:?}"))?
            .to_owned();

        // Deduct the cost
        app.database
            .lock().await
            .offset_balance(user_api_key.clone(), -1 * cost)?;
        
        // Create a log
        let api_usage_log = APIUsage {
            category,
            service,
            pii_type,
            pii,
            cost,
            id:       None
        };
        if let Err(e) = app.database
            .lock().await
            .create_api_usage_log(api_usage_log, user_api_key) {
            eprintln!("[ WARNING ]: Failed to create API usage log: {:?}", e);
        }

        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum API {
    #[serde(rename = "snusbase_query")]
    SnusbaseQuery,
    #[serde(rename = "snusbase_hashing")]
    SnusbaseHashing,
    #[serde(rename = "snusbase_geolocation")]
    SnusbaseGeolocation,
    #[serde(rename = "bulkvs")]
    BulkVS,
    #[serde(rename = "sherlock")]
    Sherlock
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PII {
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "phone")]
    Phone,
    #[serde(rename = "username")]
    Username,
    #[serde(rename = "hash")]
    Hash,
    #[serde(rename = "ip")]
    Ip,
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "password")]
    Password
}

pub struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            headers,
            format!("{{\"error\": \"{}\"}}", self.0.to_string()),
        ).into_response()
    }
}
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
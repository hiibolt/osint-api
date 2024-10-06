use crate::{ AppState, AppError, PII };
use crate::apis::snusbase::SnusbaseDBResponse;

use axum::{
    http::header::HeaderMap,
    extract::{State, Path},
    Json
};
use anyhow::{ Result, anyhow, Context };

pub async fn snusbase_query ( 
    State(app): State<AppState>,
    Path(pii_type): Path<PII>,
    headers: HeaderMap,
    pii: String
) -> Result<Json<SnusbaseDBResponse>, AppError> {
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

    // Query Snusbase
    let res = match pii_type {
        PII::Email => app.snusbase.lock()
                .await
                .get_by_email(pii)
                .await?,
        PII::Username => app.snusbase.lock()
                .await
                .get_by_username(pii)
                .await?,
        PII::Hash => app.snusbase.lock()
                .await
                .get_by_hash(pii)
                .await?,
        PII::Ip => app.snusbase.lock()
                .await
                .get_by_last_ip(pii)
                .await?,
        PII::Name => app.snusbase.lock()
                .await
                .get_by_name(pii)
                .await?,
        PII::Password => app.snusbase.lock()
                .await
                .get_by_password(pii)
                .await?,
        _ => {
            return Err(anyhow!("Invalid PII type for Snusbase Query API!"))?;
        }
    };

    Ok(Json(res))
}
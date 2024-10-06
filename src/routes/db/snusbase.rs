use crate::helper::types::{ AppState, AppError, PII };
use crate::apis::snusbase::SnusbaseDBResponse;

use axum::{
    http::header::HeaderMap,
    extract::{State, Path},
    Json
};
use anyhow::{ Result, anyhow };

pub async fn snusbase_query ( 
    State(app): State<AppState>,
    Path(pii_type): Path<PII>,
    headers: HeaderMap,
    pii: String
) -> Result<Json<SnusbaseDBResponse>, AppError> {
    // Verify the API key
    app.verify_api_key_header(&headers)?;

    let cost = crate::COST_PER_DB_SNUSBASE;

    // Verify the user has enough balance
    app.verify_user_api_key_has_balance(
        &app,
        &headers, 
        cost
    ).await?;

    // Query Snusbase
    let res = match pii_type {
        PII::Email => app.snusbase.lock()
                .await
                .get_by_email(pii.clone())
                .await?,
        PII::Username => app.snusbase.lock()
                .await
                .get_by_username(pii.clone())
                .await?,
        PII::Hash => app.snusbase.lock()
                .await
                .get_by_hash(pii.clone())
                .await?,
        PII::Ip => app.snusbase.lock()
                .await
                .get_by_last_ip(pii.clone())
                .await?,
        PII::Name => app.snusbase.lock()
                .await
                .get_by_name(pii.clone())
                .await?,
        PII::Password => app.snusbase.lock()
                .await
                .get_by_password(pii.clone())
                .await?,
        _ => {
            return Err(anyhow!("Invalid PII type for Snusbase Query API!"))?;
        }
    };

    // Deduct the cost from the user's balance
    app.deduct_cost_and_log(
        &app,
        &headers, 
        ("DB".to_string(), "Snusbase".to_string(), pii_type, pii, cost),
    ).await?;

    Ok(Json(res))
}
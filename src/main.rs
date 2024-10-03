mod apis;
mod routes;

use crate::apis::{
    Snusbase,
    Sherlock,
    BulkVS,
    NocoDB
};

use std::sync::Arc;
use axum::{
    http::StatusCode,
    response::{
        IntoResponse,
        Response
    }, 
    routing::post, 
    Router
};
use tokio::sync::Mutex;
use anyhow::{ Result, anyhow, Context };
use serde::{ Serialize, Deserialize };

#[derive(Clone)]
struct AppState {
    sherlock: Arc<Mutex<Sherlock>>,
    snusbase: Arc<Mutex<Snusbase>>,
    bulkvs:   Arc<Mutex<BulkVS>>,
    database: Arc<Mutex<NocoDB>>
}
impl AppState {
    pub fn verify_api_key ( &self, api_key: String ) -> Result<bool> {
        // Check if the API key is in the list of valid keys
        Ok(std::env::var("API_KEYS")
            .context("Missing API_KEYS env variable!")?
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .contains(&api_key))
    }
}
#[derive(Debug, Serialize, Deserialize)]
enum API {
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
#[derive(Debug, Serialize, Deserialize)]
enum PII {
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

struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
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

#[tokio::main]
async fn main() -> Result<()> {
    // Build each microservice
    let app_state = AppState {
        sherlock: Arc::new(Mutex::new(Sherlock::new()?)),
        snusbase: Arc::new(Mutex::new(Snusbase::new()?)),
        bulkvs:   Arc::new(Mutex::new(BulkVS::new()?)),
        database: Arc::new(Mutex::new(NocoDB::new()?))
    };

    // Verify the database connection
    app_state.database
        .lock().await.verify_db()
        .context("Failed to verify database connection!")?;

    let app = Router::new()
        .route("/api/tally/:target_api/:pii_type", post(crate::routes::tally_api          ) )
        .route("/api/db/users/get",                post(crate::routes::db::get_user       ) )
        .route("/api/db/users/create",             post(crate::routes::db::create_user    ) )
        .route("/api/db/users/fund",               post(crate::routes::db::offset_balance ) )
        .with_state(app_state);

    let port = std::env::var("PORT")
        .context("Missing PORT env variable!")?;
    let address = format!("0.0.0.0:{port}");

    println!("Listening on {port}, address {address}...");
    let listener = tokio::net::TcpListener::bind(&address).await
        .context("Failed to bind to address!")?;


    axum::serve(listener, app).await
        .map_err(|e| anyhow!("{:?}", e))
        .context("Error in core server, terminating...")?;

    Ok(())
}
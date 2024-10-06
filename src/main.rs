mod apis;
mod routes;
mod helper;

pub const COST_PER_DB_SNUSBASE:     i32 = 30;
pub const COST_PER_GEO_SNUSBASE:    i32 = 15;
pub const COST_PER_XREF_SHERLOCK:   i32 = 10;
pub const COST_PER_TELE_BULKVS:     i32 = 50;
pub const COST_PER_HASHES_SNUSBASE: i32 = 15;


use crate::apis::{
    Snusbase,
    Sherlock,
    BulkVS,
    NocoDB
};
use crate::helper::types::AppState;

use std::sync::Arc;
use axum::{
    routing::post, 
    Router
};
use tokio::sync::Mutex;
use anyhow::{ Result, anyhow, Context };


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
    
    // Build each route set
    let tele_routes = Router::new()
        .route( "/bulkvs_cnam", post(crate::routes::tele::bulkvs_cnam::bulkvs_cnam) );

    let xref_routes = Router::new()
        .route( "/sherlock", post(crate::routes::xref::sherlock::sherlock) );
    
    let geo_routes = Router::new()
        .route( "/snusbase", post(crate::routes::geo::snusbase::snusbase_geo) );
    
    let hashes_routes = Router::new()
        .route( "/snusbase/:pii_type", post(crate::routes::hashes::snusbase::snusbase_hashing) );
    
    let tally_routes = Router::new()
        .route( "/:target_api/:pii_type", post(crate::routes::tally_api) );
    
    let nocodb_routes = Router::new()
        .route("/get",    post(crate::routes::nocodb::get_user       ) )
        .route("/create", post(crate::routes::nocodb::create_user    ) )
        .route("/fund",   post(crate::routes::nocodb::offset_balance ) );
    
    let db_routes = Router::new()
        .route("/snusbase/:pii_type", post(crate::routes::db::snusbase::snusbase_query) );

    // Build the API routes
    let api_v1 = Router::new()
        .nest("/tally", tally_routes)
        .nest("/users", nocodb_routes)
        .nest("/tele", tele_routes)
        .nest("/xref", xref_routes)
        .nest("/geo", geo_routes)
        .nest("/hashes", hashes_routes)
        .nest("/db", db_routes)
        .with_state(app_state);

    let app = Router::new()
        .nest("/api/v1", api_v1);

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
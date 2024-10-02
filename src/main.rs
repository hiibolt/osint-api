mod apis;

use crate::apis::{
    Snusbase,
    Sherlock,
    BulkVS
};

use std::sync::Arc;
use std::collections::HashSet;
use axum::{
    http::{ StatusCode, header::HeaderMap },
    response::{
        IntoResponse,
        Response
    }, 
    extract::{ State, Path },
    routing::post, 
    Router, Json
};
use tokio::sync::Mutex;
use anyhow::{ Result, anyhow, Context };
use serde::{ Serialize, Deserialize };
use serde_json::Value;

#[derive(Clone)]
struct AppState {
    sherlock: Arc<Mutex<Sherlock>>,
    snusbase: Arc<Mutex<Snusbase>>,
    bulkvs:   Arc<Mutex<BulkVS>>
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
#[derive(Debug, Serialize, Deserialize, Default)]
struct Tally {
    usernames: usize,
    emails:    usize,
    phones:    usize,
    hashes:    usize,
    salts:     usize,
    ips:       usize,
    names:     usize,
    passwords: usize,
    addresses: usize,
    companies: usize,
    other:     usize
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

async fn route_api ( 
    State(app): State<AppState>,
    Path((api, pii_type)): Path<(API, PII)>,
    headers: HeaderMap,
    pii: String
) -> Result<Json<Tally>, AppError> {
    // Extract the API key from the headers
    let api_key = headers.get("Authorization")
        .ok_or_else(|| anyhow!("Missing \"Authorization\" header!"))?
        .to_str()
        .map_err(|e| anyhow!("{e:?}"))?
        .to_owned();
    
    println!(
        "Received request:\n\tAPI: {:?}\n\tPII: '{}' ({:?})\n\tAPI Key: '{}'",
        api, pii, pii_type, api_key
    );

    // Ensure the API key is valid
    let api_keys: Vec<String> = std::env::var("API_KEYS")
        .context("Missing API_KEYS env variable!")?
        .split(',')
        .map(|s| s.to_string())
        .collect();

    println!("API keys: {api_keys:?}");

    if !api_keys.contains(&api_key) {
        Err(anyhow!("Invalid API key: '{api_key}'"))?;
    }

    match api {
        API::SnusbaseQuery => {
            let mut tally = Tally::default();

            let mut found_usernames = HashSet::new();
            let mut found_emails    = HashSet::new();
            let mut found_phones    = HashSet::new();
            let mut found_names     = HashSet::new();
            let mut found_addresses = HashSet::new();
            let mut found_companies = HashSet::new();
            let mut found_ips       = HashSet::new();
            let mut found_passwords = HashSet::new();
            let mut found_hashes    = HashSet::new();
            let mut found_salts     = HashSet::new();
            let mut found_other     = HashSet::new();

            let res;

            let pii_value = Value::String(pii.clone());

            // Query Snusbase
            match pii_type {
                PII::Email => {
                    res = app.snusbase.lock()
                        .await
                        .get_by_email(pii)
                        .await?;

                    found_emails.insert(&pii_value);
                },
                PII::Username => {
                    res = app.snusbase.lock()
                        .await
                        .get_by_username(pii)
                        .await?;

                    found_usernames.insert(&pii_value);
                },
                PII::Hash => {
                    res = app.snusbase.lock()
                        .await
                        .get_by_hash(pii)
                        .await?;

                    found_hashes.insert(&pii_value);
                },
                PII::Ip => {
                    res = app.snusbase.lock()
                        .await
                        .get_by_last_ip(pii)
                        .await?;

                    found_ips.insert(&pii_value);
                },
                PII::Name => {
                    res = app.snusbase.lock()
                        .await
                        .get_by_name(pii)
                        .await?;

                    found_names.insert(&pii_value);
                },
                PII::Password => {
                    res = app.snusbase.lock()
                        .await
                        .get_by_password(pii)
                        .await?;

                    found_passwords.insert(&pii_value);
                },
                _ => {
                    return Err(anyhow!("Invalid PII type for Snusbase Query API!"))?;
                }
            }
            
            println!("Res: {res:#?}");

            for (_dump_name, dump_content) in &res.results {
                for entry in dump_content {
                    // If the result is already found, skip it,
                    //  otherwise add it to the tally
                    if let Some(username) = entry.get("username") {
                        if !found_usernames.contains(username) {
                            tally.usernames += 1;
                            found_usernames.insert(username);
                        }
                    }

                    if let Some(email) = entry.get("email") {
                        if !found_emails.contains(email) {
                            tally.emails += 1;
                            found_emails.insert(email);
                        }
                    }

                    if let Some(phone) = entry.get("phone") {
                        if !found_phones.contains(phone) {
                            tally.phones += 1;
                            found_phones.insert(phone);
                        }
                    }

                    if let Some(name) = entry.get("name") {
                        if !found_names.contains(name) {
                            tally.names += 1;
                            found_names.insert(name);
                        }
                    }

                    if let Some(address) = entry.get("address") {
                        if !found_addresses.contains(address) {
                            tally.addresses += 1;
                            found_addresses.insert(address);
                        }
                    }

                    if let Some(company) = entry.get("company") {
                        if !found_companies.contains(company) {
                            tally.companies += 1;
                            found_companies.insert(company);
                        }
                    }

                    // IPs
                    if let Some(last_ip) = entry.get("last_ip") {
                        if !found_ips.contains(last_ip) {
                            tally.ips += 1;
                            found_ips.insert(last_ip);
                        }
                    }
                    if let Some(last_ip) = entry.get("lastip") {
                        if !found_ips.contains(last_ip) {
                            tally.ips += 1;
                            found_ips.insert(last_ip);
                        }
                    }
                    if let Some(last_ip) = entry.get("ip") {
                        if !found_ips.contains(last_ip) {
                            tally.ips += 1;
                            found_ips.insert(last_ip);
                        }
                    }

                    // Passwords, Hashes, and Salts
                    if let Some(password) = entry.get("password") {
                        if !found_passwords.contains(password) {
                            tally.passwords += 1;
                            found_passwords.insert(password);
                        }
                    }
                    if let Some(hash) = entry.get("hash") {
                        if !found_hashes.contains(hash) {
                            tally.hashes += 1;
                            found_hashes.insert(hash);
                        }
                    }
                    if let Some(salt) = entry.get("salt") {
                        if !found_salts.contains(salt) {
                            tally.salts += 1;
                            found_salts.insert(salt);
                        }
                    }

                    for (key, value) in entry {
                        if key == "username" || key == "email" || key == "phone" 
                            || key == "name" || key == "last_ip" || key == "address" || key == "zip" 
                            || key == "company" || key == "lastip" || key == "ip" || key == "password"
                            || key == "hash" || key == "salt" {
                            continue;
                        }

                        if !found_other.contains(value) {
                            tally.other += 1;
                            found_other.insert(value);
                        }
                    }
                }
            }
        
            return Ok(Json(tally))
        },
        API::SnusbaseHashing => {
            let mut tally = Tally::default();

            match pii_type {
                PII::Password => {
                    // Query Snusbase
                    let res = app.snusbase.lock()
                        .await
                        .rehash(pii)
                        .await?;
                    
                    println!("Res: {res:#?}");

                    let mut found_hashes = HashSet::new();
                    let mut found_salts = HashSet::new();

                    for (_dump_name, dump_content) in &res.results {
                        for entry in dump_content {
                            // If the result is already found, skip it,
                            //  otherwise add it to the tally
                            if let Some(hash) = entry.get("hash") {
                                if !found_hashes.contains(hash) {
                                    tally.hashes += 1;
                                    found_hashes.insert(hash);
                                }
                            }

                            if let Some(salt) = entry.get("salt") {
                                if !found_salts.contains(salt) {
                                    tally.salts += 1;
                                    found_salts.insert(salt);
                                }
                            }
                        }
                    }

                    return Ok(Json(tally))
                }
                PII::Hash => {
                    // Query Snusbase
                    let res = app.snusbase.lock()
                        .await
                        .dehash(pii)
                        .await?;
                    
                    println!("Res: {res:#?}");

                    let mut found_passwords = HashSet::new();

                    for (_dump_name, dump_content) in &res.results {
                        for entry in dump_content {
                            // If the result is already found, skip it,
                            //  otherwise add it to the tally
                            if let Some(password) = entry.get("password") {
                                if !found_passwords.contains(password) {
                                    tally.passwords += 1;
                                    found_passwords.insert(password);
                                }
                            }
                        }
                    }

                    return Ok(Json(tally))
                },
                _ => {
                    Err(anyhow!("Invalid PII type for Snusbase Hashing API!"))?
                }
            }
        },
        API::SnusbaseGeolocation => {
            match pii_type {
                PII::Ip => {
                    let mut tally = Tally::default();

                    // Query Snusbase
                    let res = app.snusbase.lock()
                        .await
                        .whois_ip_query(vec![pii])
                        .await?;

                    for (ip, content) in &res.results {
                        println!("IP: {ip}");
                        println!("Content: {content:#?}");

                        if content.get("company").is_some() || content.get("org").is_some() {
                            tally.companies += 1;
                        }

                        if content.get("lat").is_some() && content.get("lon").is_some() {
                            tally.addresses += 1;
                        }
                    }

                    return Ok(Json(tally))
                },
                _ => {
                    Err(anyhow!("Invalid PII type for Snusbase Geolocation API!"))?
                }
            }
        },
        API::BulkVS => {
            match pii_type {
                PII::Phone => {
                    let bulkvs = app.bulkvs.lock().await;

                    let tally = bulkvs.query_phone_number(&pii)?;

                    return Ok(Json(tally))
                },
                _ => {
                    Err(anyhow!("Invalid PII type for BulkVS API!"))?
                }
            }
        },
        API::Sherlock => {
            match pii_type {
                PII::Username => {
                    let sherlock = app.sherlock.lock().await;

                    let usernames = vec![pii];

                    let tally = sherlock.get_and_stringify_potential_profiles(
                        &usernames.into_iter().collect(),
                        false
                    ).await?;

                    return Ok(Json(tally))
                },
                _ => {
                    Err(anyhow!("Invalid PII type for Sherlock API!"))?
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Build each microservice
    let app_state = AppState {
        sherlock: Arc::new(Mutex::new(Sherlock::new()?)),
        snusbase: Arc::new(Mutex::new(Snusbase::new()?)),
        bulkvs:   Arc::new(Mutex::new(BulkVS::new()?))
    };

    let app = Router::new()
        .route("/api/:target_api/:pii_type", post(route_api))
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
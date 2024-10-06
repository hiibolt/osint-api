use crate::helper::types::{ API, AppState, PII, AppError };

use std::collections::HashSet;
use axum::{
    http::header::HeaderMap,
    extract::{ State, Path },
    Json
};
use anyhow::{ Result, anyhow, Context };
use serde::{ Serialize, Deserialize };
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tally {
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

pub async fn tally_api ( 
    State(app): State<AppState>,
    Path((api, pii_type)): Path<(API, PII)>,
    headers: HeaderMap,
    pii: String
) -> Result<Json<Tally>, AppError> {
    // Verify the API key
    app.verify_api_key_header(&headers)?;

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
                    let mut tally = Tally::default();

                    let bulkvs = app.bulkvs.lock().await;

                    if bulkvs.query_phone_number(&pii).context("Failed to query BulkVS!")?.name.is_some() {
                        tally.names += 1;
                    }

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
                    let mut tally = Tally::default();

                    let sherlock = app.sherlock.lock().await;

                    tally.usernames += sherlock
                        .get_and_stringify_potential_profiles(
                            pii,
                            false
                        ).await?
                        .sites
                        .len();

                    return Ok(Json(tally))
                },
                _ => {
                    Err(anyhow!("Invalid PII type for Sherlock API!"))?
                }
            }
        }
    }
}
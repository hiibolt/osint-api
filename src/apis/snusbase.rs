use std::collections::HashMap;

use anyhow::{ Result, Context, bail };
use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct SnusbaseDBResponse {
    pub took: u32,
    pub size: u32,
    pub results: HashMap<String, Vec<HashMap<String, Value>>>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SnusbaseHashLookupResponse {
    pub took: u32,
    pub size: u32,
    pub results: HashMap<String, Vec<Value>>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SnusbaseIPResponse {
    pub took: i32,
    pub size: i32,
    pub results: HashMap<String, HashMap<String, Value>>
}
impl SnusbaseDBResponse {
    pub fn _dumps ( &self ) -> Vec<String> {
        self.results
            .keys()
            .map(|key| key.to_string())
            .collect()
    }
    pub fn _usernames ( &self ) -> Vec<String> {
        let mut usernames = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(username) = entry.get("username") {
                    usernames.push(username.to_string());
                }
            }
        }

        usernames    
    }
    pub fn _emails ( &self ) -> Vec<String> {
        let mut emails = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(email) = entry.get("email") {
                    emails.push(email.to_string());
                }
            }
        }

        emails
    }
    pub fn _passwords ( &self ) -> Vec<String> {
        let mut passwords = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(password) = entry.get("password") {
                    passwords.push(password.to_string());
                }
            }
        }

        passwords
    }
    pub fn _names ( &self ) -> Vec<String> {
        let mut names = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(name) = entry.get("name") {
                    names.push(name.to_string());
                }
            }
        }

        names
    }
    pub fn _last_ips ( &self ) -> Vec<String> {
        let mut last_ips = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(last_ip) = entry.get("last_ip") {
                    last_ips.push(last_ip.to_string());
                }
            }
        }

        last_ips
    }
    pub fn _addresses ( &self ) -> Vec<String> {
        let mut addresses = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(address) = entry.get("address") {
                    addresses.push(address.to_string());

                    continue;
                }
                if let Some(zip) = entry.get("zip") {
                    addresses.push(zip.to_string());

                    continue;
                }
            }
        }

        addresses
    }
    pub fn _companies ( &self ) -> Vec<String> {
        let mut companies = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(company) = entry.get("company") {
                    companies.push(company.to_string());
                }
            }
        }

        companies
    }
    pub fn _other ( &self ) -> Vec<String> {
        let mut other = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                for (key, value) in entry {
                    if key == "username" || key == "email" || key == "password" || key == "name" || key == "last_ip" || key == "address" || key == "zip" || key == "company" {
                        continue;
                    }

                    other.push(format!("{key}: {value}"));
                }
            }
        }

        other
    }
}
#[derive(Debug)]
pub struct Snusbase {
    api_key: String,
}
impl Snusbase {
    pub fn new() -> Result<Self> {
        Ok(Self { 
            api_key: std::env::var("SNUSBASE_API_KEY")
                .context("Missing 'SNUSBASE_API_KEY' environment variable!")? 
        })
    }
    pub async fn whois_ip_query (
        &self,
        ips: Vec<String>
    ) -> Result<SnusbaseIPResponse> {
        if ips.len() == 0 {
            bail!("No IPs to query!");
        }

        // Build a proxied `ureq` client
        let proxy = ureq::Proxy::new(&std::env::var("PROXY_LINK")
            .context("PROXY_LINK not set!")?)?;
        let agent = ureq::AgentBuilder::new()
            .proxy(proxy)
            .build();
        
        // Query Snusbase
        let resp_object = agent.post("https://api-experimental.snusbase.com/tools/ip-whois")
            .set("Auth", &self.api_key )
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({
                "terms": ips
            }))
            .map_err(|e| anyhow::anyhow!("Failed to query IP geolocation backend! {:?}", e))?;

        // Debug print response
        let resp_as_string = resp_object.into_string()
            .context("Failed to convert response to string! It was probably too big.")?;
        
        // Deserialize response with serde_json
        let deserialized_resp: SnusbaseIPResponse = serde_json::from_str(&resp_as_string)
            .context("Failed to deserialize response!")?;
        
        Ok(deserialized_resp)
    }
    pub async fn database_query ( 
        &self,
        terms: Vec<String>,
        types: Vec<String>,
        wildcard: bool
    ) -> Result<SnusbaseDBResponse> {
        // Build a proxied `ureq` client
        let proxy = ureq::Proxy::new(&std::env::var("PROXY_LINK")
            .context("PROXY_LINK not set!")?)?;
        let agent = ureq::AgentBuilder::new()
            .proxy(proxy)
            .build();

        // Query Snusbase
        let resp_object = agent.post("https://api-experimental.snusbase.com/data/search")
            .set("Auth", &self.api_key )
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({
                "terms": terms,
                "types": types,
                "wildcard": wildcard
            }))
            .map_err(|e| anyhow::anyhow!("Failed to query database query backend! {:?}", e))?;

        // Debug print response
        let resp_as_string = resp_object.into_string()
            .context("Failed to convert response to string!")?;
        
        // Deserialize response with serde_json
        let deserialized_resp: SnusbaseDBResponse = serde_json::from_str(&resp_as_string)
            .context("Failed to deserialize response!")?;
        
        Ok(deserialized_resp)
    }
    pub async fn hash_lookup_query ( 
        &self,
        terms: Vec<String>,
        types: Vec<String>,
        wildcard: bool
    ) -> Result<SnusbaseHashLookupResponse> {
        // Build a proxied `ureq` client
        let proxy = ureq::Proxy::new(&std::env::var("PROXY_LINK")
            .context("PROXY_LINK not set!")?)?;
        let agent = ureq::AgentBuilder::new()
            .proxy(proxy)
            .build();

        // Query Snusbase
        let resp_object = agent.post("https://api-experimental.snusbase.com/tools/hash-lookup")
            .set("Auth", &self.api_key )
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({
                "terms": terms,
                "types": types,
                "wildcard": wildcard
            }))
            .map_err(|e| anyhow::anyhow!("Failed to query database query backend! {:?}", e))?;

        // Debug print response
        let resp_as_string = resp_object.into_string()
            .context("Failed to convert response to string!")?;
        
        // Deserialize response with serde_json
        let deserialized_resp: SnusbaseHashLookupResponse = serde_json::from_str(&resp_as_string)
            .context("Failed to deserialize response!")?;
        
        Ok(deserialized_resp)
    }
    pub async fn get_by_email (
        &self,
        email: String
    ) -> Result<SnusbaseDBResponse> {
        self.database_query(
            vec!(email),
            vec!(String::from("email")),
            false
        ).await
    }
    pub async fn get_by_username (
        &self,
        username: String
    ) -> Result<SnusbaseDBResponse> {
        self.database_query(
            vec!(username),
            vec!(String::from("username")),
            false
        ).await
    }
    pub async fn get_by_last_ip (
        &self,
        last_ip: String
    ) -> Result<SnusbaseDBResponse> {
        self.database_query(
            vec!(last_ip),
            vec!(String::from("lastip")),
            false
        ).await
    }
    pub async fn get_by_password (
        &self,
        password: String
    ) -> Result<SnusbaseDBResponse> {
        self.database_query(
            vec!(password),
            vec!(String::from("password")),
            false
        ).await
    }
    pub async fn get_by_name (
        &self,
        name: String
    ) -> Result<SnusbaseDBResponse> {
        self.database_query(
            vec!(name),
            vec!(String::from("name")),
            false
        ).await
    }
    pub async fn get_by_hash (
        &self,
        hash: String
    ) -> Result<SnusbaseDBResponse> {
        self.database_query(
            vec!(hash),
            vec!(String::from("hash")),
            false
        ).await
    }
    pub async fn rehash (
        &self,
        password: String
    ) -> Result<SnusbaseHashLookupResponse> {
        self.hash_lookup_query(
            vec!(password),
            vec!(String::from("password")),
            false
        ).await
    }
    pub async fn dehash (
        &self,
        hash: String
    ) -> Result<SnusbaseHashLookupResponse> {
        self.hash_lookup_query(
            vec!(hash),
            vec!(String::from("hash")),
            false
        ).await
    }
}
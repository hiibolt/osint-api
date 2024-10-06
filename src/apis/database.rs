use crate::helper::types::PII;

use serde::{Deserialize, Serialize};
use anyhow::{ Result, anyhow, Context };
use serde_json::{ json, Value };

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct User {
    pub api_key: String,
    pub balance: i32,
    #[serde(rename = "Id")]
    pub id:      Option<usize>
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct APIUsage {
    pub category: String,
    pub service:  String,
    pub pii_type: PII,
    pub pii:      String,
    pub cost:     i32,
    #[serde(rename = "Id")]
    pub id:      Option<usize>
}

#[derive(Debug)]
pub struct NocoDB {
    api_key:                 String,
    base_url:                String,

    api_keys_table_id:       String,
    api_usage_table_id:      String,
    api_usage_link_field_id: String,
    purchase_table_id:       String,
}
impl NocoDB {
    pub fn new() -> Result<Self> {
        Ok(Self {
            api_key: std::env::var("NOCODB_API_KEY")
                .context("NOCODB_API_KEY must be set")?,
            base_url: std::env::var("NOCODB_URL")
                .context("NOCODB_URL must be set")?,
            api_keys_table_id: std::env::var("API_KEYS_TABLE_ID")
                .context("API_KEYS_TABLE_ID must be set")?,
            api_usage_table_id: std::env::var("API_USAGE_TABLE_ID")
                .context("API_USAGE_TABLE_ID must be set")?,
            api_usage_link_field_id: std::env::var("API_USAGE_LINK_FIELD_ID")
                .context("API_USAGE_LINK_FIELD_ID must be set")?,
            purchase_table_id: std::env::var("PURCHASE_TABLE_ID")
                .context("PURCHASE_TABLE_ID must be set")?
        })
    }
    pub fn verify_db ( &self ) -> Result<()> {
        let url = format!("{}/api/v2/tables/{}/records", self.base_url, self.api_keys_table_id);

        // Use the `ureq` crate to send a POST request to the database
        let _ = ureq::get(&url)
            .set("xc-token", &self.api_key)
            .call()
            .context("Failed to send the request!")?;

        Ok(())
    }

    /* Interfaces for the `api_keys` table */
    pub fn get_users ( &self ) -> Result<Vec<User>> {
        let url = format!("{}/api/v2/tables/{}/records", self.base_url, self.api_keys_table_id);

        // Use the `ureq` crate to send a POST request to the database
        let response = ureq::get(&url)
            .set("xc-token", &self.api_key)
            .call()
            .context("Failed to send the request!")?;

        let response_string = response.into_string()
            .context("Failed to convert response into string!")?;

        let response_value = serde_json::from_str::<Value>(&response_string)
            .context("Response was not valid JSON!")?;
        
        let users_value = response_value.get("list")
            .context("Response was missing `list` field!")?;

        let users: Vec<User> = serde_json::from_value(users_value.clone())
            .context("Failed to deserialize response!")?;

        Ok(users)
    }
    pub fn get_user ( &self, user_api_key: String ) -> Result<User> {
        let users = self.get_users()?;

        for user in &users {
            if user.api_key == user_api_key {
                return Ok((*user).clone());
            }
        }

        Err(anyhow!("User API key '{}' does not exist!", &user_api_key).into())
    }
    pub fn create_user ( &self, user: User ) -> Result<User> {
        // First, verify that the user does not exist
        let users = self.get_users()?;

        for current_user in &users {
            if current_user.api_key == user.api_key {
                return Err(anyhow!("User API key `{}` already exists!", user.api_key).into());
            }
        }

        let mut user = user;

        let url = format!("{}/api/v2/tables/{}/records", self.base_url, self.api_keys_table_id);

        // Use the `ureq` crate to send a POST request to the database
        let response = ureq::post(&url)
            .set("xc-token", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(json!({
                "api_key": user.api_key,
                "balance": user.balance
            }))
            .context("Failed to send the request!")?;

        let response_string = response.into_string()
            .context("Failed to convert response into string!")?;
        
        let response_value = serde_json::from_str::<Value>(&response_string)
            .context("Response was not valid JSON!")?;
        
        let user_id = response_value.get("Id")
            .context("Response was missing `Id` field!")?;
        
        user.id = Some(serde_json::from_value(user_id.clone())
            .context("Failed to deserialize response!")?);
        
        Ok(user)
    }
    pub fn offset_balance ( &self, user_api_key: String, amount: i32 ) -> Result<User> {
        // Verify the user exists
        let mut user = self.get_user(user_api_key)
            .context("User does not exist!")?;
        
        user.balance = (user.balance + amount).max(0);

        let url = format!("{}/api/v2/tables/{}/records", self.base_url, self.api_keys_table_id);

        // Send the PATCH request
        let response = ureq::patch(&url)
            .set("xc-token", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(json!([{
                "Id":      user.id,
                "api_key": user.api_key,
                "balance": user.balance
            }]))
            .context("Failed to send the request!");
        
        let response_string = response?.into_string()
            .context("Failed to convert response into string!")?;

        let response_value = serde_json::from_str::<Value>(&response_string)
            .context("Response was not valid JSON!")?;
        
        // Check that it has the `Id` field within the array of responses
        let _ = response_value
            .as_array()
            .context("Response was not an array!")?
            .get(0)
            .context("Response was missing first element!")?
            .get("Id")
            .context("Response was missing `Id` field!")?;

        Ok(user)
    }
    pub fn create_api_usage_log (
        &self,
        api_usage_log: APIUsage,
        user_api_key: String
    ) -> Result<()> {
        let mut log = api_usage_log.clone();

        // Build the log creation URL
        let create_log_url = format!("{}/api/v2/tables/{}/records", self.base_url, self.api_usage_table_id);

        // Use the `ureq` crate to send a POST request to the database
        let response = ureq::post(&create_log_url)
            .set("xc-token", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(json!({
                "category": log.category,
                "service":  log.service,
                "pii_type": log.pii_type,
                "pii":      log.pii,
                "cost":     log.cost
            }))
            .context("Failed to send the request!")?;

        let response_string = response.into_string()
            .context("Failed to convert response into string!")?;
        
        let response_value = serde_json::from_str::<Value>(&response_string)
            .context("Response was not valid JSON!")?;
        
        // Extract the log's ID
        let log_id = response_value.get("Id")
            .context("Response was missing `Id` field!")?;
        
        log.id = Some(serde_json::from_value(log_id.clone())
            .context("Failed to deserialize response!")?);

        // Get the user's ID
        let user = self.get_user(user_api_key)
            .context("User does not exist!")?;
        
        // Build the table link URL
        let link_log_url = format!(
            "{}/api/v2/tables/{}/links/{}/records/{}",
            self.base_url,
            self.api_keys_table_id,
            self.api_usage_link_field_id,
            user.id.context("User ID was not set!")?
        );

        // Use the `ureq` crate to send a POST request to the database
        let response = ureq::post(&link_log_url)
            .set("xc-token", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(json!([{
                "Id": log.id.context("Log ID was not set!")?,
            }]))
            .context("Failed to send the request!")?;
        
        let response_string = response.into_string()
            .context("Failed to convert response into string!")?;

        if response_string != "true" {
            return Err(anyhow!("Failed to link the log to the user!").into());
        }
        
        Ok(())
    }
}
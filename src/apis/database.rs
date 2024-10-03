use serde::{Deserialize, Serialize};
use anyhow::{ Result, anyhow, Context };
use serde_json::{ json, Value };

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub api_key: String,
    pub balance: Option<i32>,
    #[serde(rename = "Id")]
    pub id:      Option<usize>
}

#[derive(Debug)]
pub struct NocoDB {
    api_key:                 String,
    base_url:                String,

    api_keys_table_id:       String,
    tally_usage_table_id:    String,
    sherlock_usage_table_id: String,
    purchase_table_id:       String,
    bulkvs_table_id:         String,
    snusbase_table_id:       String
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
            tally_usage_table_id: std::env::var("TALLY_USAGE_TABLE_ID")
                .context("TALLY_USAGE_TABLE_ID must be set")?,
            sherlock_usage_table_id: std::env::var("SHERLOCK_USAGE_TABLE_ID")
                .context("SHERLOCK_USAGE_TABLE_ID must be set")?,
            purchase_table_id: std::env::var("PURCHASE_TABLE_ID")
                .context("PURCHASE_TABLE_ID must be set")?,
            bulkvs_table_id: std::env::var("BULKVS_TABLE_ID")
                .context("BULKVS_TABLE_ID must be set")?,
            snusbase_table_id: std::env::var("SNUSBASE_TABLE_ID")
                .context("SNUSBASE_TABLE_ID must be set")?
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
    pub fn get_user ( &self, user: User ) -> Result<User> {
        let users = self.get_users()?;

        for user in &users {
            if user.api_key == user.api_key {
                return Ok((*user).clone());
            }
        }

        Err(anyhow!("User API key `{:?}` does not exist!", user.api_key).into())
    }
    pub fn create_user ( &self, user: User ) -> Result<User> {
        // First, verify that the user does not exist
        let users = self.get_users()?;

        for current_user in &users {
            if current_user.api_key == user.api_key {
                return Err(anyhow!("User API key `{:?}` already exists!", user.api_key).into());
            }
        }

        // If the user doesn't have a balance, set it to 0
        let mut user = user;
        if user.balance.is_none() {
            user.balance = Some(0);
        }

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
    pub fn offset_balance ( &self, user: User ) -> Result<User> {
        // Verify the user exists
        let old_user = self.get_user(user.clone())
            .context("User does not exist!")?;
        
        let new_balance: i32 = old_user.balance.unwrap_or(0) + user.balance.unwrap_or(0);

        let mut user = user;
        user.balance = Some(new_balance);
        if new_balance < 0 {
            user.balance = Some(0);
        }

        let url = format!("{}/api/v2/tables/{}/records", self.base_url, self.api_keys_table_id);

        // Send the PATCH request
        let response = ureq::patch(&url)
            .set("xc-token", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(json!([{
                "Id":      old_user.id,
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
}
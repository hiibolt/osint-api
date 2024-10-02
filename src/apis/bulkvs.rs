use crate::Tally;

use anyhow::{ Result, Context };
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BulkVSPhoneNumberResponse {
    pub name: Option<String>,
    pub _number: Option<String>,
    pub _time: Option<i64>
}
#[derive(Debug)]
pub struct BulkVS {
    api_key: String,
}
impl BulkVS {
    pub fn new () -> Result<Self> {
        Ok(Self {
            api_key: std::env::var("BULKVS_API_KEY")
                .context("Couldn't find API key in environment! Be sure to set `BULKVS_API_KEY`.")?
        })
    }
    pub fn query_phone_number ( &self, phone_number: &str ) -> Result<Tally> {
        let mut tally = Tally::default();

        let path = format!(
            "https://cnam.bulkvs.com/?id={}&did={}&format=json",
            self.api_key,
            phone_number);

        let resp_object = ureq::get(&path)
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to query CNAM lookup backend! {:?}", e))?;

        let resp_object_string = resp_object.into_string()
            .context("Failed to convert response into string!")?;

        let res: BulkVSPhoneNumberResponse = serde_json::from_str(&resp_object_string)
            .context("Failed to deserialize response!")?;

        if res.name.is_some() && res.name != Some("INVALID CALL".into()) {
            tally.names += 1;
        }

        Ok(tally)
    }
}
use anyhow::{ Result, Context };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkVSPhoneNumberResponse {
    pub name:   Option<String>,
    pub number: Option<String>,
    pub time:   Option<i64>
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
    pub fn query_phone_number ( &self, phone_number: &str ) -> Result<BulkVSPhoneNumberResponse> {
        // Build a proxied `ureq` client
        let proxy = ureq::Proxy::new(&std::env::var("PROXY_LINK")
            .context("PROXY_LINK not set!")?)?;
        let agent = ureq::AgentBuilder::new()
            .proxy(proxy)
            .build();

        let path = format!(
            "https://cnam.bulkvs.com/?id={}&did={}&format=json",
            self.api_key,
            phone_number);

        let resp_object = agent.get(&path)
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to query CNAM lookup backend! {:?}", e))?;

        let resp_object_string = resp_object.into_string()
            .context("Failed to convert response into string!")?;

        let res: BulkVSPhoneNumberResponse = serde_json::from_str(&resp_object_string)
            .context("Failed to deserialize response!")?;

        Ok(res)
    }
}
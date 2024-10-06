use tungstenite::connect;
use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SherlockResponse {
    pub username: String,
    pub sites: Vec<String>
}

pub struct Sherlock { }
impl Sherlock {
    pub fn new () -> Result<Self> {
        // Ensure the required environment variables are set
        let _ = std::env::var("SHERLOCK_WS_URL")
            .context("SHERLOCK_WS_URL not set!")?;

        // Verify you can connect to Sherlock
        let _ = connect(&std::env::var("SHERLOCK_WS_URL")
            .context("Can't connect to Sherlock! Is the Sherlock REST API started?")?)
            .context("Can't connect to Sherlock! Is the Sherlock REST API started?")?;

        Ok(Self { })
    }
    pub async fn get_and_stringify_potential_profiles(
        &self,
        username: String, 
        allow_all: bool
    ) -> Result<SherlockResponse> {
        let mut ret = Vec::new();
        
        // If the username is bad, let the user know.
        if !Self::is_valid_sherlock_username(&username, allow_all) {
            return Err(anyhow!("Username would produce very poor results!"));
        }
        
        println!("Querying Sherlock for {username}");

        let sherlock_ws_url = std::env::var("SHERLOCK_WS_URL")
            .expect("SHERLOCK_WS_URL not set!");
        let (
            mut socket,
            response
        ) = connect(&sherlock_ws_url)
            .context("Can't connect to Sherlock! Is the Sherlock REST API started?")?;
        let status = response.status();

        println!("Connected to Sherlock API!");
        println!("Response HTTP code: {status}");

        socket.send(tungstenite::protocol::Message::Text(format!("{username}")))
            .context("Failed to send message to Sherlock API!")?;

        loop {
            let message = socket.read()
                .context("Failed to read message from Sherlock API!")?;

            if let tungstenite::protocol::Message::Text(text) = message {
                if text.contains("http") || text.contains("https") {
                    println!("Found site for {username}: {text}");
                    
                    ret.push(text);
                }
            } else {
                break;
            }
        }

        let ret = SherlockResponse {
            username,
            sites: ret
        };
    
        Ok(ret)
    }
    fn is_valid_sherlock_username ( 
        username: &str,
        allow_all: bool 
    ) -> bool {
        let invalid_characters: [char; 5] = [' ', '.', '-', '_', '#'];
        
        let has_no_invalid_char: bool = !invalid_characters
            .iter()
            .any(|&ch| username.contains(ch));
        let has_alpha_first: bool = username
            .chars()
            .next().unwrap_or(' ')
            .is_alphabetic();
        let within_length: bool = username.chars().count() < 20;
    
        // If the username is bad, let the user know.
        allow_all || ( has_no_invalid_char && has_alpha_first && within_length )
    }
}
use regex::Regex; // Import Regex
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::Response;
// Import Lazy for efficient regex compilation (optional but recommended)

use urlencoding::encode;
// Import the encode function
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;
use reqwest::StatusCode;

use crate::mfl_api::MflError::{ApiStatusError, ClientInitializationFailed, LoginCookieNotFound, RequestFailed};
// Add thiserror crate for convenience

#[derive(Clone, Debug)]
pub struct MflApi {
    client: reqwest::Client,
    pub year: String,
    pub mfl_user_id_cookie: Option<String>,
}

// get_free_agents
#[derive(Serialize, Deserialize, Debug)]
pub struct FreeAgentPlayer {
    pub id: String,
    pub salary: String,
    pub contractStatus: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FreeAgentLeagueUnit {
    pub unit: String,
    pub player: Vec<FreeAgentPlayer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FreeAgents {
    pub leagueUnit: FreeAgentLeagueUnit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FreeAgentResponse {
    pub version: String,
    pub freeAgents: FreeAgents,
    pub encoding: String,
}

// get_players
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayersStatusResponse {
    pub players: PlayersPlayers,
    pub encoding: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayersPlayers {
    pub player: Vec<PlayersPlayer>,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayersPlayer {
    pub position: Option<String>,
    pub id: String,
    pub team: Option<String>,
    pub name: String,
    pub status: Option<String>, // Added status as it is present in some player objects.
}
// end- get_player_roster_status

#[derive(Error, Debug)]
pub enum MflError {
    #[error("Network request failed")]
    Network(#[from] reqwest::Error), // Use #[from] to auto-generate From impl

    #[error("Failed to parse JSON response")]
    JsonParse(#[from] serde_json::Error),

    #[error("Failed to parse using regex")]
    Regex(#[from] regex::Error),

    #[error("Login failed: MFL User ID cookie not found in response")]
    LoginCookieNotFound,

    #[error("Login failed: Check credentials (Specific MFL status: {0})")]
    LoginFailed(String), // You might parse MFL error status here

    #[error("Could not find league host URL in response")]
    LeagueHostNotFound,

    #[error("Header value error")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("MFL API returned an error status: {status} - {body}")]
    ApiStatusError { status: StatusCode, body: String }, // Include status and body

    #[error("Request to MFL failed to complete: {0}")]
    RequestFailed(String),

    #[error("The Request Client initialization failed: {0}")]
    ClientInitializationFailed(String),
    // Add other specific errors as needed
}


// Compile the regex once using once_cell/lazy_static for efficiency
static MFL_USER_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Using expect here because if this fails, the program can't function correctly anyway.
    Regex::new(r#"MFL_USER_ID="([^"]*)">OK"#).expect("Invalid MFL_USER_ID regex")
});

static MFL_API_URL: Lazy<String> = Lazy::new(|| {
    "https://api.myfantasyleague.com".to_string()
});

impl MflApi {
    pub fn new(year: String) -> Result<Self, MflError> {

        let client = reqwest::ClientBuilder::new()
            // Add any required configurations like user agent, timeouts etc.
            // .cookie_store(true) // If using the cookie store feature
            .build().map_err(|e| ClientInitializationFailed(e.to_string()))?; // Adapt error mapping

        Ok(MflApi {
            client,
            year,
            mfl_user_id_cookie: None,
        })
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), MflError> {
        let login_url = format!("https://api.myfantasyleague.com/{}/login?USERNAME={}&PASSWORD={}&XML=1",
                                self.year,
                                encode(username),
                                encode(password)); // Ensure proper encoding

        println!("Making request to get cookie: {}", login_url);
        let login_response = self.client.get(&login_url)
            // Add any necessary headers
            .send()
            .await // Await the send operation
            .map_err(|e| RequestFailed(e.to_string()))?; // Adapt error

        let status = login_response.status(); // *** Get status ***
        let body_text = login_response.text() // *** Read body regardless of status ***
            .await
            .map_err(|e| RequestFailed(format!("Failed to read response body: {}", e)))?;

        if !status.is_success() { // *** Check status ***
            // Consider specific handling for common MFL login errors if possible
            return Err(ApiStatusError { status, body: body_text });
        }

        // Apply the regex to the body text
        // Use the pre-compiled regex
        let cookie_value = MFL_USER_ID_REGEX.captures(&body_text)
            .and_then(|captures| captures.get(1)) // Get the first capture group
            .map(|m| m.as_str().to_string());     // Convert the match to a String


        // Store the cookie or return an error if not found
        match cookie_value {
            Some(cookie) => {
                self.mfl_user_id_cookie = Some(cookie);
                println!("Got cookie {}", self.mfl_user_id_cookie.as_ref().unwrap());
                log::info!("Successfully extracted MFL_USER_ID cookie."); // Optional logging
                Ok(())
            }
            None => {
                log::error!("Could not find MFL_USER_ID cookie in login response body: {}", body_text); // Optional logging
                Err(LoginCookieNotFound)
            }
        }
    }

    // pub async fn get_league_host(&mut self) -> Result<String, MflError> {
    //     let ml_args = "TYPE=myleagues&JSON=1".to_string();
    //     let ml_url = format!(
    //         "{}://{}/{}/export?{}",
    //         self.proto, self.api_host, self.year, ml_args
    //     );
    //     println!("Making request to get league host: {}", ml_url);
    //
    //     let ml_resp = self.send_request(&ml_url).await?;
    //     let ml_body = ml_resp.text().await?;
    //
    //     let league_host_regex = Regex::new(&format!(
    //         r#"url="(https?)://([a-z0-9]+.myfantasyleague.com)/{}/home/([^"]+)"#,
    //         self.year
    //     ))?;
    //     if let Some(captures) = league_host_regex.captures(&ml_body) {
    //         self.proto = captures.get(1).map(|m| m.as_str()).unwrap_or("https").to_string(); // Defaulting 'https' is likely safe
    //
    //         // Use ok_or to handle potential missing capture group cleanly
    //         let league_host = captures.get(2)
    //             .map(|m| m.as_str().to_string())
    //             .ok_or(MflError::LeagueHostNotFound)?; // *** FIXED: Return error if capture group missing ***
    //
    //         println!("Got league host {}", league_host);
    //         Ok(league_host)
    //     } else {
    //         eprintln!("Can't find league host. Response: {}", ml_body);
    //         Err(MflError::LeagueHostNotFound) // *** FIXED: Return error, don't exit ***
    //     }
    // }

    async fn send_request(&self, url: &str) -> Result<Response, MflError> {
        let mut headers = HeaderMap::new();
        if let Some(cookie) = &self.mfl_user_id_cookie {
            headers.insert(COOKIE, HeaderValue::from_str(&format!("MFL_USER_ID={}", cookie))?);
        }
        // Await the send call
        let response = self.client.get(url).headers(headers).send().await?;
        Ok(response)
    }

    pub async fn get_league_info(
        &self,
        league_id: &str
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/{}/export", MFL_API_URL.to_string(), self.year);
        let args = format!("TYPE=league&L={}&JSON=1", league_id);
        let req_url = format!("{}?{}", url, args);
        println!("Making request to get league info {}", req_url);
        let resp = self.send_request(&req_url).await?;
        Ok(resp.text().await?)
    }

    pub async fn get_free_agents(
        &self,
        league_id: &str,
        position: Option<&str>
    ) -> Result<Vec<FreeAgentPlayer>, MflError> {
        let url = format!("{}/{}/export?TYPE=freeAgents", MFL_API_URL.to_string(), self.year);

        let args = match position {
            Some(pos) => format!("&L={}&POSITION={}&JSON=1", league_id, pos),
            None => format!("&L={}&JSON=1", league_id),
        };

        let req_url = format!("{}{}", url, args);
        println!("Making request to get free agents {}", req_url);
        let resp = self.send_request(&req_url).await?; // Get the Response from the Result

        let status = resp.status(); // *** Get status code ***
        let resp_body = resp.text().await.map_err(|e| RequestFailed(format!("Failed to read free agents response body: {}", e)))?; // *** Read body text ***

        if !status.is_success() { // *** Check status code ***
            eprintln!("MFL API error fetching free agents. Status: {}, Body: {}", status, resp_body);
            return Err(ApiStatusError { status, body: resp_body });
        }

        // *** Deserialize only if status was success ***
        let response: FreeAgentResponse = serde_json::from_str(&resp_body)
            .map_err(|e| {
                // Add context if JSON parsing fails even on success status
                eprintln!("Failed to parse successful MFL free agents response. Status: {}, Body: {}, Error: {}", status, resp_body, e);
                MflError::JsonParse(e)
            })?;

        Ok(response.freeAgents.leagueUnit.player)
    }

    pub async fn get_players(
        &self,
        league_id: &str,
        player_ids: &str // can be single player_id or list separated by commas
    ) -> Result<PlayersPlayers, MflError> {
        let url = format!("{}/{}/export?TYPE=players", MFL_API_URL.to_string(), self.year);

        let args = format!("&L={}&PLAYERS={}&JSON=1", league_id, player_ids);

        let req_url = format!("{}{}", url, args);
        println!("Making request to get player info {}", req_url);
        let resp = self.send_request(&req_url).await?; // Get the Response from the Result

        let status = resp.status(); // *** Get status code ***
        let resp_body = resp.text().await.map_err(|e| RequestFailed(format!("Failed to read players response body: {}", e)))?; // *** Read body text ***

        if !status.is_success() { // *** Check status code ***
            eprintln!("MFL API error fetching players. Status: {}, Body: {}", status, resp_body);
            return Err(ApiStatusError { status, body: resp_body });
        }

        // *** Deserialize only if status was success ***
        let response: PlayersStatusResponse = serde_json::from_str(&resp_body)
            .map_err(|e| {
                // Add context if JSON parsing fails even on success status
                eprintln!("Failed to parse successful MFL players response. Status: {}, Body: {}, Error: {}", status, resp_body, e);
                MflError::JsonParse(e)
            })?;

        Ok(response.players)
    }
}
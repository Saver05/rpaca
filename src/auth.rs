//! Authentication module for Alpaca API.
//!
//! This module provides functionality for authenticating with the Alpaca API,
//! including creating clients for both paper trading and live trading environments.
//! It handles API key management and provides methods for making authenticated requests.

use crate::request::create_trading_request;
use reqwest::{Client as HttpClient, Method};
use std::cmp::PartialEq;
use std::env;

/// Client for interacting with the Alpaca API.
///
/// This struct holds authentication credentials and connection details
/// required for making requests to the Alpaca trading API.
pub struct Alpaca {
    /// The Alpaca API key ID used for authentication.
    apca_api_key_id: String,
    /// The Alpaca API secret key used for authentication.
    apca_api_secret_key: String,
    /// The base URL for the Alpaca API, depends on trading type (paper/live).
    trading_url: String,
    /// HTTP client used for making requests to the Alpaca API.
    http_client: HttpClient,
}

#[derive(Default)]
pub enum TradingType {
    #[default]
    Paper,
    Live,
}

impl PartialEq for TradingType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (TradingType::Paper, TradingType::Paper) | (TradingType::Live, TradingType::Live)
        )
    }
}

impl Alpaca {
    pub fn new(apca_api_key: String, apca_api_secret: String, trading_type: TradingType) -> Alpaca {
        let trading_url: String;
        if trading_type == TradingType::Live {
            trading_url = "https://api.alpaca.markets".to_string();
        } else {
            trading_url = "https://paper-api.alpaca.markets".to_string();
        }
        Alpaca {
            apca_api_key_id: apca_api_key,
            apca_api_secret_key: apca_api_secret,
            trading_url,
            http_client: HttpClient::new(),
        }
    }

    pub fn from_env(trading_type: TradingType) -> Result<Alpaca, env::VarError> {
        dotenv::dotenv().ok(); // Loads .env into std::env

        let api_key = env::var("APCA_API_KEY_ID")?;
        let api_secret = env::var("APCA_API_SECRET_KEY")?;

        let trading_url = match trading_type {
            TradingType::Live => "https://api.alpaca.markets".to_string(),
            TradingType::Paper => "https://paper-api.alpaca.markets".to_string(),
        };

        Ok(Alpaca {
            apca_api_key_id: api_key,
            apca_api_secret_key: api_secret,
            trading_url,
            http_client: HttpClient::new(),
        })
    }

    pub fn get_apca_api_key_id(&self) -> String {
        self.apca_api_key_id.clone()
    }
    pub fn get_apca_api_secret(&self) -> String {
        self.apca_api_secret_key.clone()
    }
    pub fn get_trading_url(&self) -> String {
        self.trading_url.clone()
    }
    pub fn get_http_client(&self) -> HttpClient {
        self.http_client.clone()
    }
}

#[tokio::test]
async fn test_auth() {
    let alpaca = Alpaca::new("test".to_string(), "test".to_string(), TradingType::Paper);
    assert_eq!(alpaca.get_apca_api_key_id(), "test");
    assert_eq!(alpaca.get_apca_api_secret(), "test");
    assert_eq!(alpaca.get_trading_url(), "https://paper-api.alpaca.markets");
    match create_trading_request::<()>(&alpaca, Method::GET, "/v2/account", None).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => assert_eq!(text, "{\"message\": \"forbidden.\"}\n"),
            Err(e) => {
                eprintln!("Failed to read response: {}", e);
                assert!(false);
            }
        },
        Err(e) => {
            eprintln!("Request failed: {}", e);
            assert!(false);
        }
    }
}

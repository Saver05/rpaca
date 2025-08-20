//! Request handling module for Alpaca API.
//!
//! This module provides functionality for creating and sending HTTP requests to the Alpaca API,
//! including both trading and market data endpoints. It handles authentication headers and
//! request formatting.

use crate::auth;
use crate::auth::TradingType;
use auth::Alpaca;
use reqwest::{Method, Response};
use serde::Serialize;

/// Creates and sends an HTTP request to the Alpaca trading API.
///
/// # Parameters
/// * `alpaca` - The Alpaca authentication instance containing API keys and configuration
/// * `method` - The HTTP method to use for the request (GET, POST, etc.)
/// * `endpoint` - The API endpoint to call (e.g., "/v2/account")
/// * `body` - Optional JSON body to include with the request
///
/// # Returns
/// A Result containing either the HTTP Response or a reqwest Error
pub async fn create_trading_request<T: Serialize>(
    alpaca: &Alpaca,
    method: Method,
    endpoint: &str,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    let url = format!("{}{}", alpaca.get_trading_url(), endpoint);
    let client = alpaca.get_http_client();

    let mut request_builder = client
        .request(method, &url)
        .header("APCA-API-KEY-ID", alpaca.get_apca_api_key_id())
        .header("APCA-API-SECRET-KEY", alpaca.get_apca_api_secret());

    if let Some(json_body) = body {
        request_builder = request_builder.json(&json_body);
    }

    request_builder.send().await
}

/// Creates and sends an HTTP request to the Alpaca market data API.
///
/// # Parameters
/// * `alpaca` - The Alpaca authentication instance containing API keys and configuration
/// * `method` - The HTTP method to use for the request (GET, POST, etc.)
/// * `endpoint` - The API endpoint to call (e.g., "/v2/stocks/snapshots")
/// * `body` - Optional JSON body to include with the request
///
/// # Returns
/// A Result containing either the HTTP Response or a reqwest Error
pub async fn create_data_request<T: Serialize>(
    alpaca: &Alpaca,
    method: Method,
    endpoint: &str,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    let url = format!("{}{}", "https://data.alpaca.markets", endpoint);
    let client = alpaca.get_http_client();

    let mut request_builder = client
        .request(method, &url)
        .header("APCA-API-KEY-ID", alpaca.get_apca_api_key_id())
        .header("APCA-API-SECRET-KEY", alpaca.get_apca_api_secret());

    if let Some(json_body) = body {
        request_builder = request_builder.json(&json_body);
    }

    request_builder.send().await
}

#[tokio::test]
async fn test_auth_connection() {
    let alpaca = Alpaca::from_env(TradingType::Paper).expect("Failed to read env");
    match create_trading_request::<()>(&alpaca, Method::GET, "/v2/account", None).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => assert_ne!(text, "{\"message\":\"forbidden.\"}\n"),
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

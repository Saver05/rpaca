//! Clock module for Alpaca API v2.
//!
//! This module provides functionality for retrieving the current market clock status
//! from Alpaca's trading API. It allows checking if the market is currently open
//! and when the next market open and close times will occur.
//!
//! The clock information is essential for scheduling trading activities and ensuring
//! that orders are placed during market hours.

use crate::auth::{Alpaca, TradingType};
use crate::request::create_trading_request;
use reqwest::Method;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Clock {
    pub timestamp: String,
    pub is_open: bool,
    pub next_open: String,
    pub next_close: String,
}

/// Retrieves the current market clock status.
///
/// This function fetches the current market clock information from Alpaca's API,
/// including whether the market is currently open and the next market open/close times.
/// This information is useful for scheduling trading activities and ensuring orders
/// are placed during market hours.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
///
/// # Returns
/// * `Result<Clock, Box<dyn std::error::Error>>` - The current market clock information or an error
pub async fn get_clock(alpaca: &Alpaca) -> Result<Clock, Box<dyn std::error::Error>> {
    let response = create_trading_request::<()>(alpaca, Method::GET, "/v2/clock", None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting clock failed: {}", text).into());
    };
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_clock() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_clock(&alpaca).await {
        Ok(clock) => {
            if clock.is_open {
                assert!(clock.is_open)
            } else {
                assert!(!clock.is_open)
            }
        }
        Err(e) => println!("Error: {e:?}"),
    }
}

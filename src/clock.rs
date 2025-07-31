use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use reqwest::Method;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Clock {
    timestamp: String,
    is_open: bool,
    next_open: String,
    next_close: String,
}

pub async fn get_clock(alpaca: &Alpaca) -> Result<Clock, Box<dyn std::error::Error>> {
    let response = create_request::<()>(alpaca, Method::GET, "/v2/clock", None).await?;
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

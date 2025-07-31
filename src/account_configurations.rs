use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AccountConfigurations {
    dtbp_check: String,
    trade_confirm_email: Option<String>,
    suspend_trade: bool,
    no_shorting: bool,
    fractional_trading: bool,
    max_margin_multiplier: String,
    max_options_trading_level: Option<i8>,
    pdt_check: String,
    ptp_no_exception_entry: bool,
}

pub async fn get_account_configurations(
    alpaca: &Alpaca,
) -> Result<AccountConfigurations, Box<dyn std::error::Error>> {
    let response =
        create_request::<()>(alpaca, Method::GET, "/v2/account/configurations", None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Deleting symbol from watchlist failed: {text}").into());
    };
    Ok(response.json().await?)
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateAccountConfigurations {
    #[serde(skip_serializing_if = "Option::is_none")]
    dtbp_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trade_confirm_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suspend_trade: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    no_shorting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fractional_trading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_margin_multiplier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_options_trading_level: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdt_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ptp_no_exception_entry: Option<bool>,
}
pub async fn update_account_configurations(
    alpaca: &Alpaca,
    configs: UpdateAccountConfigurations,
) -> Result<AccountConfigurations, Box<dyn std::error::Error>> {
    let response = create_request(
        alpaca,
        Method::PATCH,
        "/v2/account/configurations",
        Some(configs),
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Deleting symbol from watchlist failed: {text}").into());
    };
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_account_configurations() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    let configs = match get_account_configurations(&alpaca).await {
        Ok(configs) => {
            assert_eq!(configs.suspend_trade, false);
            assert_eq!(configs.no_shorting, false);
            assert_eq!(configs.fractional_trading, true);
            configs
        }
        Err(e) => panic!("{:?}", e),
    };
    let new_configs = match update_account_configurations(
        &alpaca,
        UpdateAccountConfigurations {
            no_shorting: Some(true),
            ..Default::default()
        },
    )
    .await
    {
        Ok(configs) => configs,
        Err(e) => panic!("Failed to update configs: {:?}", e),
    };
    assert!(new_configs.no_shorting);
    assert_eq!(new_configs.suspend_trade, configs.suspend_trade);
    assert_eq!(new_configs.fractional_trading, configs.fractional_trading);
    match update_account_configurations(
        &alpaca,
        UpdateAccountConfigurations {
            no_shorting: Some(false),
            ..Default::default()
        },
    )
    .await
    {
        Ok(configs) => {
            assert_eq!(configs.no_shorting, false);
        }
        Err(e) => panic!("Failed to update configs: {e:?}"),
    };
}

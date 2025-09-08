//! Account configurations module for Alpaca API v2.
//!
//! This module provides functionality for retrieving and updating account configuration settings
//! through Alpaca's trading API. These settings control various aspects of trading behavior
//! and account functionality.
//!
//! The module includes functionality for:
//! - Retrieving current account configuration settings
//! - Updating account configuration settings
//! - Managing settings like day trading buying power checks, margin multipliers, and trading restrictions

use crate::auth::{Alpaca, TradingType};
use crate::request::create_trading_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Deserialize)]
pub struct AccountConfigurations {
    pub dtbp_check: String,
    pub trade_confirm_email: Option<String>,
    pub suspend_trade: bool,
    pub no_shorting: bool,
    pub fractional_trading: bool,
    pub max_margin_multiplier: String,
    pub max_options_trading_level: Option<i8>,
    pub pdt_check: String,
    pub ptp_no_exception_entry: bool,
}

/// Retrieves the current account configuration settings.
///
/// This function fetches the current configuration settings for the Alpaca trading account,
/// including settings for day trading buying power checks, trade confirmations, trading
/// restrictions, and margin multipliers.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
///
/// # Returns
/// * `Result<AccountConfigurations, Box<dyn std::error::Error>>` - The account configuration settings or an error
pub async fn get_account_configurations(
    alpaca: &Alpaca,
) -> Result<AccountConfigurations, Box<dyn std::error::Error>> {
    let response =
        create_trading_request::<()>(alpaca, Method::GET, "/v2/account/configurations", None)
            .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Deleting symbol from watchlist failed: {text}").into());
    };
    Ok(response.json().await?)
}

#[derive(Debug, Serialize, Default, TypedBuilder)]
pub struct UpdateAccountConfigurations {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub dtbp_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub trade_confirm_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub suspend_trade: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub no_shorting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub fractional_trading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub max_margin_multiplier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub max_options_trading_level: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub pdt_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub ptp_no_exception_entry: Option<bool>,
}
/// Updates the account configuration settings.
///
/// This function allows modifying various account configuration settings such as
/// day trading buying power checks, trade confirmations, trading restrictions,
/// and margin multipliers. Only the fields specified in the `configs` parameter
/// will be updated; other settings will remain unchanged.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `configs` - The configuration settings to update
///
/// # Returns
/// * `Result<AccountConfigurations, Box<dyn std::error::Error>>` - The updated account configuration settings or an error
pub async fn update_account_configurations(
    alpaca: &Alpaca,
    configs: UpdateAccountConfigurations,
) -> Result<AccountConfigurations, Box<dyn std::error::Error>> {
    let response = create_trading_request(
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
        UpdateAccountConfigurations::builder()
            .no_shorting(true)
            .build(),
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
        UpdateAccountConfigurations::builder()
            .no_shorting(false)
            .build(),
    )
    .await
    {
        Ok(configs) => {
            assert_eq!(configs.no_shorting, false);
        }
        Err(e) => panic!("Failed to update configs: {e:?}"),
    };
}

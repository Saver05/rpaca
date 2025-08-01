use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use reqwest::Method;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    pub account_blocked: bool,
    pub account_number: String,
    pub accrued_fees: String,
    pub admin_configurations: serde_json::Value, // can be a custom struct if needed
    pub balance_asof: String,
    pub bod_dtbp: String,
    pub buying_power: String,
    pub cash: String,
    pub created_at: String, // Or chrono::DateTime<Utc> if using chrono
    pub crypto_status: String,
    pub crypto_tier: u8,
    pub currency: String,
    pub daytrade_count: u32,
    pub daytrading_buying_power: String,
    pub effective_buying_power: String,
    pub equity: String,
    pub id: String,
    pub initial_margin: String,
    pub intraday_adjustments: String,
    pub last_equity: String,
    pub last_maintenance_margin: String,
    pub long_market_value: String,
    pub maintenance_margin: String,
    pub multiplier: String,
    pub non_marginable_buying_power: String,
    pub options_approved_level: u8,
    pub options_buying_power: String,
    pub options_trading_level: u8,
    pub pattern_day_trader: bool,
    pub pending_reg_taf_fees: String,
    pub portfolio_value: String,
    pub position_market_value: String,
    pub regt_buying_power: String,
    pub short_market_value: String,
    pub shorting_enabled: bool,
    pub sma: String,
    pub status: String,
    pub trade_suspended_by_user: bool,
    pub trading_blocked: bool,
    pub transfers_blocked: bool,
    pub user_configurations: Option<serde_json::Value>, // null in JSON
}

pub async fn get_account_info(alpaca: &Alpaca) -> Result<AccountInfo, Box<dyn std::error::Error>> {
    let response = create_request::<()>(&alpaca, Method::GET, "/v2/account", None).await?;
    let info: AccountInfo = response.json().await?;
    Ok(info)
}

#[tokio::test]
async fn test_get_account_info() {
    let alpaca = Alpaca::from_env(TradingType::Paper).expect("Failed to read env");
    match get_account_info(&alpaca).await {
        Ok(r) => {
            assert_eq!(r.trade_suspended_by_user, false);
            assert_eq!(r.transfers_blocked, false);
        }
        Err(e) => {
            eprintln!("Failed to get account info: {}", e);
            assert!(false);
        }
    }
}

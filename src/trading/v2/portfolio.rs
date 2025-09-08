use crate::auth::{Alpaca, TradingType};
use crate::request::create_trading_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Default, Serialize, TypedBuilder, Clone)]
pub struct PortfolioParams {
    #[builder(default, setter(strip_option))]
    pub period: Option<String>,
    #[builder(default, setter(strip_option))]
    pub timeframe: Option<String>,
    #[builder(default, setter(strip_option))]
    pub intraday_reporting: Option<String>,
    #[builder(default, setter(strip_option))]
    pub start: Option<String>,
    #[builder(default, setter(strip_option))]
    pub pnl_reset: Option<String>,
    #[builder(default, setter(strip_option))]
    pub end: Option<String>,
    #[builder(default, setter(strip_option))]
    pub extended_hours: Option<String>,
    #[builder(default, setter(strip_option))]
    pub cashflow_types: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct PortfolioHistory {
    pub timestamp: Vec<i128>,
    pub equity: Vec<f64>,
    pub profit_loss: Vec<f64>,
    pub profit_loss_pct: Vec<f64>,
    pub base_value: f64,
    pub base_value_asof: Option<String>,
    pub timeframe: String,
    pub cashflow: Option<serde_json::Value>,
}
pub async fn get_portfolio_history(
    alpaca: &Alpaca,
    params: PortfolioParams,
) -> Result<PortfolioHistory, Box<dyn std::error::Error>> {
    let mut query_pairs = vec![];

    if let Some(v) = params.period {
        query_pairs.push(("period", v))
    };
    if let Some(v) = params.timeframe {
        query_pairs.push(("timeframe", v))
    };
    if let Some(v) = params.intraday_reporting {
        query_pairs.push(("intraday_reporting", v))
    };
    if let Some(v) = params.start {
        query_pairs.push(("start", v))
    };
    if let Some(v) = params.pnl_reset {
        query_pairs.push(("pnl_reset", v))
    };
    if let Some(v) = params.end {
        query_pairs.push(("end", v))
    };
    if let Some(v) = params.extended_hours {
        query_pairs.push(("extended_hours", v))
    };
    if let Some(v) = params.cashflow_types {
        query_pairs.push(("cashflow_types", v))
    };

    let query_string = serde_urlencoded::to_string(&query_pairs)?;
    let endpoint = if query_string.is_empty() {
        "/v2/account/portfolio/history".to_string()
    } else {
        format!("/v2/account/portfolio/history?{query_string}")
    };

    let response = create_trading_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_portfolio_history() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();

    let history = get_portfolio_history(&alpaca, PortfolioParams::builder().build())
        .await
        .unwrap();
    assert_eq!(history.timeframe, "1D")
}

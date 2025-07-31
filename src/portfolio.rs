use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Serialize)]
pub struct PortfolioParams {
    period: Option<String>,
    timeframe: Option<String>,
    intraday_reporting: Option<String>,
    start: Option<String>,
    pnl_reset: Option<String>,
    end: Option<String>,
    extended_hours: Option<String>,
    cashflow_types: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct PortfolioHistory {
    timestamp: Vec<i128>,
    equity: Vec<f64>,
    profit_loss: Vec<f64>,
    profit_loss_pct: Vec<f64>,
    base_value: f64,
    base_value_asof: Option<String>,
    timeframe: String,
    cashflow: Option<serde_json::Value>,
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

    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_portfolio_history() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();

    let history = get_portfolio_history(&alpaca, PortfolioParams::default())
        .await
        .unwrap();
    assert_eq!(history.timeframe, "1D")
}

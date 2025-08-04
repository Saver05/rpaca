use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use crate::trading::v2::orders::{Order, OrderRequest, create_order};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub asset_id: String,
    pub symbol: String,
    pub exchange: String,
    pub asset_class: String,
    pub asset_marginable: bool,
    pub qty: String,
    pub avg_entry_price: String,
    pub side: String,
    pub market_value: String,
    pub cost_basis: String,
    pub unrealized_pl: String,
    pub unrealized_plpc: String,
    pub unrealized_intraday_pl: String,
    pub unrealized_intraday_plpc: String,
    pub current_price: String,
    pub lastday_price: String,
    pub change_today: String,
    pub qty_available: String,
}
pub async fn get_positions(alpaca: &Alpaca) -> Result<Vec<Position>, Box<dyn std::error::Error>> {
    let endpoint = "/v2/positions".to_string();
    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting positions failed: {}", text).into());
    }
    let positions: Vec<Position> = response.json().await?;
    Ok(positions)
}

pub async fn get_single_position(
    alpaca: &Alpaca,
    symbol: String,
) -> Result<Position, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/positions/{symbol}");
    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting single position failed: {}", text).into());
    }
    let p: Position = response.json().await?;
    Ok(p)
}

pub struct ClosePositionParams {
    pub symbol: String,
    pub qty: Option<f64>,
    pub percentage: Option<f64>,
}
pub async fn close_position(
    alpaca: &Alpaca,
    params: ClosePositionParams,
) -> Result<Order, Box<dyn std::error::Error>> {
    let mut endpoint = format!("/v2/positions/{}", params.symbol);
    if params.qty.is_some() {
        let qty = params.qty.unwrap();
        endpoint = format!("{}?qty={}", endpoint, qty);
    } else if params.percentage.is_some() {
        let percentage = params.percentage.unwrap();
        endpoint = format!("{}?percentage={}", endpoint, percentage);
    }
    let response = create_request::<()>(alpaca, Method::DELETE, &endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Closing position failed: {}", text).into());
    }
    let order: Order = response.json().await?;
    Ok(order)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ClosedPositions {
    symbol: String,
    status: i128,
    body: Order,
}
pub async fn close_all_positions(
    alpaca: &Alpaca,
    cancel_orders: bool,
) -> Result<Vec<ClosedPositions>, Box<dyn std::error::Error>> {
    let response = create_request::<()>(alpaca, Method::DELETE, "/v2/positions", None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Closing all positions failed: {}", text).into());
    }
    Ok(response.json().await?)
}

pub async fn exercise_options_position(
    alpaca: &Alpaca,
    symbol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/positions/{}/exercise", symbol);
    let response = create_request::<()>(alpaca, Method::POST, &endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Exercise options position failed: {}", text).into());
    }
    Ok(())
}

#[tokio::test]
async fn test_position() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    let order = match create_order(
        &alpaca,
        OrderRequest::builder()
            .symbol("GOOG")
            .qty("2")
            .side("buy")
            .order_type("market")
            .time_in_force("day")
            .build(),
    )
    .await
    {
        Ok(order) => order,
        Err(e) => panic!("Failed to create order for positions{e}"),
    };

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let position = match get_positions(&alpaca).await {
        Ok(positions) => match positions.into_iter().find(|p| p.symbol == "GOOG") {
            Some(p) => p,
            None => panic!("GOOG position not found"),
        },
        Err(e) => panic!("Failed to get positions: {}", e),
    };

    assert_eq!(position.qty, "2");

    let single_position = match get_single_position(&alpaca, "GOOG".to_string()).await {
        Ok(p) => p,
        Err(e) => panic!("Failed to get single position: {e}"),
    };

    assert_eq!(position.qty, single_position.qty);
    assert_eq!(position.side, single_position.side);
    assert_eq!(position.symbol, single_position.symbol);
    assert_eq!(position.exchange, single_position.exchange);

    let close_position = match close_position(
        &alpaca,
        ClosePositionParams {
            symbol: "GOOG".to_string(),
            qty: Some(1.0),
            percentage: None,
        },
    )
    .await
    {
        Ok(order) => order,
        Err(e) => panic!("Failed to close position: {}", e),
    };

    assert_eq!(close_position.symbol, "GOOG");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let close_all_positions = match close_all_positions(&alpaca, true).await {
        Ok(positions) => positions,
        Err(e) => panic!("Failed to close all positions: {e}"),
    };
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    match get_positions(&alpaca).await {
        Ok(positions) => {
            assert_eq!(positions.len(), 0);
        }
        Err(e) => panic!("Failed to get positions: {e}"),
    };
}

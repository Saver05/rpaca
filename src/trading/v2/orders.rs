use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize, Serializer};
use typed_builder::TypedBuilder;
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Order {
    pub id: String,
    pub client_order_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub submitted_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub replaced_at: Option<DateTime<Utc>>,
    pub replaced_by: Option<String>,
    pub replaces: Option<String>,
    pub asset_id: String,
    pub symbol: String,
    pub asset_class: String,
    pub notional: Option<String>,
    pub qty: String,
    pub filled_qty: String,
    pub filled_avg_price: Option<String>,
    pub order_class: Option<String>, // empty string => better as Option
    #[serde(rename = "order_type")]
    pub order_type: String,
    #[serde(rename = "type")]
    pub type_field: String, // 'type' is a reserved keyword
    pub side: String,
    pub position_intent: Option<String>,
    pub time_in_force: String,
    pub limit_price: Option<String>,
    pub stop_price: Option<String>,
    pub status: String,
    pub extended_hours: bool,
    pub legs: Option<serde_json::Value>, // or Option<Vec<Order>> if recursive
    pub trail_percent: Option<String>,
    pub trail_price: Option<String>,
    pub hwm: Option<String>,
    pub subtag: Option<String>,
    pub source: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, TypedBuilder)]
pub struct OrderRequest {
    #[builder(setter(into))]
    pub symbol: String,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<String>,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,

    #[builder(setter(into))]
    pub side: String,

    #[builder(setter(into))]
    #[serde(rename = "type")]
    pub order_type: String,

    #[builder(setter(into))]
    pub time_in_force: String,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_price: Option<String>,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_percent: Option<String>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_hours: Option<bool>,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_class: Option<String>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legs: Option<Vec<Legs>>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<TakeProfit>,

    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<StopLoss>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Legs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_intent: Option<String>,

    pub symbol: String,
    pub ratio_qty: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TakeProfit {
    pub limit_price: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StopLoss {
    pub stop_price: String,
    pub limit_price: String,
}
pub async fn create_order(
    alpaca: &Alpaca,
    order: OrderRequest,
) -> Result<Order, Box<dyn std::error::Error>> {
    let response = create_request(alpaca, Method::POST, "/v2/orders", Some(order)).await?;
    if !response.status().is_success() {
        let status = response.status();
        {
            let text = response.text().await.unwrap_or_default();
            let message = format!("Request failed with status {}: {}", status, text);
            return Err(message.into());
        }
    }
    let info: Order = response.json().await?;
    Ok(info)
}

#[derive(Serialize, Deserialize, Debug, Default, TypedBuilder)]
pub struct GetOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub limit: Option<i128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub nested: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub symbols: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub side: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub asset_class: Option<String>,
}

pub async fn get_orders(
    alpaca: &Alpaca,
    params: GetOrdersParams,
) -> Result<Vec<Order>, Box<dyn std::error::Error>> {
    // Serialize params into query string, like ?status=open&limit=50
    let query_string = serde_urlencoded::to_string(&params)?;
    let endpoint = format!("/v2/orders?{query_string}");

    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        let message = format!("Request failed: {text}");
        return Err(message.into());
    }

    let orders: Vec<Order> = response.json().await?;
    Ok(orders)
}
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCancel {
    pub id: Uuid,
    pub status: i128,
}
pub async fn delete_all_orders(
    alpaca: &Alpaca,
) -> Result<Vec<Option<OrderCancel>>, Box<dyn std::error::Error>> {
    let response = create_request::<()>(alpaca, Method::DELETE, "/v2/orders", None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        let message = format!("Request failed: {text}");
        return Err(message.into());
    }
    Ok(response.json().await?)
}

pub async fn get_order_by_client_order_id(
    alpaca: &Alpaca,
    client_order_id: &str,
) -> Result<Order, Box<dyn std::error::Error>> {
    let response = create_request::<()>(
        alpaca,
        Method::GET,
        &format!("/v2/orders:by_client_order_id?client_order_id={client_order_id}"),
        None,
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        let message = format!("Request failed: {text}");
        return Err(message.into());
    }
    Ok(response.json().await?)
}

pub async fn get_order_by_id(
    alpaca: &Alpaca,
    order_id: Uuid,
    nested: Option<bool>,
) -> Result<Order, Box<dyn std::error::Error>> {
    if nested.is_none() {
        let response =
            create_request::<()>(alpaca, Method::GET, &format!("/v2/orders/{order_id}"), None)
                .await?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            let message = format!("Request failed: {text}");
            return Err(message.into());
        }
        Ok(response.json().await?)
    } else {
        let nested = nested.unwrap_or(false);
        let response = create_request::<()>(
            alpaca,
            Method::GET,
            &format!("/v2/orders/{order_id}?nested={nested}"),
            None,
        )
        .await?;
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            let message = format!("Request failed: {text}");
            return Err(message.into());
        }
        Ok(response.json().await?)
    }
}
#[derive(Serialize, Deserialize, Debug, Default, TypedBuilder)]
pub struct ReplaceOrderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub qty: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}
pub async fn replace_order_by_id(
    alpaca: &Alpaca,
    order_id: String,
    update: ReplaceOrderParams,
) -> Result<Order, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/orders/{}", order_id);
    let response = create_request(alpaca, Method::PATCH, &endpoint, Some(update)).await?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Replace failed: {}", text).into());
    }

    let order: Order = response.json().await?;
    Ok(order)
}

pub async fn delete_order_by_id(
    alpaca: &Alpaca,
    order_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/orders/{}", order_id);
    let response = create_request::<()>(alpaca, Method::DELETE, &endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Delete failed: {}", text).into());
    }
    Ok(())
}

#[tokio::test]
async fn test_orders() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    let create_order_response = match create_order(
        &alpaca,
        OrderRequest::builder()
            .symbol("AAPL")
            .qty("1")
            .side("buy")
            .order_type("market")
            .time_in_force("day")
            .build(),
    )
    .await
    {
        Ok(order) => {
            assert_eq!(order.qty, "1");
            assert_eq!(order.side, "buy");
            assert_eq!(order.order_type, "market");
            assert_eq!(order.time_in_force, "day");
            assert_eq!(order.symbol, "AAPL");
            order
        }
        Err(e) => panic!("Error creating order: {}", e),
    };
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let get_order_response = match get_orders(
        &alpaca,
        GetOrdersParams::builder().status("all".to_string()).build(),
    )
    .await
    {
        Ok(orders) => {
            let order = match orders.clone().into_iter().find(|p| p.symbol == "AAPL") {
                Some(order) => order,
                None => {
                    println!("{orders:?}");
                    panic!("Order not found")
                }
            };
            assert_eq!(order.qty, "1");
            assert_eq!(order.side, "buy");
            assert_eq!(order.order_type, "market");
            assert_eq!(order.time_in_force, "day");
            assert_eq!(order.symbol, "AAPL");
            assert_eq!(order.client_order_id, create_order_response.client_order_id);
            order
        }
        Err(e) => panic!("Error getting orders: {}", e),
    };
    assert_eq!(create_order_response.id, get_order_response.id);
    let get_order_by_client_id = match get_order_by_client_order_id(
        &alpaca,
        &*create_order_response.client_order_id,
    )
    .await
    {
        Ok(order) => {
            assert_eq!(order.qty, "1");
            assert_eq!(order.side, "buy");
            assert_eq!(order.order_type, "market");
            assert_eq!(order.time_in_force, "day");
            assert_eq!(order.symbol, "AAPL");
            assert_eq!(order.client_order_id, create_order_response.client_order_id);
            order
        }
        Err(e) => panic!("Error getting orders by client id: {}", e),
    };
    assert_eq!(create_order_response.id, get_order_by_client_id.id);

    let get_order_by_id =
        match get_order_by_id(&alpaca, create_order_response.id.parse().unwrap(), None).await {
            Ok(order) => {
                assert_eq!(order.qty, "1");
                assert_eq!(order.side, "buy");
                assert_eq!(order.order_type, "market");
                assert_eq!(order.time_in_force, "day");
                assert_eq!(order.symbol, "AAPL");
                assert_eq!(order.client_order_id, create_order_response.client_order_id);
                order
            }
            Err(e) => panic!("Error getting orders by client id: {}", e),
        };

    assert_eq!(get_order_by_id.id, create_order_response.id);

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let create_order_response = match create_order(
        &alpaca,
        OrderRequest::builder()
            .symbol("AAPL")
            .qty("1")
            .time_in_force("day")
            .side("sell")
            .order_type("market")
            .build(),
    )
    .await
    {
        Ok(order) => {
            assert_eq!(order.qty, "1");
            assert_eq!(order.side, "sell");
            assert_eq!(order.order_type, "market");
            assert_eq!(order.time_in_force, "day");
            assert_eq!(order.symbol, "AAPL");
            order
        }
        Err(e) => panic!("Error creating sell order: {}", e),
    };
}

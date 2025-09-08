//! Account activities module for Alpaca API v2.
//!
//! This module provides functionality for retrieving account activities from Alpaca's trading API.
//! It supports querying both trading and non-trading activities with various filtering options.
//!
//! The module includes functionality for:
//! - Retrieving all account activities with date range filtering
//! - Querying specific activity types (trades, dividends, fees, etc.)
//! - Handling both trading activities (orders, fills) and non-trading activities (dividends, fees)
//! - Detailed activity information including dates, amounts, and related identifiers

use crate::auth::{Alpaca, TradingType};
use crate::request::create_trading_request;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Default, TypedBuilder)]
pub struct AccountActivitiesParams {
    #[builder(default, setter(strip_option))]
    pub activity_types: Option<Vec<String>>,
    #[builder(default, setter(strip_option))]
    pub category: Option<String>,
    #[builder(default, setter(strip_option))]
    pub date: Option<String>,
    #[builder(default, setter(strip_option))]
    pub until: Option<String>,
    #[builder(default, setter(strip_option))]
    pub after: Option<String>,
    #[builder(default, setter(strip_option))]
    pub direction: Option<String>,
    #[builder(default, setter(strip_option))]
    pub page_size: Option<i32>,
    #[builder(default, setter(strip_option))]
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityType {
    Fill,
    Trans,
    Misc,
    Acatc,
    Acats,
    Cfee,
    Csd,
    Csw,
    Div,
    Divcgl,
    Divcgs,
    Divfee,
    Divft,
    Divnra,
    Divroc,
    Divtw,
    Divtxex,
    Fee,
    Int,
    Intnra,
    Inttw,
    Jnl,
    Jnlc,
    Jnls,
    Ma,
    Nc,
    Opasn,
    Opca,
    Opcsh,
    Opexc,
    Opexp,
    Optrd,
    Ptc,
    Ptr,
    Reorg,
    Spin,
    Split,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    DoneForDay,
    Canceled,
    Expired,
    Replaced,
    PendingCancel,
    PendingReplace,
    Accepted,
    PendingNew,
    AcceptedForBidding,
    Stopped,
    Rejected,
    Suspended,
    Calculated,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountTradingActivity {
    pub id: String,
    pub activity_type: ActivityType,
    pub cum_qty: Option<String>,
    pub leaves_qty: Option<String>,
    pub price: Option<String>,
    pub qty: Option<String>,
    pub side: Option<String>, // "buy" or "sell"
    pub symbol: Option<String>,
    pub transaction_time: Option<DateTime<Utc>>,
    pub order_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub fill_type: Option<String>, // "fill" or "partial_fill"
    pub order_status: Option<OrderStatus>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountNonTradeActivity {
    pub id: String,
    pub activity_type: ActivityType,
    pub activity_sub_type: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub net_amount: Option<String>,
    pub symbol: Option<String>,
    pub cusip: Option<String>,
    pub qty: Option<String>,
    pub per_share_amount: Option<String>,
    pub group_id: Option<String>,
    pub status: Option<String>, // "executed", "correct", "canceled"
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AccountActivity {
    Trading(AccountTradingActivity),
    NonTrading(AccountNonTradeActivity),
}

/// Retrieves account activities based on the provided parameters.
///
/// This function fetches a list of account activities from Alpaca's trading API,
/// which can include both trading activities (orders, fills) and non-trading activities
/// (dividends, fees, etc.). The activities can be filtered using various parameters.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `params` - Parameters to filter the activities (date range, activity types, etc.)
///
/// # Returns
/// * `Result<Vec<AccountActivity>, Box<dyn std::error::Error>>` - A list of account activities or an error
pub async fn get_account_activities(
    alpaca: &Alpaca,
    params: AccountActivitiesParams,
) -> Result<Vec<AccountActivity>, Box<dyn std::error::Error>> {
    let base_endpoint = "/v2/account/activities";

    // Convert the params struct to a query string
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{base_endpoint}?{query_string}");

    let response =
        create_trading_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting account activities failed: {text}").into());
    }

    Ok(response.json().await?)
}
#[derive(Debug, Deserialize, Serialize, Default, TypedBuilder)]
pub struct SpecificAccountActivitiesParams {
    #[builder(default, setter(strip_option))]
    pub date: Option<String>,
    #[builder(default, setter(strip_option))]
    pub until: Option<String>,
    #[builder(default, setter(strip_option))]
    pub after: Option<String>,
    #[builder(default, setter(strip_option))]
    pub direction: Option<String>,
    #[builder(default, setter(strip_option))]
    pub page_size: Option<i32>,
    #[builder(default, setter(strip_option))]
    pub page_token: Option<String>,
}

/// Retrieves account activities of a specific type based on the provided parameters.
///
/// This function fetches a list of account activities of a specific type from Alpaca's trading API.
/// It allows for more targeted querying compared to `get_account_activities` by focusing on
/// a single activity type (e.g., fills, dividends, fees).
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `activity_type` - The specific type of activity to retrieve
/// * `params` - Parameters to filter the activities (date range, pagination, etc.)
///
/// # Returns
/// * `Result<Vec<AccountActivity>, Box<dyn std::error::Error>>` - A list of account activities of the specified type or an error
pub async fn get_specific_account_activities(
    alpaca: &Alpaca,
    activity_type: ActivityType,
    params: SpecificAccountActivitiesParams,
) -> Result<Vec<AccountActivity>, Box<dyn std::error::Error>> {
    let base_endpoint = format!("/v2/account/activities/{activity_type}");

    // Convert the params struct to a query string
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{base_endpoint}?{query_string}");

    let response =
        create_trading_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting account activities failed: {text}").into());
    }

    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_account_activities() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();

    let activities = match get_account_activities(
        &alpaca,
        AccountActivitiesParams::builder()
            .activity_types(vec!["fill".to_string()])
            .build(),
    )
    .await
    {
        Ok(activities) => {
            assert!(!activities.is_empty(), "No activities returned");
            activities
        }
        Err(e) => panic!("Error getting account activities: {}", e),
    };

    match get_specific_account_activities(
        &alpaca,
        ActivityType::Fill,
        SpecificAccountActivitiesParams::builder()
            .page_size(1)
            .build(),
    )
    .await
    {
        Ok(activities) => {
            assert!(
                activities.len() == 1,
                "Expected 1 activity, got {}",
                activities.len()
            );
            match &activities[0] {
                AccountActivity::Trading(t) => assert_eq!(t.activity_type, ActivityType::Fill),
                AccountActivity::NonTrading(n) => {
                    panic!("Expected Trading activity, got NonTrading: {:?}", n)
                }
            }
        }
        Err(e) => panic!("Error getting specific account activities: {}", e),
    }
}

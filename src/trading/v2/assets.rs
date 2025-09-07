//! Assets module for Alpaca API v2.
//!
//! This module provides functionality for retrieving information about tradable assets
//! from Alpaca's trading API. It supports querying both stock and option assets with
//! various filtering capabilities.
//!
//! The module includes functionality for:
//! - Retrieving lists of assets with filtering by status, class, and exchange
//! - Looking up specific assets by symbol
//! - Retrieving option contract information
//! - Getting detailed information about option contracts including deliverables

use crate::auth::{Alpaca, TradingType};
use crate::request::create_trading_request;
use chrono::NaiveDate;
use reqwest::Method;
use serde::{Deserialize, Deserializer, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Asset {
    pub id: String,
    pub class: String,
    pub exchange: String,
    pub symbol: String,
    pub name: String,
    pub status: String,
    pub tradable: bool,
    pub marginable: bool,
    pub maintenance_margin_requirement: u32,
    pub margin_requirement_long: String,
    pub margin_requirement_short: String,
    pub shortable: bool,
    pub easy_to_borrow: bool,
    pub fractionable: bool,
    #[serde(default, deserialize_with = "null_to_empty_vec_str")]
    pub attributes: Vec<String>,
}

/// Deserializes a JSON null value as an empty vector of strings.
///
/// This function is used as a custom deserializer for the `attributes` field in the `Asset` struct.
/// It converts JSON null values to empty vectors, allowing for more consistent handling of optional arrays.
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// * `Result<Vec<String>, D::Error>` - An empty vector if the JSON value is null, or the deserialized vector otherwise
fn null_to_empty_vec_str<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

impl Asset {
    fn get_field(&self, field: &str) -> Option<String> {
        match field {
            "id" => Some(self.id.clone()),
            "class" => Some(self.class.clone()),
            "exchange" => Some(self.exchange.clone()),
            "symbol" => Some(self.symbol.clone()),
            "name" => Some(self.name.clone()),
            "status" => Some(self.status.clone()),
            "tradable" => Some(self.tradable.to_string()),
            "marginable" => Some(self.marginable.to_string()),
            "maintenance_margin_requirement" => {
                Some(self.maintenance_margin_requirement.to_string())
            }
            "margin_requirement_long" => Some(self.margin_requirement_long.clone()),
            "margin_requirement_short" => Some(self.margin_requirement_short.clone()),
            "shortable" => Some(self.shortable.to_string()),
            "easy_to_borrow" => Some(self.easy_to_borrow.to_string()),
            "fractionable" => Some(self.fractionable.to_string()),
            "attributes" => Some(self.attributes.join(",")),
            _ => None,
        }
    }
}
/// Retrieves a list of assets based on the provided filters.
///
/// This function fetches a list of tradable assets from Alpaca's trading API,
/// with optional filtering by status, asset class, exchange, and attributes.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `status` - Optional filter for asset status (e.g., "active")
/// * `asset_class` - Optional filter for asset class (e.g., "us_equity")
/// * `exchange` - Optional filter for the exchange (e.g., "NYSE")
/// * `attributes` - Optional list of attributes to filter by
///
/// # Returns
/// * `Result<Vec<Asset>, Box<dyn std::error::Error>>` - A list of assets matching the filters or an error
pub async fn get_assets(
    alpaca: &Alpaca,
    status: Option<String>,
    asset_class: Option<String>,
    exchange: Option<String>,
    attributes: Vec<Option<String>>,
) -> Result<Vec<Asset>, Box<dyn std::error::Error>> {
    // Build query parameters
    let mut params = vec![];

    if let Some(s) = status {
        params.push(format!("status={s}"));
    }
    if let Some(s) = asset_class {
        params.push(format!("asset_class={s}"));
    }
    if let Some(s) = exchange {
        params.push(format!("exchange={s}"));
    }

    // Convert Vec<Option<String>> to comma-separated string
    let attribute_values: Vec<String> = attributes.into_iter().flatten().collect();

    if !attribute_values.is_empty() {
        params.push(format!("attributes={}", attribute_values.join(",")));
    }

    let query_string = params.join("&");
    let endpoint = format!("/v2/assets?{query_string}");

    // Make the request
    let response = create_trading_request::<()>(alpaca, Method::GET, &endpoint, None).await?;

    Ok(response.json().await?)
}

/// Retrieves information about a specific asset by its symbol.
///
/// This function fetches detailed information about a single asset identified by its trading symbol.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `symbol` - The trading symbol of the asset to retrieve
///
/// # Returns
/// * `Result<Asset, Box<dyn std::error::Error>>` - The asset information or an error
pub async fn get_asset_by_symbol(
    alpaca: &Alpaca,
    symbol: String,
) -> Result<Asset, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/assets/{symbol}");
    let response = create_trading_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
    Ok(response.json().await?)
}

#[derive(Debug, Deserialize)]
pub struct OptionContract {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub status: String,
    pub tradable: bool,
    pub root_symbol: String,
    pub expiration_date: String,
    pub underlying_symbol: String,
    pub underlying_asset_id: String,
    #[serde(rename = "type")]
    pub contract_type: String,
    pub style: String,
    pub strike_price: String,
    pub multiplier: String,
    pub size: String,
    pub open_interest: Option<String>,
    pub open_interest_date: Option<String>,
    pub close_price: Option<String>,
    pub close_price_date: Option<String>,
    pub ppind: bool,
}

#[derive(Debug, Deserialize)]
pub struct GetOptionContractsResponse {
    pub option_contracts: Vec<OptionContract>,
    #[serde(rename = "next_page_token")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Default, Serialize, TypedBuilder)]
pub struct GetOptionContractsParams {
    #[builder(default, setter(strip_option))]
    underlying_symbols: Option<String>, // comma-separated
    #[builder(default, setter(strip_option))]
    status: Option<String>,
    #[builder(default, setter(strip_option))]
    expiration_date: Option<NaiveDate>,
    #[builder(default, setter(strip_option))]
    expiration_date_gte: Option<NaiveDate>,
    #[builder(default, setter(strip_option))]
    expiration_date_lte: Option<NaiveDate>,
    #[builder(default, setter(strip_option))]
    root_symbol: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "type")]
    contract_type: Option<String>,
    #[builder(default, setter(strip_option))]
    style: Option<String>,
    #[builder(default, setter(strip_option))]
    strike_price_gte: Option<f64>,
    #[builder(default, setter(strip_option))]
    strike_price_lte: Option<f64>,
    #[builder(default, setter(strip_option))]
    limit: Option<u32>,
    #[builder(default, setter(strip_option))]
    page_token: Option<String>,
    #[builder(default, setter(strip_option))]
    ppind: Option<bool>,
    #[builder(default, setter(strip_option))]
    show_deliverables: Option<bool>,
}

/// Retrieves a list of option contracts based on the provided parameters.
///
/// This function fetches option contracts from Alpaca's trading API with various filtering options
/// such as underlying symbol, expiration date, strike price, and contract type.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `params` - Parameters to filter the option contracts
///
/// # Returns
/// * `Result<GetOptionContractsResponse, Box<dyn std::error::Error>>` - A response containing option contracts and pagination information, or an error
pub async fn get_option_contracts(
    alpaca: &Alpaca,
    params: GetOptionContractsParams,
) -> Result<GetOptionContractsResponse, Box<dyn std::error::Error>> {
    // build query string
    let mut query_pairs = vec![];
    if let Some(v) = params.underlying_symbols {
        query_pairs.push(("underlying_symbols", v));
    }
    if let Some(v) = params.status {
        query_pairs.push(("status", v));
    }
    if let Some(d) = params.expiration_date {
        query_pairs.push(("expiration_date", d.format("%Y-%m-%d").to_string()));
    }
    if let Some(d) = params.expiration_date_gte {
        query_pairs.push(("expiration_date_gte", d.format("%Y-%m-%d").to_string()));
    }
    if let Some(d) = params.expiration_date_lte {
        query_pairs.push(("expiration_date_lte", d.format("%Y-%m-%d").to_string()));
    }
    if let Some(v) = params.root_symbol {
        query_pairs.push(("root_symbol", v));
    }
    if let Some(v) = params.contract_type {
        query_pairs.push(("type", v));
    }
    if let Some(v) = params.style {
        query_pairs.push(("style", v));
    }
    if let Some(v) = params.strike_price_gte {
        query_pairs.push(("strike_price_gte", v.to_string()));
    }
    if let Some(v) = params.strike_price_lte {
        query_pairs.push(("strike_price_lte", v.to_string()));
    }
    if let Some(v) = params.limit {
        query_pairs.push(("limit", v.to_string()));
    }
    if let Some(v) = params.page_token {
        query_pairs.push(("page_token", v));
    }
    if let Some(v) = params.ppind {
        query_pairs.push(("ppind", v.to_string()));
    }
    if let Some(v) = params.show_deliverables {
        query_pairs.push(("show_deliverables", v.to_string()));
    }

    let query_string = serde_urlencoded::to_string(query_pairs)?;
    let endpoint = if query_string.is_empty() {
        "/v2/options/contracts".to_string()
    } else {
        format!("/v2/options/contracts?{query_string}")
    };

    let response = create_trading_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
    Ok(response.json::<GetOptionContractsResponse>().await?)
}
#[derive(Debug, Deserialize)]
pub struct OptionContractBySymbol {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub status: String,
    pub tradable: bool,
    pub expiration_date: String,
    pub root_symbol: String,
    pub underlying_symbol: String,
    pub underlying_asset_id: String,
    #[serde(rename = "type")]
    pub contract_type: String,
    pub style: String,
    pub strike_price: String,
    pub multiplier: String,
    pub size: String,
    pub open_interest: String,
    pub open_interest_date: String,
    pub close_price: String,
    pub close_price_date: String,
    pub deliverables: Vec<Deliverable>,
    pub ppind: bool,
}

#[derive(Debug, Deserialize)]
pub struct Deliverable {
    #[serde(rename = "type")]
    pub deliverable_type: String,
    pub symbol: String,
    pub asset_id: String,
    pub amount: String,
    pub allocation_percentage: String,
    pub settlement_type: String,
    pub settlement_method: String,
    pub delayed_settlement: bool,
}
/// Retrieves detailed information about a specific option contract by its symbol.
///
/// This function fetches comprehensive information about a single option contract,
/// including deliverables and contract specifications.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `symbol` - The option contract symbol (e.g., "AAPL230616C00150000")
///
/// # Returns
/// * `Result<OptionContractBySymbol, Box<dyn std::error::Error>>` - Detailed option contract information or an error
pub async fn get_option_contracts_by_symbol(
    alpaca: &Alpaca,
    symbol: String,
) -> Result<OptionContractBySymbol, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/options/contracts/{symbol}");
    let response = create_trading_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
    Ok(response.json::<OptionContractBySymbol>().await?)
}

#[tokio::test]
async fn test_assets() {
    let alpaca = Alpaca::from_env(TradingType::Paper).expect("Failed to read env");
    match get_assets(&alpaca, None, None, None, vec![None]).await {
        Ok(assets) => {
            let results: Vec<&Asset> = assets
                .iter()
                .filter(|asset| asset.get_field("symbol") == Some("AAPL".to_string()))
                .collect();
            assert_eq!(results[0].symbol, "AAPL");
            let results: Vec<&Asset> = assets
                .iter()
                .filter(|asset| asset.get_field("symbol") == Some("OGGNF".to_string()))
                .collect();
            assert_eq!(results[0].id, "9ba5e076-680f-432f-9519-76ddeb000a24");
        }
        Err(e) => {
            println!("Failed to get assets with error: {e}");
            assert!(false);
        }
    }

    match get_asset_by_symbol(&alpaca, String::from("OGGNF")).await {
        Ok(asset) => {
            assert_eq!(asset.symbol, "OGGNF");
            assert_eq!(asset.get_field("symbol").unwrap(), "OGGNF");
            assert_eq!(
                asset.get_field("id").unwrap(),
                "9ba5e076-680f-432f-9519-76ddeb000a24"
            );
        }
        Err(e) => {
            println!("Failed to get assets by symbol with error: {e}");
            assert!(false);
        }
    }
}

#[tokio::test]
async fn test_options() {
    let alpaca = Alpaca::from_env(TradingType::Paper).expect("Failed to read env");
    let options = match get_option_contracts(
        &alpaca,
        GetOptionContractsParams::builder()
            .root_symbol("AAPL".to_string())
            .build(),
    )
    .await
    {
        Ok(options) => {
            assert_eq!(options.option_contracts[0].root_symbol, "AAPL");
            options
        }
        Err(e) => {
            println!("Failed to get options with error: {e}");
            assert!(false);
            return;
        }
    };
    match get_option_contracts_by_symbol(&alpaca, options.option_contracts[0].symbol.clone()).await
    {
        Ok(options) => {
            assert_eq!(options.root_symbol, "AAPL");
        }
        Err(e) => {
            println!("Failed to get options with error: {e}");
            assert!(false);
        }
    }
}

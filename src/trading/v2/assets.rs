use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use chrono::NaiveDate;
use reqwest::Method;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Asset {
    id: String,
    class: String,
    exchange: String,
    symbol: String,
    name: String,
    status: String,
    tradable: bool,
    marginable: bool,
    maintenance_margin_requirement: u32,
    margin_requirement_long: String,
    margin_requirement_short: String,
    shortable: bool,
    easy_to_borrow: bool,
    fractionable: bool,
    #[serde(default, deserialize_with = "null_to_empty_vec_str")]
    attributes: Vec<String>,
}

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
    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;

    Ok(response.json().await?)
}

pub async fn get_asset_by_symbol(
    alpaca: &Alpaca,
    symbol: String,
) -> Result<Asset, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/assets/{symbol}");
    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
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

#[derive(Debug, Default, Serialize)]
pub struct GetOptionContractsParams {
    underlying_symbols: Option<String>, // comma-separated
    status: Option<String>,
    expiration_date: Option<NaiveDate>,
    expiration_date_gte: Option<NaiveDate>,
    expiration_date_lte: Option<NaiveDate>,
    root_symbol: Option<String>,
    #[serde(rename = "type")]
    contract_type: Option<String>,
    style: Option<String>,
    strike_price_gte: Option<f64>,
    strike_price_lte: Option<f64>,
    limit: Option<u32>,
    page_token: Option<String>,
    ppind: Option<bool>,
    show_deliverables: Option<bool>,
}

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

    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
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
pub async fn get_option_contracts_by_symbol(
    alpaca: &Alpaca,
    symbol: String,
) -> Result<OptionContractBySymbol, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/options/contracts/{symbol}");
    let response = create_request::<()>(alpaca, Method::GET, &endpoint, None).await?;
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
    let params = GetOptionContractsParams {
        root_symbol: Option::from("AAPL".to_string()),
        ..Default::default()
    };
    let options = match get_option_contracts(&alpaca, params).await {
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

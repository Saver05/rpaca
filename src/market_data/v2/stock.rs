use crate::auth::{Alpaca, TradingType};
use crate::request::create_data_request;
use reqwest::Method;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use typed_builder::TypedBuilder;

fn serialize_symbols<S>(symbols: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let joined = symbols.join(",");
    serializer.serialize_str(&joined)
}
#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalAuctionsParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u16>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "asof")]
    asof_date: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[builder(default =Some("asc".to_string()), setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionsResponse {
    pub auctions: HashMap<String, Vec<AuctionDay>>,

    pub currency: Option<String>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionDay {
    /// Date in RFC-3339 format.
    #[serde(rename = "d")]
    pub date: String,

    /// Opening auctions.
    #[serde(rename = "o")]
    pub opening: Vec<AuctionPrint>,

    /// Closing auctions (optional in your example).
    #[serde(rename = "c")]
    pub closing: Option<Vec<AuctionPrint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionPrint {
    /// Timestamp in RFC-3339 with nanosecond precision.
    #[serde(rename = "t")]
    pub timestamp: String,

    /// Exchange code.
    #[serde(rename = "x")]
    pub exchange: String,

    /// Auction price.
    #[serde(rename = "p")]
    pub price: f64,

    /// Auction trade size.
    #[serde(rename = "s")]
    pub size: Option<i64>,

    /// Condition flag.
    #[serde(rename = "c")]
    pub condition: String,
}

impl AuctionsResponse {
    /// Get auction days for a specific symbol
    pub fn get_symbol_data(&self, symbol: &str) -> Option<&Vec<AuctionDay>> {
        self.auctions.get(symbol)
    }

    /// Get all symbols in the response
    pub fn symbols(&self) -> Vec<&String> {
        self.auctions.keys().collect()
    }

    /// Check if data exists for a symbol
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.auctions.contains_key(symbol)
    }

    /// Get the latest auction day for a symbol
    pub fn get_latest_day(&self, symbol: &str) -> Option<&AuctionDay> {
        self.auctions.get(symbol)?.last()
    }

    /// Get all opening prices for a symbol
    pub fn get_opening_prices(&self, symbol: &str) -> Vec<f64> {
        self.auctions
            .get(symbol)
            .map(|days| {
                days.iter()
                    .flat_map(|day| &day.opening)
                    .map(|auction| auction.price)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_closing_prices(&self, symbol: &str) -> Vec<f64> {
        self.auctions
            .get(symbol)
            .map(|days| {
                days.iter()
                    .filter_map(|day| day.closing.as_ref())
                    .flatten()
                    .map(|auction| auction.price)
                    .collect()
            })
            .unwrap_or_default()
    }
}
pub async fn get_historical_auctions(
    alpaca: &Alpaca,
    params: HistoricalAuctionsParams,
) -> Result<AuctionsResponse, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/auctions";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting historical auctions failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_historial_auctions() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_historical_auctions(
        &alpaca,
        HistoricalAuctionsParams::builder()
            .symbols(vec!["AAPL".to_string()])
            .start("2024-01-03T00:00:00Z".to_string())
            .end("2024-01-04T01:02:03.123456789Z".to_string())
            .limit(1)
            .feed("sip".to_string())
            .currency("USD".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(res.has_symbol("AAPL"));
            assert_eq!(
                res.get_symbol_data("AAPL").unwrap().first().unwrap().date,
                "2024-01-03".to_string()
            );
            assert_eq!(*res.get_opening_prices("AAPL").first().unwrap(), 184.22);
            assert_eq!(*res.get_closing_prices("AAPL").first().unwrap(), 184.24);
            assert!(res.has_symbol("AAPL"));
        }
        Err(e) => panic!("Error getting historical auctions: {}", e),
    }
}

#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalBarParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    timeframe: String,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u16>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    adjustment: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarResponse {
    bars: HashMap<String, Vec<Bars>>,
    next_page_token: String,
    currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bars {
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: i64,
    #[serde(rename = "n")]
    pub count: i64,
    #[serde(rename = "vw")]
    pub volume_weighted_average: f64,
}

impl BarResponse {
    /* =========================
    Basic access / metadata
    ========================= */

    /// List all symbols present.
    pub fn symbols(&self) -> impl Iterator<Item = &str> {
        self.bars.keys().map(|s| s.as_str())
    }

    /// Borrow bars for a symbol.
    pub fn bars_for(&self, symbol: &str) -> Option<&[Bars]> {
        self.bars.get(symbol).map(|v| v.as_slice())
    }

    /// Mutable access (if you need to transform/sort).
    pub fn bars_for_mut(&mut self, symbol: &str) -> Option<&mut Vec<Bars>> {
        self.bars.get_mut(symbol)
    }

    /// Total number of bars across all symbols.
    pub fn len_total(&self) -> usize {
        self.bars.values().map(|v| v.len()).sum()
    }

    /// True if there are no bars for any symbol.
    pub fn is_empty(&self) -> bool {
        self.bars.values().all(|v| v.is_empty())
    }

    /// Convenience: treat empty string as "no more pages".
    pub fn next_page_token(&self) -> Option<&str> {
        if self.next_page_token.is_empty() {
            None
        } else {
            Some(self.next_page_token.as_str())
        }
    }

    pub fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }

    /* =========================
    Per-symbol convenience
    ========================= */

    pub fn first_bar(&self, symbol: &str) -> Option<&Bars> {
        self.bars.get(symbol).and_then(|v| v.first())
    }

    pub fn last_bar(&self, symbol: &str) -> Option<&Bars> {
        self.bars.get(symbol).and_then(|v| v.last())
    }

    pub fn closing_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.close).collect())
            .unwrap_or_default()
    }

    pub fn opening_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.open).collect())
            .unwrap_or_default()
    }

    pub fn high_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.high).collect())
            .unwrap_or_default()
    }

    pub fn low_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.low).collect())
            .unwrap_or_default()
    }

    pub fn volumes(&self, symbol: &str) -> Vec<i64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.volume).collect())
            .unwrap_or_default()
    }

    pub fn counts(&self, symbol: &str) -> Vec<i64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.count).collect())
            .unwrap_or_default()
    }

    pub fn vwap_values(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.volume_weighted_average).collect())
            .unwrap_or_default()
    }

    /// Average close for a symbol.
    pub fn avg_close(&self, symbol: &str) -> Option<f64> {
        let v = self.bars.get(symbol)?;
        if v.is_empty() {
            return None;
        }
        Some(v.iter().map(|b| b.close).sum::<f64>() / v.len() as f64)
    }

    pub fn max_high(&self, symbol: &str) -> Option<f64> {
        self.bars
            .get(symbol)?
            .iter()
            .map(|b| b.high)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn min_low(&self, symbol: &str) -> Option<f64> {
        self.bars
            .get(symbol)?
            .iter()
            .map(|b| b.low)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn total_volume(&self, symbol: &str) -> Option<i64> {
        Some(self.bars.get(symbol)?.iter().map(|b| b.volume).sum())
    }

    /* =========================
    Cross-symbol utilities
    ========================= */

    /// Flatten an iterator over all bars for all symbols.
    pub fn iter_all(&self) -> impl Iterator<Item = (&str, &Bars)> {
        self.bars
            .iter()
            .flat_map(|(sym, v)| v.iter().map(move |b| (sym.as_str(), b)))
    }

    pub fn max_high_all(&self) -> Option<(&str, f64)> {
        self.iter_all()
            .max_by(|(_, a), (_, b)| a.high.partial_cmp(&b.high).unwrap())
            .map(|(s, b)| (s, b.high))
    }

    pub fn min_low_all(&self) -> Option<(&str, f64)> {
        self.iter_all()
            .min_by(|(_, a), (_, b)| a.low.partial_cmp(&b.low).unwrap())
            .map(|(s, b)| (s, b.low))
    }

    pub fn total_volume_all(&self) -> i64 {
        self.bars.values().flatten().map(|b| b.volume).sum()
    }
}

pub async fn get_historical_bars(
    alpaca: &Alpaca,
    params: HistoricalBarParams,
) -> Result<BarResponse, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/bars";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting historical bars failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_historical_bars() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_historical_bars(
        &alpaca,
        HistoricalBarParams::builder()
            .symbols(vec!["AAPL".to_string()])
            .timeframe("1Min".to_string())
            .start("2024-01-03T00:00:00Z".to_string())
            .end("2024-01-04T01:02:03.123456789Z".to_string())
            .limit(1)
            .feed("sip".to_string())
            .currency("USD".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(!res.is_empty());
            assert_eq!(res.len_total(), 1);
            assert_eq!(res.first_bar("AAPL").unwrap().close, 185.31);
            assert_eq!(res.last_bar("AAPL").unwrap().open, 185.31);
            assert_eq!(
                res.first_bar("AAPL").unwrap().timestamp,
                "2024-01-03T00:00:00Z"
            );
        }
        Err(e) => panic!("Error getting historical bars: {}", e),
    }
}
#[derive(Debug, TypedBuilder, Serialize)]
pub struct LatestBarsParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestBarsResponse {
    pub bars: HashMap<String, Bars>,
    #[serde(default)]
    pub next_page_token: Option<String>, // usually absent for "latest"
    #[serde(default)]
    pub currency: Option<String>,
}

// Helpers for LatestBarsResponse
impl LatestBarsResponse {
    pub fn bar(&self, symbol: &str) -> Option<&Bars> {
        self.bars.get(symbol)
    }

    pub fn symbols(&self) -> impl Iterator<Item = &str> {
        self.bars.keys().map(|s| s.as_str())
    }

    pub fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref().filter(|s| !s.is_empty())
    }

    pub fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }
}

pub async fn get_latest_bars(
    alpaca: &Alpaca,
    params: LatestBarsParams,
) -> Result<LatestBarsResponse, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/bars/latest";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting latest bars failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_latest_bars() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_latest_bars(
        &alpaca,
        LatestBarsParams::builder()
            .symbols(vec!["AAPL".to_string()])
            .feed("iex".to_string())
            .currency("USD".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(res.symbols().any(|s| s == "AAPL"));
        }
        Err(e) => panic!("Error getting latest bar: {}", e),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TradeConditionResponse(pub HashMap<char, String>);

impl TradeConditionResponse {
    /// Lookup by a single char code, e.g. ' ', '4', 'B', 'T'
    pub fn describe(&self, code: char) -> Option<&str> {
        self.0.get(&code).map(|s| s.as_str())
    }

    /// Convenience: accept &str like "B" or "4" and take the first char
    pub fn describe_str(&self, code: &str) -> Option<&str> {
        code.chars().next().and_then(|c| self.describe(c))
    }
}
#[derive(Serialize)]
struct CondQuery<'a> {
    tape: &'a str,
}

pub async fn get_condition_codes(
    alpaca: &Alpaca,
    ticktype: &str,
    tape: &str,
) -> Result<TradeConditionResponse, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/stocks/meta/conditions/{ticktype}");
    let query_string = serde_qs::to_string(&CondQuery { tape })?; // "tape=A"
    let endpoint_with_query = format!("{endpoint}?{query_string}");

    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting condition codes failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_condition_codes() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_condition_codes(&alpaca, "trade", "A").await {
        Ok(res) => {
            assert_eq!(res.describe('4'), Some("Derivatively Priced"));
            assert_eq!(res.describe('Z'), Some("Sold (Out Of Sequence)"))
        }
        Err(e) => panic!("Error getting condition codes: {e}"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExchangeCodesResponse(pub HashMap<char, String>);

impl ExchangeCodesResponse {
    /// Lookup by a single char code, e.g. '', '4', 'B', 'T'
    pub fn describe(&self, code: char) -> Option<&str> {
        self.0.get(&code).map(|s| s.as_str())
    }

    /// Convenience: accept &str like "B" or "4" and take the first char
    pub fn describe_str(&self, code: &str) -> Option<&str> {
        code.chars().next().and_then(|c| self.describe(c))
    }
}

pub async fn get_exchance_codes(
    alpaca: &Alpaca,
) -> Result<ExchangeCodesResponse, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/meta/exchanges";
    let response = create_data_request::<()>(alpaca, Method::GET, endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting exchange codes failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_exchange_codes() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_exchance_codes(&alpaca).await {
        Ok(res) => {
            assert_eq!(res.describe('A'), Some("NYSE American (AMEX)"));
            assert_eq!(res.describe('Z'), Some("Cboe BZ"))
        }
        Err(e) => panic!("Error getting exchange codes: {e}"),
    }
}
#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalQuotesParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalQuotes {
    pub quotes: HashMap<String, Vec<Quotes>>, // symbol → quotes
    #[serde(default)]
    pub currency: Option<String>,
    pub next_page_token: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotes {
    #[serde(rename = "t")]
    timestamp: String,
    #[serde(rename = "bx")]
    bid_exchange: String,
    #[serde(rename = "bp")]
    bid_price: f64,
    #[serde(rename = "bs")]
    bid_size: u64,
    #[serde(rename = "ax")]
    ask_exchange: String,
    #[serde(rename = "ap")]
    ask_price: f64,
    #[serde(rename = "as")]
    ask_size: u64,
    #[serde(rename = "c")]
    condition_flags: Vec<String>,
    #[serde(rename = "z")]
    exchange: String,
}
impl HistoricalQuotes {
    /// Get all quotes for a specific symbol.
    pub fn get_symbol_quotes(&self, symbol: &str) -> Option<&Vec<Quotes>> {
        self.quotes.get(symbol)
    }

    /// Get all symbols present in the response.
    pub fn symbols(&self) -> Vec<&String> {
        self.quotes.keys().collect()
    }

    /// Check if the response contains data for a symbol.
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.quotes.contains_key(symbol)
    }

    /// Get the most recent quote for a symbol (by last element in Vec).
    pub fn get_last_quote(&self, symbol: &str) -> Option<&Quotes> {
        self.quotes.get(symbol)?.last()
    }

    /// Get all bid prices for a symbol.
    pub fn get_bid_prices(&self, symbol: &str) -> Vec<f64> {
        self.quotes
            .get(symbol)
            .map(|qs| qs.iter().map(|q| q.bid_price).collect())
            .unwrap_or_default()
    }

    /// Get all ask prices for a symbol.
    pub fn get_ask_prices(&self, symbol: &str) -> Vec<f64> {
        self.quotes
            .get(symbol)
            .map(|qs| qs.iter().map(|q| q.ask_price).collect())
            .unwrap_or_default()
    }

    /// Get all timestamps for a symbol.
    pub fn get_timestamps(&self, symbol: &str) -> Vec<&str> {
        self.quotes
            .get(symbol)
            .map(|qs| qs.iter().map(|q| q.timestamp.as_str()).collect())
            .unwrap_or_default()
    }

    /// Convenience: true if there’s another page of data.
    pub fn has_next_page(&self) -> bool {
        self.next_page_token
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }
}

pub async fn get_historical_quotes(
    alpaca: &Alpaca,
    params: HistoricalQuotesParams,
) -> Result<HistoricalQuotes, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/quotes";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting historical quotes failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_historical_quotes() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_historical_quotes(
        &alpaca,
        HistoricalQuotesParams::builder()
            .symbols(vec!["AAPL".parse().unwrap()])
            .start("2024-01-03T00:00:00Z".to_string())
            .end("2024-01-04T01:02:03.123456789Z".to_string())
            .limit(1)
            .feed("iex".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(!res.quotes.is_empty());
            assert_eq!(res.get_bid_prices("AAPL").first(), Some(184.42).as_ref());
        }
        Err(e) => panic!("Error getting historical quotes: {}", e),
    }
}

#[derive(Debug, TypedBuilder, Serialize)]
pub struct LatestQuotesParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestQuotes {
    pub quotes: HashMap<String, Quotes>,
    #[serde(default)]
    pub currency: Option<String>,
}

impl LatestQuotes {
    pub fn get_symbol_quote(&self, symbol: &str) -> Option<&Quotes> {
        self.quotes.get(symbol)
    }

    pub fn symbols(&self) -> Vec<&String> {
        self.quotes.keys().collect()
    }

    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.quotes.contains_key(symbol)
    }

    pub fn get_last_quote(&self, symbol: &str) -> Option<&Quotes> {
        // kept for API compatibility; same as get_symbol_quote now
        self.quotes.get(symbol)
    }

    pub fn get_bid_price(&self, symbol: &str) -> Option<f64> {
        self.quotes.get(symbol).map(|q| q.bid_price)
    }

    pub fn get_ask_price(&self, symbol: &str) -> Option<f64> {
        self.quotes.get(symbol).map(|q| q.ask_price)
    }

    pub fn get_timestamp(&self, symbol: &str) -> Option<&str> {
        self.quotes.get(symbol).map(|q| q.timestamp.as_str())
    }
}

pub async fn get_latest_quotes(
    alpaca: &Alpaca,
    params: LatestQuotesParams,
) -> Result<LatestQuotes, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/quotes/latest";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting latest quotes failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_latest_quotes() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_latest_quotes(
        &alpaca,
        LatestQuotesParams::builder()
            .symbols(vec!["AAPL".parse().unwrap()])
            .feed("iex".to_string())
            .currency("USD".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(res.has_symbol("AAPL"));
        }
        Err(e) => panic!("Error getting latest quotes: {e}"),
    }
}

#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalTradesParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTrades {
    pub trades: HashMap<String, Vec<Trades>>,
    #[serde(default)]
    pub currency: Option<String>,
    pub next_page_token: Option<String>,
}
impl HistoricalTrades {
    /// Get trades for a specific symbol
    pub fn trades_for_symbol(&self, symbol: &str) -> Option<&Vec<Trades>> {
        self.trades.get(symbol)
    }

    /// Get the first trade for a specific symbol
    pub fn first_trade(&self, symbol: &str) -> Option<&Trades> {
        self.trades.get(symbol)?.first()
    }

    /// Get the last trade for a specific symbol
    pub fn last_trade(&self, symbol: &str) -> Option<&Trades> {
        self.trades.get(symbol)?.last()
    }

    /// Flatten all trades into a single vector
    pub fn all_trades(&self) -> Vec<(&String, &Trades)> {
        self.trades
            .iter()
            .flat_map(|(sym, trades)| trades.iter().map(move |t| (sym, t)))
            .collect()
    }

    /// Count total number of trades across all symbols
    pub fn total_trade_count(&self) -> usize {
        self.trades.values().map(|v| v.len()).sum()
    }

    /// Get a map of symbol -> number of trades
    pub fn counts_per_symbol(&self) -> HashMap<&String, usize> {
        self.trades
            .iter()
            .map(|(sym, trades)| (sym, trades.len()))
            .collect()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trades {
    #[serde(rename = "t")]
    timestamp: String,
    #[serde(rename = "x")]
    exchange: String,
    #[serde(rename = "p")]
    price: f64,
    #[serde(rename = "s")]
    size: u64,
    #[serde(rename = "i")]
    trade_id: u64,
    #[serde(rename = "c")]
    condition_flags: Vec<String>,
    #[serde(rename = "z")]
    exchange_code: String,
    #[serde(rename = "u")]
    #[serde(default)]
    update: Option<String>,
}

pub async fn get_historical_trades(
    alpaca: &Alpaca,
    params: HistoricalTradesParams,
) -> Result<HistoricalTrades, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/trades";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting historical trades failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_hisotrical_trades() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_historical_trades(
        &alpaca,
        HistoricalTradesParams::builder()
            .symbols(vec!["AAPL".parse().unwrap()])
            .start("2024-01-03T00:00:00Z".to_string())
            .end("2024-01-04T01:02:03.123456789Z".to_string())
            .limit(1)
            .feed("iex".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(res.first_trade("AAPL").is_some());
            assert_eq!(res.first_trade("AAPL").unwrap().price, 184.37);
            assert_eq!(
                res.last_trade("AAPL").unwrap().timestamp,
                "2024-01-03T13:00:13.51278393Z"
            );
        }
        Err(e) => panic!("Error getting historical trades: {e}"),
    }
}

#[derive(Debug, TypedBuilder, Serialize)]
pub struct LatestTradesParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestTrades {
    pub trades: HashMap<String, Trades>,
    #[serde(default)]
    pub currency: Option<String>,
}

impl LatestTrades {
    /// Get trade for a specific symbol
    pub fn trade_for_symbol(&self, symbol: &str) -> Option<&Trades> {
        self.trades.get(symbol)
    }

    /// Flatten all trades into a single vector
    pub fn all_trades(&self) -> Vec<(&String, &Trades)> {
        self.trades.iter().collect()
    }

    /// Count total number of trades (will equal number of symbols)
    pub fn total_trade_count(&self) -> usize {
        self.trades.len()
    }

    /// Get a map of symbol -> trade count (always 1 here)
    pub fn counts_per_symbol(&self) -> HashMap<&String, usize> {
        self.trades.iter().map(|(sym, _)| (sym, 1)).collect()
    }
}

pub async fn get_latest_trades(
    alpaca: &Alpaca,
    params: LatestTradesParams,
) -> Result<LatestTrades, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/trades/latest";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting latest trades failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_latest_trades() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_latest_trades(
        &alpaca,
        LatestTradesParams::builder()
            .symbols(vec!["AAPL".parse().unwrap()])
            .feed("iex".to_string())
            .currency("USD".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(res.trade_for_symbol("AAPL").is_some());
        }
        Err(e) => panic!("Error getting latest trades: {e}"),
    }
}

#[derive(Debug, TypedBuilder, Serialize)]
pub struct SnapshotsParams {
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResponse(pub HashMap<String, StockData>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub dailyBar: Bars,
    pub latestQuote: Quotes,
    pub latestTrade: Trades,
    pub minuteBar: Bars,
    pub prevDailyBar: Bars,
}

impl SnapshotResponse {
    /// Get StockData for a symbol if it exists
    pub fn get(&self, symbol: &str) -> Option<&StockData> {
        self.0.get(symbol)
    }

    /// Mutable access to StockData
    pub fn get_mut(&mut self, symbol: &str) -> Option<&mut StockData> {
        self.0.get_mut(symbol)
    }

    /// List all symbols
    pub fn symbols(&self) -> Vec<&String> {
        self.0.keys().collect()
    }

    /// Return all StockData entries
    pub fn all(&self) -> impl Iterator<Item = (&String, &StockData)> {
        self.0.iter()
    }
}

impl StockData {
    /// Get the latest trade price
    pub fn latest_price(&self) -> f64 {
        self.latestTrade.price
    }

    /// Get the spread between bid and ask
    pub fn spread(&self) -> f64 {
        self.latestQuote.ask_price - self.latestQuote.bid_price
    }

    /// Get daily OHLC data as tuple
    pub fn daily_ohlc(&self) -> (f64, f64, f64, f64) {
        (
            self.dailyBar.open,
            self.dailyBar.high,
            self.dailyBar.low,
            self.dailyBar.close,
        )
    }

    /// Check if price is above previous daily close
    pub fn is_above_prev_close(&self) -> bool {
        self.latestTrade.price > self.prevDailyBar.close
    }
}

pub async fn get_snapshots(
    alpaca: &Alpaca,
    params: SnapshotsParams,
) -> Result<SnapshotResponse, Box<dyn std::error::Error>> {
    let endpoint = "/v2/stocks/snapshots";
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{endpoint}?{query_string}");
    let response =
        create_data_request::<()>(alpaca, Method::GET, &endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting snapshot: {text}").into());
    }
    Ok(response.json().await?)
}

#[tokio::test]
async fn test_get_snapshots() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_snapshots(
        &alpaca,
        SnapshotsParams::builder()
            .symbols(vec!["AAPL".parse().unwrap()])
            .feed("iex".to_string())
            .currency("USD".to_string())
            .build(),
    )
    .await
    {
        Ok(res) => {
            assert!(res.get("AAPL").is_some());
        }
        Err(e) => panic!("Error getting snapshots: {e}"),
    }
}

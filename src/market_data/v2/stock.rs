use crate::auth::{Alpaca, TradingType};
use crate::request::create_data_request;
use reqwest::Method;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use typed_builder::TypedBuilder;

/// Serializes a vector of stock symbols into a comma-separated string.
///
/// This function is used by serde to convert a Vec<String> of stock symbols
/// into a single comma-separated string for API requests.
///
/// # Arguments
/// * `symbols` - A vector of stock symbols to serialize
/// * `serializer` - The serializer to use
///
/// # Returns
/// * Result containing the serialized string or an error
fn serialize_symbols<S>(symbols: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let joined = symbols.join(",");
    serializer.serialize_str(&joined)
}
/// Parameters for retrieving historical auction data from the Alpaca API.
///
/// This struct is used to build requests for historical auction data, including
/// opening and closing auctions for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalAuctionsParams {
    /// List of stock symbols to retrieve auction data for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Start time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,

    /// End time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,

    /// Maximum number of data points to return.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u16>,

    /// Query for data as of this date (for historical snapshots).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "asof")]
    asof_date: Option<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,

    /// Token for pagination to get the next page of results.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,

    /// Sort order for results, defaults to "asc" (ascending).
    #[builder(default =Some("asc".to_string()), setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}

/// Response from the historical auctions API endpoint.
///
/// Contains auction data for requested symbols, organized by symbol and day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionsResponse {
    /// Map of symbol to a vector of auction days.
    /// Each symbol has a list of days with auction data.
    pub auctions: HashMap<String, Vec<AuctionDay>>,

    /// Currency used for the price values (e.g., "USD").
    pub currency: Option<String>,

    /// Token for pagination to get the next page of results.
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

/// Methods for accessing and manipulating auction data.
impl AuctionsResponse {
    /// Get auction days for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve data for
    ///
    /// # Returns
    /// * `Option<&Vec<AuctionDay>>` - Vector of auction days if the symbol exists, None otherwise
    pub fn get_symbol_data(&self, symbol: &str) -> Option<&Vec<AuctionDay>> {
        self.auctions.get(symbol)
    }

    /// Get all symbols in the response.
    ///
    /// # Returns
    /// * `Vec<&String>` - A vector of all symbols in the response
    pub fn symbols(&self) -> Vec<&String> {
        self.auctions.keys().collect()
    }

    /// Check if data exists for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to check
    ///
    /// # Returns
    /// * `bool` - True if the symbol exists in the response, false otherwise
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.auctions.contains_key(symbol)
    }

    /// Get the latest auction day for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the latest day for
    ///
    /// # Returns
    /// * `Option<&AuctionDay>` - The latest auction day if the symbol exists, None otherwise
    pub fn get_latest_day(&self, symbol: &str) -> Option<&AuctionDay> {
        self.auctions.get(symbol)?.last()
    }

    /// Get all opening prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve opening prices for
    ///
    /// # Returns
    /// * `Vec<f64>` - A vector of all opening prices for the symbol
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

    /// Get all closing prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve closing prices for
    ///
    /// # Returns
    /// * `Vec<f64>` - A vector of all closing prices for the symbol
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
/// Retrieves historical auction data from the Alpaca API.
///
/// This function fetches historical auction data for specified stock symbols,
/// including opening and closing auctions.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `params` - Parameters for the historical auctions request
///
/// # Returns
/// * `Result<AuctionsResponse, Box<dyn std::error::Error>>` - The auction data or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let params = HistoricalAuctionsParams::builder()
///     .symbols(vec!["AAPL".to_string()])
///     .start("2024-01-03T00:00:00Z".to_string())
///     .end("2024-01-04T00:00:00Z".to_string())
///     .build();
/// let auctions = get_historical_auctions(&alpaca, params).await?;
///
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
        Err(e) => panic!("Error getting historical auctions: {e}"),
    }
}

/// Parameters for retrieving historical bar (OHLC) data from the Alpaca API.
///
/// This struct is used to build requests for historical price bars (candles) with
/// open, high, low, close, and volume data for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalBarParams {
    /// List of stock symbols to retrieve bar data for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Time frame for the bars, e.g., "1Min", "5Min", "1Hour", "1Day".
    timeframe: String,

    /// Start time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,

    /// End time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,

    /// Maximum number of bars to return.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u16>,

    /// Type of adjustment to apply to the data (e.g., "raw", "split", "dividend", "all").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    adjustment: Option<String>,

    /// Query for data as of this date (for historical snapshots).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,

    /// Token for pagination to get the next page of results.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,

    /// Sort order for results (e.g., "asc", "desc").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}
/// Response from the historical bars API endpoint.
///
/// Contains OHLC (Open, High, Low, Close) bar data for requested symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarResponse {
    /// Map of symbol to a vector of price bars.
    /// Each symbol has a list of bars representing price action over time.
    bars: HashMap<String, Vec<Bars>>,

    /// Token for pagination to get the next page of results.
    next_page_token: String,

    /// Currency used for the price values (e.g., "USD").
    currency: Option<String>,
}

/// Represents a single OHLC (Open, High, Low, Close) price bar.
///
/// Contains price and volume data for a specific time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bars {
    /// Timestamp in RFC-3339 format representing the start of the bar period.
    #[serde(rename = "t")]
    pub timestamp: String,

    /// Opening price for the period.
    #[serde(rename = "o")]
    pub open: f64,

    /// Highest price reached during the period.
    #[serde(rename = "h")]
    pub high: f64,

    /// Lowest price reached during the period.
    #[serde(rename = "l")]
    pub low: f64,

    /// Closing price for the period.
    #[serde(rename = "c")]
    pub close: f64,

    /// Total trading volume during the period.
    #[serde(rename = "v")]
    pub volume: i64,

    /// Number of trades executed during the period.
    #[serde(rename = "n")]
    pub count: i64,

    /// Volume-weighted average price (VWAP) for the period.
    #[serde(rename = "vw")]
    pub volume_weighted_average: f64,
}

/// Methods for accessing and manipulating bar data.
impl BarResponse {
    /* =========================
    Basic access / metadata
    ========================= */

    /// List all symbols present in the response.
    ///
    /// # Returns
    /// * An iterator over all symbol strings in the response
    pub fn symbols(&self) -> impl Iterator<Item = &str> {
        self.bars.keys().map(|s| s.as_str())
    }

    /// Borrow bars for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve bars for
    ///
    /// # Returns
    /// * A slice of bars if the symbol exists, None otherwise
    pub fn bars_for(&self, symbol: &str) -> Option<&[Bars]> {
        self.bars.get(symbol).map(|v| v.as_slice())
    }

    /// Get mutable access to bars for a symbol (for transforming/sorting).
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve mutable bars for
    ///
    /// # Returns
    /// * A mutable reference to the vector of bars if the symbol exists, None otherwise
    pub fn bars_for_mut(&mut self, symbol: &str) -> Option<&mut Vec<Bars>> {
        self.bars.get_mut(symbol)
    }

    /// Get the total number of bars across all symbols.
    ///
    /// # Returns
    /// * The total count of bars in the response
    pub fn len_total(&self) -> usize {
        self.bars.values().map(|v| v.len()).sum()
    }

    /// Check if there are no bars for any symbol.
    ///
    /// # Returns
    /// * `true` if there are no bars for any symbol, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.bars.values().all(|v| v.is_empty())
    }

    /// Get the next page token, treating empty string as "no more pages".
    ///
    /// # Returns
    /// * The next page token if it exists and is not empty, None otherwise
    pub fn next_page_token(&self) -> Option<&str> {
        if self.next_page_token.is_empty() {
            None
        } else {
            Some(self.next_page_token.as_str())
        }
    }

    /// Get the currency used for the price values.
    ///
    /// # Returns
    /// * The currency code (e.g., "USD") if available, None otherwise
    pub fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }

    /* =========================
    Per-symbol convenience
    ========================= */

    /// Get the first bar for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the first bar for
    ///
    /// # Returns
    /// * The first bar if the symbol exists and has bars, None otherwise
    pub fn first_bar(&self, symbol: &str) -> Option<&Bars> {
        self.bars.get(symbol).and_then(|v| v.first())
    }

    /// Get the last bar for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the last bar for
    ///
    /// # Returns
    /// * The last bar if the symbol exists and has bars, None otherwise
    pub fn last_bar(&self, symbol: &str) -> Option<&Bars> {
        self.bars.get(symbol).and_then(|v| v.last())
    }

    /// Get all closing prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve closing prices for
    ///
    /// # Returns
    /// * A vector of closing prices for the symbol, empty if symbol doesn't exist
    pub fn closing_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.close).collect())
            .unwrap_or_default()
    }

    /// Get all opening prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve opening prices for
    ///
    /// # Returns
    /// * A vector of opening prices for the symbol, empty if symbol doesn't exist
    pub fn opening_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.open).collect())
            .unwrap_or_default()
    }

    /// Get all high prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve high prices for
    ///
    /// # Returns
    /// * A vector of high prices for the symbol, empty if symbol doesn't exist
    pub fn high_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.high).collect())
            .unwrap_or_default()
    }

    /// Get all low prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve low prices for
    ///
    /// # Returns
    /// * A vector of low prices for the symbol, empty if symbol doesn't exist
    pub fn low_prices(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.low).collect())
            .unwrap_or_default()
    }

    /// Get all volume values for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve volumes for
    ///
    /// # Returns
    /// * A vector of volume values for the symbol, empty if symbol doesn't exist
    pub fn volumes(&self, symbol: &str) -> Vec<i64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.volume).collect())
            .unwrap_or_default()
    }

    /// Get all trade count values for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve trade counts for
    ///
    /// # Returns
    /// * A vector of trade count values for the symbol, empty if symbol doesn't exist
    pub fn counts(&self, symbol: &str) -> Vec<i64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.count).collect())
            .unwrap_or_default()
    }

    /// Get all volume-weighted average price (VWAP) values for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve VWAP values for
    ///
    /// # Returns
    /// * A vector of VWAP values for the symbol, empty if symbol doesn't exist
    pub fn vwap_values(&self, symbol: &str) -> Vec<f64> {
        self.bars
            .get(symbol)
            .map(|v| v.iter().map(|b| b.volume_weighted_average).collect())
            .unwrap_or_default()
    }

    /// Calculate the average closing price for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to calculate average close for
    ///
    /// # Returns
    /// * The average closing price if the symbol exists and has bars, None otherwise
    pub fn avg_close(&self, symbol: &str) -> Option<f64> {
        let v = self.bars.get(symbol)?;
        if v.is_empty() {
            return None;
        }
        Some(v.iter().map(|b| b.close).sum::<f64>() / v.len() as f64)
    }

    /// Find the maximum high price for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to find maximum high price for
    ///
    /// # Returns
    /// * The maximum high price if the symbol exists and has bars, None otherwise
    pub fn max_high(&self, symbol: &str) -> Option<f64> {
        self.bars
            .get(symbol)?
            .iter()
            .map(|b| b.high)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Find the minimum low price for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to find minimum low price for
    ///
    /// # Returns
    /// * The minimum low price if the symbol exists and has bars, None otherwise
    pub fn min_low(&self, symbol: &str) -> Option<f64> {
        self.bars
            .get(symbol)?
            .iter()
            .map(|b| b.low)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Calculate the total trading volume for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to calculate total volume for
    ///
    /// # Returns
    /// * The total volume if the symbol exists and has bars, None otherwise
    pub fn total_volume(&self, symbol: &str) -> Option<i64> {
        Some(self.bars.get(symbol)?.iter().map(|b| b.volume).sum())
    }

    /* =========================
    Cross-symbol utilities
    ========================= */

    /// Flatten an iterator over all bars for all symbols.
    ///
    /// This method provides a convenient way to iterate through all bars
    /// across all symbols, with each item containing both the symbol and the bar.
    ///
    /// # Returns
    /// * An iterator yielding tuples of (symbol, bar reference)
    pub fn iter_all(&self) -> impl Iterator<Item = (&str, &Bars)> {
        self.bars
            .iter()
            .flat_map(|(sym, v)| v.iter().map(move |b| (sym.as_str(), b)))
    }

    /// Find the maximum high price across all symbols and return the symbol and price.
    ///
    /// # Returns
    /// * A tuple containing the symbol and its maximum high price, or None if there are no bars
    pub fn max_high_all(&self) -> Option<(&str, f64)> {
        self.iter_all()
            .max_by(|(_, a), (_, b)| a.high.partial_cmp(&b.high).unwrap())
            .map(|(s, b)| (s, b.high))
    }

    /// Find the minimum low price across all symbols and return the symbol and price.
    ///
    /// # Returns
    /// * A tuple containing the symbol and its minimum low price, or None if there are no bars
    pub fn min_low_all(&self) -> Option<(&str, f64)> {
        self.iter_all()
            .min_by(|(_, a), (_, b)| a.low.partial_cmp(&b.low).unwrap())
            .map(|(s, b)| (s, b.low))
    }

    /// Calculate the total trading volume across all symbols.
    ///
    /// # Returns
    /// * The sum of all volume values across all bars for all symbols
    pub fn total_volume_all(&self) -> i64 {
        self.bars.values().flatten().map(|b| b.volume).sum()
    }
}

/// Retrieves historical price bars (OHLC) data from the Alpaca API.
///
/// This function fetches historical price bars for specified stock symbols,
/// with configurable timeframes (e.g., 1Min, 5Min, 1Hour, 1Day).
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `params` - Parameters for the historical bars request
///
/// # Returns
/// * `Result<BarResponse, Box<dyn std::error::Error>>` - The bar data or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let params = HistoricalBarParams::builder()
///     .symbols(vec!["AAPL".to_string()])
///     .timeframe("1Day".to_string())
///     .start("2024-01-01T00:00:00Z".to_string())
///     .end("2024-01-31T00:00:00Z".to_string())
///     .build();
/// let bars = get_historical_bars(&alpaca, params).await?;
///
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
        Err(e) => panic!("Error getting historical bars: {e}"),
    }
}
/// Parameters for retrieving the latest price bars from the Alpaca API.
///
/// This struct is used to build requests for the most recent price bars
/// for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct LatestBarsParams {
    /// List of stock symbols to retrieve the latest bars for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

/// Response from the latest bars API endpoint.
///
/// Contains the most recent OHLC (Open, High, Low, Close) bar data for requested symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestBarsResponse {
    /// Map of symbol to its most recent price bar.
    /// Each symbol has exactly one bar representing the latest price action.
    pub bars: HashMap<String, Bars>,

    /// Token for pagination to get the next page of results.
    /// Usually absent for "latest" endpoints.
    #[serde(default)]
    pub next_page_token: Option<String>,

    /// Currency used for the price values (e.g., "USD").
    #[serde(default)]
    pub currency: Option<String>,
}

/// Helper methods for accessing latest bars data.
impl LatestBarsResponse {
    /// Get the latest bar for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the bar for
    ///
    /// # Returns
    /// * The latest bar if the symbol exists, None otherwise
    pub fn bar(&self, symbol: &str) -> Option<&Bars> {
        self.bars.get(symbol)
    }

    /// Get all symbols present in the response.
    ///
    /// # Returns
    /// * An iterator over all symbol strings in the response
    pub fn symbols(&self) -> impl Iterator<Item = &str> {
        self.bars.keys().map(|s| s.as_str())
    }

    /// Get the next page token, filtering out empty strings.
    ///
    /// # Returns
    /// * The next page token if it exists and is not empty, None otherwise
    pub fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref().filter(|s| !s.is_empty())
    }

    /// Get the currency used for the price values.
    ///
    /// # Returns
    /// * The currency code (e.g., "USD") if available, None otherwise
    pub fn currency(&self) -> Option<&str> {
        self.currency.as_deref()
    }
}

/// Retrieves the latest price bars for specified stock symbols from the Alpaca API.
///
/// This function fetches the most recent OHLC (Open, High, Low, Close) bar
/// for each of the specified stock symbols.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `params` - Parameters for the latest bars request
///
/// # Returns
/// * `Result<LatestBarsResponse, Box<dyn std::error::Error>>` - The latest bar data or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let params = LatestBarsParams::builder()
///     .symbols(vec!["AAPL".to_string(), "MSFT".to_string()])
///     .feed("iex".to_string())
///     .build();
/// let latest_bars = get_latest_bars(&alpaca, params).await?;
///
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
        Err(e) => panic!("Error getting latest bar: {e}"),
    }
}

/// Response containing trade condition codes and their descriptions.
///
/// This struct maps single character condition codes to their human-readable descriptions.
/// Trade condition codes are used to indicate special circumstances for a trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TradeConditionResponse(pub HashMap<char, String>);

impl TradeConditionResponse {
    /// Look up a condition description by its single character code.
    ///
    /// # Arguments
    /// * `code` - The single character condition code (e.g., ' ', '4', 'B', 'T')
    ///
    /// # Returns
    /// * The human-readable description if the code exists, None otherwise
    pub fn describe(&self, code: char) -> Option<&str> {
        self.0.get(&code).map(|s| s.as_str())
    }

    /// Look up a condition description by a string code, taking only the first character.
    ///
    /// This is a convenience method that accepts a string like "B" or "4" and
    /// uses only the first character for the lookup.
    ///
    /// # Arguments
    /// * `code` - A string containing the condition code (only first char is used)
    ///
    /// # Returns
    /// * The human-readable description if the code exists, None otherwise
    pub fn describe_str(&self, code: &str) -> Option<&str> {
        code.chars().next().and_then(|c| self.describe(c))
    }
}
/// Query parameters for condition codes request.
///
/// Used to specify which tape (exchange group) to retrieve condition codes for.
#[derive(Serialize)]
struct CondQuery<'a> {
    /// The tape code (e.g., "A", "B", "C") representing an exchange group.
    tape: &'a str,
}

/// Retrieves trade condition codes and their descriptions from the Alpaca API.
///
/// Trade condition codes are single characters that indicate special circumstances
/// for a trade or quote. This function fetches the mapping of these codes to
/// their human-readable descriptions.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `ticktype` - The type of tick data ("trade" or "quote")
/// * `tape` - The tape code (e.g., "A", "B", "C") representing an exchange group
///
/// # Returns
/// * `Result<TradeConditionResponse, Box<dyn std::error::Error>>` - The condition codes or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let conditions = get_condition_codes(&alpaca, "trade", "A").await?;
/// println!("Code '4' means: {}", conditions.describe('4').unwrap_or("Unknown"));
///
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

/// Response containing exchange codes and their descriptions.
///
/// This struct maps single character exchange codes to their human-readable descriptions.
/// Exchange codes identify different stock exchanges and trading venues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExchangeCodesResponse(pub HashMap<char, String>);

impl ExchangeCodesResponse {
    /// Look up an exchange description by its single character code.
    ///
    /// # Arguments
    /// * `code` - The single character exchange code (e.g., 'A', 'P', 'Q')
    ///
    /// # Returns
    /// * The human-readable exchange description if the code exists, None otherwise
    pub fn describe(&self, code: char) -> Option<&str> {
        self.0.get(&code).map(|s| s.as_str())
    }

    /// Look up an exchange description by a string code, taking only the first character.
    ///
    /// This is a convenience method that accepts a string like "A" or "Q" and
    /// uses only the first character for the lookup.
    ///
    /// # Arguments
    /// * `code` - A string containing the exchange code (only first char is used)
    ///
    /// # Returns
    /// * The human-readable exchange description if the code exists, None otherwise
    pub fn describe_str(&self, code: &str) -> Option<&str> {
        code.chars().next().and_then(|c| self.describe(c))
    }
}

/// Retrieves exchange codes and their descriptions from the Alpaca API.
///
/// Exchange codes are single characters that identify different stock exchanges
/// and trading venues. This function fetches the mapping of these codes to
/// their human-readable descriptions.
///
/// Note: There's a typo in the function name ("exchance" instead of "exchange"),
/// but it's kept for backward compatibility.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
///
/// # Returns
/// * `Result<ExchangeCodesResponse, Box<dyn std::error::Error>>` - The exchange codes or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let exchanges = get_exchance_codes(&alpaca).await?;
/// println!("Exchange 'A' is: {}", exchanges.describe('A').unwrap_or("Unknown"));
///
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
/// Parameters for retrieving historical quotes data from the Alpaca API.
///
/// This struct is used to build requests for historical bid/ask quotes
/// for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalQuotesParams {
    /// List of stock symbols to retrieve quote data for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Start time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,

    /// End time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,

    /// Maximum number of quotes to return.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,

    /// Query for data as of this date (for historical snapshots).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,

    /// Token for pagination to get the next page of results.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,

    /// Sort order for results (e.g., "asc", "desc").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}

/// Response from the historical quotes API endpoint.
///
/// Contains bid/ask quote data for requested symbols, organized by symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalQuotes {
    /// Map of symbol to a vector of quotes.
    /// Each symbol has a list of quotes representing the bid/ask data over time.
    pub quotes: HashMap<String, Vec<Quotes>>,

    /// Currency used for the price values (e.g., "USD").
    #[serde(default)]
    pub currency: Option<String>,

    /// Token for pagination to get the next page of results.
    pub next_page_token: Option<String>,
}

/// Represents a single bid/ask quote.
///
/// Contains information about the best bid and ask prices, sizes, and exchanges
/// at a specific point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotes {
    /// Timestamp in RFC-3339 format when the quote was recorded.
    #[serde(rename = "t")]
    timestamp: String,

    /// Exchange code for the best bid.
    #[serde(rename = "bx")]
    bid_exchange: String,

    /// Best bid price.
    #[serde(rename = "bp")]
    bid_price: f64,

    /// Size of the best bid (number of shares).
    #[serde(rename = "bs")]
    bid_size: u64,

    /// Exchange code for the best ask.
    #[serde(rename = "ax")]
    ask_exchange: String,

    /// Best ask price.
    #[serde(rename = "ap")]
    ask_price: f64,

    /// Size of the best ask (number of shares).
    #[serde(rename = "as")]
    ask_size: u64,

    /// Condition flags for the quote.
    #[serde(rename = "c")]
    condition_flags: Vec<String>,

    /// Exchange code where the quote was recorded.
    #[serde(rename = "z")]
    exchange: String,
}
/// Methods for accessing and manipulating historical quotes data.
impl HistoricalQuotes {
    /// Get all quotes for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve quotes for
    ///
    /// # Returns
    /// * A vector of quotes if the symbol exists, None otherwise
    pub fn get_symbol_quotes(&self, symbol: &str) -> Option<&Vec<Quotes>> {
        self.quotes.get(symbol)
    }

    /// Get all symbols present in the response.
    ///
    /// # Returns
    /// * A vector of all symbols in the response
    pub fn symbols(&self) -> Vec<&String> {
        self.quotes.keys().collect()
    }

    /// Check if the response contains data for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to check
    ///
    /// # Returns
    /// * `true` if the symbol exists in the response, `false` otherwise
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.quotes.contains_key(symbol)
    }

    /// Get the most recent quote for a symbol (by last element in Vec).
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the latest quote for
    ///
    /// # Returns
    /// * The most recent quote if the symbol exists and has quotes, None otherwise
    pub fn get_last_quote(&self, symbol: &str) -> Option<&Quotes> {
        self.quotes.get(symbol)?.last()
    }

    /// Get all bid prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve bid prices for
    ///
    /// # Returns
    /// * A vector of bid prices for the symbol, empty if symbol doesn't exist
    pub fn get_bid_prices(&self, symbol: &str) -> Vec<f64> {
        self.quotes
            .get(symbol)
            .map(|qs| qs.iter().map(|q| q.bid_price).collect())
            .unwrap_or_default()
    }

    /// Get all ask prices for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve ask prices for
    ///
    /// # Returns
    /// * A vector of ask prices for the symbol, empty if symbol doesn't exist
    pub fn get_ask_prices(&self, symbol: &str) -> Vec<f64> {
        self.quotes
            .get(symbol)
            .map(|qs| qs.iter().map(|q| q.ask_price).collect())
            .unwrap_or_default()
    }

    /// Get all timestamps for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve timestamps for
    ///
    /// # Returns
    /// * A vector of timestamp strings for the symbol, empty if symbol doesn't exist
    pub fn get_timestamps(&self, symbol: &str) -> Vec<&str> {
        self.quotes
            .get(symbol)
            .map(|qs| qs.iter().map(|q| q.timestamp.as_str()).collect())
            .unwrap_or_default()
    }

    /// Check if there's another page of data available.
    ///
    /// # Returns
    /// * `true` if there's a non-empty next page token, `false` otherwise
    pub fn has_next_page(&self) -> bool {
        self.next_page_token
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }
}

/// Retrieves historical quote data from the Alpaca API.
///
/// This function fetches historical bid/ask quotes for specified stock symbols,
/// providing insight into the market's order book over time.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `params` - Parameters for the historical quotes request
///
/// # Returns
/// * `Result<HistoricalQuotes, Box<dyn std::error::Error>>` - The quote data or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let params = HistoricalQuotesParams::builder()
///     .symbols(vec!["AAPL".to_string()])
///     .start("2024-01-03T00:00:00Z".to_string())
///     .end("2024-01-03T01:00:00Z".to_string())
///     .limit(100)
///     .build();
/// let quotes = get_historical_quotes(&alpaca, params).await?;
///
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
        Err(e) => panic!("Error getting historical quotes: {e}"),
    }
}

/// Parameters for retrieving the latest quotes from the Alpaca API.
///
/// This struct is used to build requests for the most recent bid/ask quotes
/// for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct LatestQuotesParams {
    /// List of stock symbols to retrieve the latest quotes for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

/// Response from the latest quotes API endpoint.
///
/// Contains the most recent bid/ask quote data for requested symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestQuotes {
    /// Map of symbol to its most recent quote.
    /// Each symbol has exactly one quote representing the latest bid/ask data.
    pub quotes: HashMap<String, Quotes>,

    /// Currency used for the price values (e.g., "USD").
    #[serde(default)]
    pub currency: Option<String>,
}

/// Helper methods for accessing latest quotes data.
impl LatestQuotes {
    /// Get the latest quote for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the quote for
    ///
    /// # Returns
    /// * The latest quote if the symbol exists, None otherwise
    pub fn get_symbol_quote(&self, symbol: &str) -> Option<&Quotes> {
        self.quotes.get(symbol)
    }

    /// Get all symbols present in the response.
    ///
    /// # Returns
    /// * A vector of all symbols in the response
    pub fn symbols(&self) -> Vec<&String> {
        self.quotes.keys().collect()
    }

    /// Check if the response contains data for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to check
    ///
    /// # Returns
    /// * `true` if the symbol exists in the response, `false` otherwise
    pub fn has_symbol(&self, symbol: &str) -> bool {
        self.quotes.contains_key(symbol)
    }

    /// Get the latest quote for a symbol (alias for get_symbol_quote).
    ///
    /// This method is kept for API compatibility with HistoricalQuotes.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the quote for
    ///
    /// # Returns
    /// * The latest quote if the symbol exists, None otherwise
    pub fn get_last_quote(&self, symbol: &str) -> Option<&Quotes> {
        // kept for API compatibility; same as get_symbol_quote now
        self.quotes.get(symbol)
    }

    /// Get the bid price for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the bid price for
    ///
    /// # Returns
    /// * The bid price if the symbol exists, None otherwise
    pub fn get_bid_price(&self, symbol: &str) -> Option<f64> {
        self.quotes.get(symbol).map(|q| q.bid_price)
    }

    /// Get the ask price for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the ask price for
    ///
    /// # Returns
    /// * The ask price if the symbol exists, None otherwise
    pub fn get_ask_price(&self, symbol: &str) -> Option<f64> {
        self.quotes.get(symbol).map(|q| q.ask_price)
    }

    /// Get the timestamp for a symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the timestamp for
    ///
    /// # Returns
    /// * The timestamp string if the symbol exists, None otherwise
    pub fn get_timestamp(&self, symbol: &str) -> Option<&str> {
        self.quotes.get(symbol).map(|q| q.timestamp.as_str())
    }
}

/// Retrieves the latest quotes for specified stock symbols from the Alpaca API.
///
/// This function fetches the most recent bid/ask quotes for each of the specified
/// stock symbols, providing the current state of the market's order book.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `params` - Parameters for the latest quotes request
///
/// # Returns
/// * `Result<LatestQuotes, Box<dyn std::error::Error>>` - The latest quote data or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let params = LatestQuotesParams::builder()
///     .symbols(vec!["AAPL".to_string(), "MSFT".to_string()])
///     .feed("iex".to_string())
///     .build();
/// let latest_quotes = get_latest_quotes(&alpaca, params).await?;
///
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

/// Parameters for retrieving historical trades data from the Alpaca API.
///
/// This struct is used to build requests for historical executed trades
/// for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct HistoricalTradesParams {
    /// List of stock symbols to retrieve trade data for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Start time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<String>,

    /// End time for the data query in ISO 8601 format.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,

    /// Maximum number of trades to return.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,

    /// Query for data as of this date (for historical snapshots).
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    asof: Option<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,

    /// Token for pagination to get the next page of results.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,

    /// Sort order for results (e.g., "asc", "desc").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}

/// Response from the historical trades API endpoint.
///
/// Contains executed trade data for requested symbols, organized by symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTrades {
    /// Map of symbol to a vector of trades.
    /// Each symbol has a list of trades representing executed transactions over time.
    pub trades: HashMap<String, Vec<Trades>>,

    /// Currency used for the price values (e.g., "USD").
    #[serde(default)]
    pub currency: Option<String>,

    /// Token for pagination to get the next page of results.
    pub next_page_token: Option<String>,
}
/// Methods for accessing and manipulating historical trades data.
impl HistoricalTrades {
    /// Get all trades for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve trades for
    ///
    /// # Returns
    /// * A vector of trades if the symbol exists, None otherwise
    pub fn trades_for_symbol(&self, symbol: &str) -> Option<&Vec<Trades>> {
        self.trades.get(symbol)
    }

    /// Get the first (earliest) trade for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the first trade for
    ///
    /// # Returns
    /// * The first trade if the symbol exists and has trades, None otherwise
    pub fn first_trade(&self, symbol: &str) -> Option<&Trades> {
        self.trades.get(symbol)?.first()
    }

    /// Get the last (most recent) trade for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the last trade for
    ///
    /// # Returns
    /// * The last trade if the symbol exists and has trades, None otherwise
    pub fn last_trade(&self, symbol: &str) -> Option<&Trades> {
        self.trades.get(symbol)?.last()
    }

    /// Flatten all trades into a single vector with symbol references.
    ///
    /// This method provides a convenient way to iterate through all trades
    /// across all symbols, with each item containing both the symbol and the trade.
    ///
    /// # Returns
    /// * A vector of (symbol, trade) tuples containing all trades
    pub fn all_trades(&self) -> Vec<(&String, &Trades)> {
        self.trades
            .iter()
            .flat_map(|(sym, trades)| trades.iter().map(move |t| (sym, t)))
            .collect()
    }

    /// Count the total number of trades across all symbols.
    ///
    /// # Returns
    /// * The sum of trade counts for all symbols
    pub fn total_trade_count(&self) -> usize {
        self.trades.values().map(|v| v.len()).sum()
    }

    /// Get a map of symbol to number of trades.
    ///
    /// # Returns
    /// * A HashMap mapping each symbol to its trade count
    pub fn counts_per_symbol(&self) -> HashMap<&String, usize> {
        self.trades
            .iter()
            .map(|(sym, trades)| (sym, trades.len()))
            .collect()
    }
}
/// Represents a single executed trade.
///
/// Contains information about a specific trade transaction including
/// price, size, exchange, and condition flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trades {
    /// Timestamp in RFC-3339 format when the trade was executed.
    #[serde(rename = "t")]
    timestamp: String,

    /// Exchange where the trade was executed.
    #[serde(rename = "x")]
    exchange: String,

    /// Price at which the trade was executed.
    #[serde(rename = "p")]
    price: f64,

    /// Size of the trade (number of shares).
    #[serde(rename = "s")]
    size: u64,

    /// Unique identifier for the trade.
    #[serde(rename = "i")]
    trade_id: u64,

    /// Condition flags indicating special circumstances for the trade.
    #[serde(rename = "c")]
    condition_flags: Vec<String>,

    /// Exchange code where the trade was executed.
    #[serde(rename = "z")]
    exchange_code: String,

    /// Optional update timestamp if the trade was updated.
    #[serde(rename = "u")]
    #[serde(default)]
    update: Option<String>,
}

/// Retrieves historical trade data from the Alpaca API.
///
/// This function fetches historical executed trades for specified stock symbols,
/// providing insight into actual market transactions over time.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `params` - Parameters for the historical trades request
///
/// # Returns
/// * `Result<HistoricalTrades, Box<dyn std::error::Error>>` - The trade data or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let params = HistoricalTradesParams::builder()
///     .symbols(vec!["AAPL".to_string()])
///     .start("2024-01-03T00:00:00Z".to_string())
///     .end("2024-01-03T01:00:00Z".to_string())
///     .limit(100)
///     .build();
/// let trades = get_historical_trades(&alpaca, params).await?;
///
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

/// Parameters for retrieving the latest trades from the Alpaca API.
///
/// This struct is used to build requests for the most recent executed trades
/// for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct LatestTradesParams {
    /// List of stock symbols to retrieve the latest trades for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

/// Response from the latest trades API endpoint.
///
/// Contains the most recent executed trade data for requested symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestTrades {
    /// Map of symbol to its most recent trade.
    /// Each symbol has exactly one trade representing the latest executed transaction.
    pub trades: HashMap<String, Trades>,

    /// Currency used for the price values (e.g., "USD").
    #[serde(default)]
    pub currency: Option<String>,
}

/// Helper methods for accessing latest trades data.
impl LatestTrades {
    /// Get the latest trade for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The stock symbol to retrieve the trade for
    ///
    /// # Returns
    /// * The latest trade if the symbol exists, None otherwise
    pub fn trade_for_symbol(&self, symbol: &str) -> Option<&Trades> {
        self.trades.get(symbol)
    }

    /// Flatten all trades into a single vector with symbol references.
    ///
    /// This method provides a convenient way to iterate through all trades
    /// across all symbols, with each item containing both the symbol and the trade.
    ///
    /// # Returns
    /// * A vector of (symbol, trade) tuples containing all trades
    pub fn all_trades(&self) -> Vec<(&String, &Trades)> {
        self.trades.iter().collect()
    }

    /// Count the total number of trades (will equal number of symbols).
    ///
    /// Since this is a "latest trades" response, each symbol has exactly one trade.
    ///
    /// # Returns
    /// * The number of symbols/trades in the response
    pub fn total_trade_count(&self) -> usize {
        self.trades.len()
    }

    /// Get a map of symbol to trade count (always 1 for each symbol).
    ///
    /// This method is kept for API compatibility with HistoricalTrades.
    ///
    /// # Returns
    /// * A HashMap mapping each symbol to 1 (the count of trades per symbol)
    pub fn counts_per_symbol(&self) -> HashMap<&String, usize> {
        self.trades.keys().map(|sym| (sym, 1)).collect()
    }
}

/// Retrieves the latest trades for specified stock symbols from the Alpaca API.
///
/// This function fetches the most recent executed trade for each of the specified
/// stock symbols, providing the current state of market transactions.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication
/// * `params` - Parameters for the latest trades request
///
/// # Returns
/// * `Result<LatestTrades, Box<dyn std::error::Error>>` - The latest trade data or an error
///
/// # Examples
///
/// let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
/// let params = LatestTradesParams::builder()
///     .symbols(vec!["AAPL".to_string(), "MSFT".to_string()])
///     .feed("iex".to_string())
///     .build();
/// let latest_trades = get_latest_trades(&alpaca, params).await?;
///
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

/// Parameters for retrieving market snapshots from the Alpaca API.
///
/// This struct is used to build requests for comprehensive market snapshots
/// that include bars, quotes, and trades for specified stock symbols.
#[derive(Debug, TypedBuilder, Serialize)]
pub struct SnapshotsParams {
    /// List of stock symbols to retrieve snapshots for.
    /// Will be serialized as a comma-separated string.
    #[serde(serialize_with = "serialize_symbols")]
    symbols: Vec<String>,

    /// Data feed to use (e.g., "sip", "iex").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<String>,

    /// Currency to use for the data (e.g., "USD").
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

/// Response from the snapshots API endpoint.
///
/// Contains comprehensive market data for requested symbols, including
/// bars, quotes, and trades in a single response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResponse(pub HashMap<String, StockData>);

/// Comprehensive market data for a single stock symbol.
///
/// Contains various data points including daily and minute bars,
/// latest quote and trade information, and previous day's data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    /// The current day's OHLC bar data.
    pub daily_bar: Bars,

    /// The latest bid/ask quote information.
    pub latest_quote: Quotes,

    /// The latest executed trade.
    pub latest_trade: Trades,

    /// The most recent 1-minute OHLC bar.
    pub minute_bar: Bars,

    /// The previous day's OHLC bar data.
    pub prev_daily_bar: Bars,
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
        self.latest_trade.price
    }

    /// Get the spread between bid and ask
    pub fn spread(&self) -> f64 {
        self.latest_quote.ask_price - self.latest_quote.bid_price
    }

    /// Get daily OHLC data as tuple
    pub fn daily_ohlc(&self) -> (f64, f64, f64, f64) {
        (
            self.daily_bar.open,
            self.daily_bar.high,
            self.daily_bar.low,
            self.daily_bar.close,
        )
    }

    /// Check if price is above previous daily close
    pub fn is_above_prev_close(&self) -> bool {
        self.latest_trade.price > self.prev_daily_bar.close
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

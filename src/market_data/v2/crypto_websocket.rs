//! Represents an error message structure typically used for structured error handling or reporting.
//!
//! This struct is used to encapsulate a generic error message and
//! an associated optional error code. Both fields are optional to provide flexibility in error representation.
//!
//! # Fields
//! - `msg`:
//!   An optional string containing a human-readable error message. For example,
//!   this could describe the specific error encountered, such as "Invalid API Key".
//!
//! - `code`:
//!   An optional integer representing an error status code. This can be useful
//!   for programmatically identifying the type of error in applications.
//!
//! # Derives
//! - `Debug`: Enables the struct to be formatted and displayed using the `{:?}` formatter,
//!   ideal for debugging purposes.
//! - `Deserialize`: Facilitates the deserialization of this struct from serialized formats, such as JSON.
//! - `Clone`: Allows this struct to be cloned, providing the ability to easily create duplicate instances of the error.
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::Utf8Bytes;
use typed_builder::TypedBuilder;
use crate::auth::{Alpaca, TradingType};

/// An enumeration `NumF64` that represents a number which can be one of three types:
/// - `i64`: A signed 64-bit integer.
/// - `f64`: A 64-bit floating-point number.
/// - `String`: A textual representation.
///
/// This enum is:
/// - Derived with `Deserialize` from Serde, allowing it to be deserialized from different formats (e.g., JSON).
/// - Clone-able to easily create duplicates of the value.
/// - Debuggable to enable debugging using the `Debug` formatter.
///
/// Additionally, `#[serde(untagged)]` indicates that Serde will infer the variant to deserialize
/// based on the input type, rather than requiring a tag to distinguish between variants.
///
/// ## Example
///
/// ```rust
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Debug)]
/// #[serde(untagged)]
/// pub enum NumF64 {
///     I(i64),
///     F(f64),
///     S(String),
/// }
///
/// let json_integer = "42";
/// let parsed: NumF64 = serde_json::from_str(json_integer).unwrap();
/// assert!(matches!(parsed, NumF64::I(42)));
///
/// let json_float = "42.42";
/// let parsed: NumF64 = serde_json::from_str(json_float).unwrap();
/// assert!(matches!(parsed, NumF64::F(42.42)));
///
/// let json_string = "\"42\"";
/// let parsed: NumF64 = serde_json::from_str(json_string).unwrap();
/// assert!(matches!(parsed, NumF64::S(ref s) if s == "42"));
/// ```
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum NumF64 { I(i64), F(f64), S(String) }

impl From<NumF64> for f64 {
    fn from(n: NumF64) -> Self {
        match n {
            NumF64::I(i) => i as f64,
            NumF64::F(f) => f,
            NumF64::S(s) => s.parse::<f64>().unwrap_or(0.0),
        }
    }
}

/// The `Subscribe` struct is used to manage subscription requests for different types of market data.
/// Each field represents a subscription group, allowing customization of which data streams to subscribe to.
///
/// # Fields
///
/// * `trades` - A vector of strings representing the trade symbols to subscribe to.
///   If empty, this field will be skipped during serialization.
///
/// * `quotes` - A vector of strings representing the quote symbols to subscribe to.
///   If empty, this field will be skipped during serialization.
///
/// * `bars` - A vector of strings representing the bar (candlestick) symbols to subscribe to.
///   If empty, this field will be skipped during serialization.
///
/// * `daily_bars` (serialized as `dailyBars`) - A vector of strings representing
///   the symbols for daily bar data (e.g., daily candlesticks) to subscribe to.
///   If empty, this field will be skipped during serialization.
///
/// * `updated_bars` (serialized as `updatedBars`) - A vector of strings representing
///   the symbols for updated bar data to subscribe to.
///   If empty, this field will be skipped during serialization.
///
/// * `orderbooks` - A vector of strings representing the symbols for order book data to subscribe to.
///   If empty, this field will be skipped during serialization.
///
/// # Traits
///
/// * `Debug` - Enables formatting of the `Subscribe` struct for debugging purposes.
/// * `Default` - Provides a default implementation where all subscription vectors are empty.
/// * `Clone` - Enables cloning of the `Subscribe` struct.
/// * `Serialize` - Allows serialization of the struct into formats like JSON, with custom rules applied
///   (e.g., skipping empty fields and renaming some fields).
#[derive(Debug, Default, Clone, Serialize)]
pub struct Subscribe {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trades: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quotes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bars: Vec<String>,
    #[serde(rename = "dailyBars", skip_serializing_if = "Vec::is_empty")]
    pub daily_bars: Vec<String>,
    #[serde(rename = "updatedBars", skip_serializing_if = "Vec::is_empty")]
    pub updated_bars: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub orderbooks: Vec<String>,
}

impl Subscribe {
    /// Creates a new instance of the struct, initializing it with default values.
    ///
    /// This function is a shorthand for calling the `default` implementation of the struct.
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of the struct with default settings.
    pub fn new() -> Self {
        Self::default()
    }
    /// Constructs a JSON object representing a subscription action for various data streams.
    ///
    /// This method serializes the subscription configuration into a JSON object using `serde_json`.
    /// The generated JSON object includes the following key-value pairs:
    /// - `"action"`: A string indicating the action type, which is always `"subscribe"`.
    /// - `"trades"`: The value of `self.trades`, representing the trade data stream subscription.
    /// - `"quotes"`: The value of `self.quotes`, representing the quote data stream subscription.
    /// - `"bars"`: The value of `self.bars`, representing the bars data stream subscription.
    /// - `"dailyBars"`: The value of `self.daily_bars`, representing the subscription for daily bar data.
    /// - `"updatedBars"`: The value of `self.updated_bars`, representing the updated bar data stream subscription.
    /// - `"orderbooks"`: The value of `self.orderbooks`, representing the order book data stream subscription.
    ///
    /// # Returns
    /// A `serde_json::Value` containing the JSON representation of the subscription action.
    ///
    pub fn action_json(&self) -> serde_json::Value {
        serde_json::json!({
            "action": "subscribe",
            "trades": self.trades,
            "quotes": self.quotes,
            "bars": self.bars,
            "dailyBars": self.daily_bars,
            "updatedBars": self.updated_bars,
            "orderbooks": self.orderbooks,
        })
    }
}

/// `SubscriptionAck` is a structure representing the acknowledgment of a subscription to various types of market data streams.
/// It contains information about the data streams successfully acknowledged by the server.
///
/// # Fields
///
/// * `trades` - A vector of strings representing the symbols for which trade-related data is acknowledged as subscribed.
///   Defaults to an empty vector if not specified.
///
/// * `quotes` - A vector of strings representing the symbols for which quote-related data is acknowledged as subscribed.
///   Defaults to an empty vector if not specified.
///
/// * `bars` - A vector of strings representing the symbols for which bar-related (e.g., candlestick) data is acknowledged as subscribed.
///   Defaults to an empty vector if not specified.
///
/// * `daily_bars` (`dailyBars`) - A vector of strings representing the symbols for which daily bar-related data
///   is acknowledged as subscribed. This field is deserialized from the key `dailyBars` in a serialized structure.
///   Defaults to an empty vector if not specified.
///
/// * `updated_bars` (`updatedBars`) - A vector of strings representing the symbols for which updated bar-related data
///   is acknowledged as subscribed. This field is deserialized from the key `updatedBars` in a serialized structure.
///   Defaults to an empty vector if not specified.
///
/// * `orderbooks` - A vector of strings representing the symbols for which order book data is acknowledged as subscribed.
///   Defaults to an empty vector if not specified.
///
/// # Derived Traits
///
/// * `Debug` - Enables formatting of the structure for debugging purposes.
/// * `Deserialize` - Provides deserialization capabilities to convert structured data (e.g., from JSON) into a `SubscriptionAck` instance.
/// * `Clone` - Allows creating a deep copy of the structure.
///
/// # Usage Example
///
/// ```rust
/// use serde::Deserialize;
/// use rpaca::market_data::v2::crypto_websocket::{SubscriptionAck, NumF64};
///
/// let json_data = r#"
///     {
///         "trades": ["AAPL", "GOOG"],
///         "quotes": ["TSLA"],
///         "dailyBars": ["MSFT"]
///     }
/// "#;
///
/// let subscription_ack: SubscriptionAck = serde_json::from_str(json_data).unwrap();
///
/// assert_eq!(subscription_ack.trades, vec!["AAPL", "GOOG"]);
/// assert_eq!(subscription_ack.quotes, vec!["TSLA"]);
/// assert_eq!(subscription_ack.daily_bars, vec!["MSFT"]);
/// assert!(subscription_ack.bars.is_empty());
/// assert!(subscription_ack.updated_bars.is_empty());
/// assert!(subscription_ack.orderbooks.is_empty());
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct SubscriptionAck {
    #[serde(default)] pub trades: Vec<String>,
    #[serde(default)] pub quotes: Vec<String>,
    #[serde(default)] pub bars: Vec<String>,
    #[serde(rename = "dailyBars", default)] pub daily_bars: Vec<String>,
    #[serde(rename = "updatedBars", default)] pub updated_bars: Vec<String>,
    #[serde(default)] pub orderbooks: Vec<String>,
}

/// Represents a success message structure typically used for responses.
///
/// This struct is used to encapsulate a generic success message and
/// an associated optional status code. Both fields are optional to allow
/// for flexibility in different use cases.
///
/// # Fields
/// - `msg`:
///   An optional string containing a human-readable success message. For example,
///   it could be a message like "Operation successful".
///
/// - `code`:
///   An optional integer representing a success status code. This is typically
///   used for applications where success codes (non-error codes) need to be passed back.
///
/// # Derives
/// - `Debug`: Allows the struct to be formatted using the `{:?}` formatter,
///   useful for debugging purposes.
/// - `Deserialize`: Enables deserialization from formats like JSON.
/// - `Clone`: Allows the struct to be cloned, useful in scenarios where
///   multiple copies of the structure need to exist.
///
#[derive(Debug, Deserialize, Clone)]
pub struct SuccessMsg {
    pub msg: Option<String>,
    pub code: Option<i64>,
}

/// A struct that represents an error message with an optional message and optional error code.
///
/// This struct is commonly used to encapsulate error-related information in a structured format.
/// It derives the `Debug`, `Deserialize`, and `Clone` traits, which allow for debugging output,
/// deserialization from formats like JSON, and cloning of the struct, respectively.
///
/// Fields:
///
/// * `msg` - An optional String field that represents the error message.
///   If present, this contains a human-readable description of the error.
///
/// * `code` - An optional i64 field that represents the error code.
///   If present, this typically contains a machine-readable code corresponding to the error.
///
/// Example:
///
/// ```
/// use serde::Deserialize;
/// #[derive(Debug, Deserialize, Clone)]
/// pub struct ErrorMsg {
///     pub msg: Option<String>,
///     pub code: Option<i64>,
/// }
///
/// let error = ErrorMsg {
///     msg: Some(String::from("An unknown error occurred")),
///     code: Some(500),
/// };
///
/// println!("{:?}", error); // Output: ErrorMsg { msg: Some("An unknown error occurred"), code: Some(500) }
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct ErrorMsg {
    pub msg: Option<String>,
    pub code: Option<i64>,
}

/// A data structure representing a trade event in the system.
///
/// This struct is deserialized from a JSON payload and contains the key details
/// of a trade, such as the traded symbol, price, size, timestamp, trade ID, and
/// the side of the trade (taker or maker).
///
/// # Fields
/// - `symbol` (`String`): The trading symbol (e.g., "BTCUSD") representing the currency pair.
///   This field is deserialized from the `S` field in the JSON payload.
/// - `price` (`f64`): The price at which the trade was executed.
///   This field is deserialized from the `p` field in the JSON payload.
/// - `size` (`f64`): The size or quantity of the asset involved in the trade.
///   This field is deserialized from the `s` field in the JSON payload.
/// - `timestamp` (`String`): A timestamp indicating when the trade occurred. It is generally
///   provided in ISO 8601 or Unix epoch timestamp format.
///   This field is deserialized from the `t` field in the JSON payload.
/// - `trade_id` (`NumF64`): The unique identifier for the trade. This is deserialized
///   from the `i` field in the JSON payload.
/// - `taker_side` (`String`): Indicates whether the trade was initiated by a taker
///   on the buy side (`"BUY"`) or the sell side (`"SELL"`). This field is deserialized
///   from the `tks` field in the JSON payload.
///
/// # Derives
/// - `Debug`: Allows the `Trade` struct to be formatted using the `{:?}` formatter, useful for debugging purposes.
/// - `Deserialize`: Enables the deserialization of the `Trade` struct from JSON using Serde.
/// - `Clone`: Allows the `Trade` struct to be duplicated (cloned) with the same field values.
///
/// # Example
/// ```rust
/// use serde_json::from_str;
/// use rpaca::market_data::v2::crypto_websocket::{Trade, NumF64};
///
/// let json_trade = r#"{
///     "S": "BTCUSD",
///     "p": 34000.00,
///     "s": 1.25,
///     "t": "2023-10-15T12:34:56Z",
///     "i": 1029384756,
///     "tks": "BUY"
/// }"#;
///
/// let trade: Trade = from_str(json_trade).unwrap();
///
/// println!("{:?}", trade);
/// ```
///
/// This will produce:
/// ```text
/// Trade {
///     symbol: "BTCUSD",
///     price: 34000.0,
///     size: 1.25,
///     timestamp: "2023-10-15T12:34:56Z",
///     trade_id: 1029384756.0,
///     taker_side: "BUY"
/// }
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct Trade {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "p")] pub price: f64,
    #[serde(rename = "s")] pub size: f64,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "i")] pub trade_id: NumF64,
    #[serde(rename = "tks")] pub taker_side: String,
}

/// Represents financial market data for a specific trading instrument,
/// encapsulating bid and ask prices, their respective sizes, and a timestamp.
///
/// This struct is primarily used for deserializing data from an external API or data source,
/// and leverages the `serde` library for mapping incoming JSON structures to Rust types.
///
/// # Fields
///
/// * `symbol` (`String`):
///   Represents the symbol (ticker) of the trading instrument, e.g., "AAPL" for Apple Inc.
///   Deserialized from the JSON key `"S"`.
///
/// * `bid_price` (`f64`):
///   The best bid (highest price a buyer is willing to pay) for the instrument.
///   Deserialized from the JSON key `"bp"`.
///
/// * `bid_size` (`f64`):
///   The size (quantity) associated with the best bid.
///   Deserialized from the JSON key `"bs"`.
///
/// * `ask_price` (`f64`):
///   The best ask (lowest price a seller is willing to accept) for the instrument.
///   Deserialized from the JSON key `"ap"`.
///
/// * `ask_size` (`f64`):
///   The size (quantity) associated with the best ask.
///   Deserialized from the JSON key `"as"`.
///
/// * `timestamp` (`String`):
///   The timestamp indicating when the quote data was recorded or received.
///   Deserialized from the JSON key `"t"`.
///
/// # Trait Implementations
///
/// * `Debug`: Enables the struct to be formatted using the `{:?}` formatter for debugging purposes.
/// * `Deserialize`: Allows the struct to be deserialized from formats like JSON or other supported data sources.
/// * `Clone`: Enables the creation of a copy of the struct.
///
/// # Example
///
/// ```rust
/// use serde::Deserialize;
/// use rpaca::market_data::v2::crypto_websocket::Quote;
///
/// let json_data = r#"{
///     "S": "AAPL",
///     "bp": 145.32,
///     "bs": 100.0,
///     "ap": 145.35,
///     "as": 120.0,
///     "t": "2023-10-16T12:00:00Z"
/// }"#;
///
/// let quote: Quote = serde_json::from_str(json_data).unwrap();
///
/// println!("{:?}", quote);
/// ```
///
/// In this example, the JSON data is successfully deserialized into a `Quote` instance,
/// which then prints the associated fields for easy inspection.
#[derive(Debug, Deserialize, Clone)]
pub struct Quote{
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "bp")] pub bid_price: f64,
    #[serde(rename = "bs")] pub bid_size: f64,
    #[serde(rename = "ap")] pub ask_price: f64,
    #[serde(rename = "as")] pub ask_size: f64,
    #[serde(rename = "t")] pub timestamp: String,
}

/// The `Bar` struct represents a trading data entity, commonly used in financial markets
/// to encapsulate data for a single period of time in a candlestick format.
///
/// Each field in the struct is deserialized from an external data source, where
/// the field names are mapped to the corresponding keys in the source via the `#[serde(rename = "...")]` attribute.
///
/// # Fields
///
/// * `symbol` (`String`) - The trading symbol of the asset, deserialized from the `"S"` key.
/// * `open` (`f64`) - The opening price for the trading period, deserialized from the `"o"` key.
/// * `high` (`f64`) - The highest price during the trading period, deserialized from the `"h"` key.
/// * `low` (`f64`) - The lowest price during the trading period, deserialized from the `"l"` key.
/// * `close` (`f64`) - The closing price for the trading period, deserialized from the `"c"` key.
/// * `volume` (`NumF64`) - The trading volume during the period, which is represented as a `NumF64` type, deserialized from the `"v"` key.
/// * `timestamp` (`String`) - The timestamp for the trading data, typically in ISO 8601 format, deserialized from the `"t"` key.
///
/// # Traits
///
/// * `Debug` - Enables the struct to be formatted using the `{:?}` formatter, useful for debugging and logging.
/// * `Deserialize` - Allows the struct to be deserialized from an external data source (e.g., JSON).
/// * `Clone` - Allows for creating a duplicate of the `Bar` instance.
/// This struct is commonly used for processing market data, such as candlestick data in financial applications.
#[derive(Debug, Deserialize, Clone)]
pub struct Bar{
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "o")] pub open: f64,
    #[serde(rename = "h")] pub high: f64,
    #[serde(rename = "l")] pub low: f64,
    #[serde(rename = "c")] pub close: f64,
    #[serde(rename = "v")] pub volume: NumF64,
    #[serde(rename = "t")] pub timestamp: String,
}

/// A struct representing a level with two parameters.
///
/// The `Level` struct is used to define a level with numerical values for parameters `p` and `s`.
/// This struct derives the `Debug`, `Deserialize`, and `Clone` traits, making it useful for debugging,
/// deserialization (e.g., from a file or API), and cloning.
///
/// Fields:
/// - `p` (`f64`): A floating-point value representing the primary parameter of the level.
/// - `s` (`f64`): A floating-point value representing the secondary parameter of the level.
///
/// Example:
/// ```
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, Clone)]
/// pub struct Level {
///     pub p: f64,
///     pub s: f64,
/// }
///
/// let level = Level { p: 1.23, s: 4.56 };
/// println!("{:?}", level); // Outputs: Level { p: 1.23, s: 4.56 }
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct Level {
    pub p: f64,
    pub s: f64,
}

/// Represents an orderbook structure, which contains information about bids and asks
/// for a specific trading symbol at a given timestamp.
///
/// The `Orderbook` struct is used to deserialize data typically obtained from market
/// data feeds. It supports additional metadata such as a reset flag to indicate the
/// need to refresh the orderbook state.
///
/// # Fields
///
/// * `symbol` (`String`):
///   The trading symbol or market identifier, such as "BTCUSD" or "ETHUSDT".
///   This field is deserialized from the "S" key in the source data.
///
/// * `timestamp` (`String`):
///   A string representing the timestamp at which the orderbook data was created
///   or updated. This is deserialized from the "t" key in the source data.
///
/// * `bids` (`Vec<Level>`):
///   A vector of bid levels representing buy orders in the orderbook.
///   Each item in this vector corresponds to a price level and its associated quantity.
///   This is deserialized from the "b" key in the source data.
///
/// * `asks` (`Vec<Level>`):
///   A vector of ask levels representing sell orders in the orderbook.
///   Like `bids`, each item corresponds to a price level and its associated quantity.
///   This is deserialized from the "a" key in the source data.
///
/// * `reset` (`Option<bool>`
#[derive(Debug, Deserialize, Clone)]
pub struct Orderbook {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "b")] pub bids: Vec<Level>,
    #[serde(rename = "a")] pub asks: Vec<Level>,
    #[serde(rename = "r")] pub reset: Option<bool>,
}

/// Represents various types of stock market messages that can be deserialized and processed.
/// This enum leverages `serde` for deserialization and is tagged using the `T` field to determine the variant type.
///
/// Variants:
/// - `Trade`: Represents a trade message, tagged as `"t"`.
/// - `Quote`: Represents a quote message, tagged as `"q"`.
/// - `Bar`: Represents a bar message containing aggregated market data for a specific interval, tagged as `"b"`.
/// - `DailyBar`: Represents a daily bar message, tagged as `"d"`.
/// - `UpdatedBar`: Represents an updated bar message, tagged as `"u"`.
/// - `Orderbook`: Represents an order book message, tagged as `"o"`.
/// - `Subscription`: Acknowledges a successful subscription to specific data streams, tagged as `"subscription"`.
/// - `Success`: Represents a success message, indicating an operation was successful, tagged as `"success"`.
/// - `Error`: Represents an error message, indicating an issue or problem, tagged as `"error"`.
///
/// Each variant corresponds to a specific message type and is deserialized according to the provided tag.
///
/// This enum is derivable as `Debug` and `Clone` and requires deserialization through the `serde` library.
#[derive(Debug, Deserialize,Clone)]
#[serde(tag = "T")]
pub enum StockMsg {
    // market data
    #[serde(rename = "t")] Trade(Trade),
    #[serde(rename = "q")] Quote(Quote),
    #[serde(rename = "b")] Bar(Bar),
    #[serde(rename = "d")] DailyBar(Bar),
    #[serde(rename = "u")] UpdatedBar(Bar),
    #[serde(rename = "o")] Orderbook(Orderbook),

    #[serde(rename = "subscription")] Subscription(SubscriptionAck),
    #[serde(rename = "success")] Success(SuccessMsg),
    #[serde(rename = "error")] Error(ErrorMsg),
}

/// Represents the parameters required to set up a crypto data WebSocket stream.
///
/// # Fields
///
/// * `endpoint`
///     - The WebSocket endpoint URL used to establish the crypto data stream connection.
///     - Defaults to `"wss://stream.data.alpaca.markets/v1beta3/crypto/us"`.
///     - Example: `"wss://stream.data.sandbox.alpaca.markets"`.
///
/// * `subscription`
///     - The subscription details specifying the crypto data streams/topics to subscribe to.
///
#[derive(Debug, TypedBuilder, Serialize)]
pub struct CryptoStreamParams{
    #[builder(default = "wss://stream.data.alpaca.markets/v1beta3/crypto/us".to_string())]
    pub endpoint: String, // e.g., "wss://stream.data.sandbox.alpaca.markets"
    pub subscription: Subscribe,
}

/// Streams cryptocurrency data using the Alpaca WebSocket API.
///
/// This asynchronous function establishes a WebSocket connection to Alpaca's
/// cryptocurrency streaming API, handles authentication, subscribes to the
/// provided crypto stream channels, and continuously streams data. It handles
/// automatic reconnection with an exponential backoff strategy in case of
/// errors such as dropped connections or authentication issues.
///
/// # Parameters
///
/// - `alpaca`: A reference to an [`Alpaca`] client that contains the API key
///   and secret used for authentication.
/// - `params`: A [`CryptoStreamParams`] struct that contains the WebSocket
///   endpoint and the subscription details (e.g., channels to subscribe to).
///
/// # Returns
///
/// An asynchronous operation that resolves to a [`Result`] containing a stream.
/// The stream yields [`StockMsg`] objects wrapped in a [`Result`]:
/// - On success, data payloads from the WebSocket are returned as `Ok(StockMsg)`.
/// - On failure, an error description is returned as `Err`.
///
/// The return type uses `impl futures_core::Stream` for flexibility, enabling
/// it to work with various stream combinator libraries or patterns.
///
/// # Behavior
///
/// 1. The function establishes a WebSocket connection to the specified
///    endpoint in `params`.
/// 2. It sends an authentication message using the API key and secret from
///    the `alpaca` client.
/// 3. Upon successful authentication, it sends a subscription message
///    containing the stream channel configuration.
/// 4. It listens for incoming messages on the WebSocket connection:
///    - It parses incoming JSON text payloads into `StockMsg` objects.
///    - Successfully parsed messages are sent to the output stream.
///    - Any errors (e.g., decoding errors) are sent as `Err` to the output stream.
/// 5. If the connection is closed, interrupted, or an error occurs, it tries
///    to reconnect indefinitely with an exponential backoff strategy (the max
///    backoff time between attempts is capped).
///
/// # Reconnection Logic
///
/// If the connection fails or the server closes the WebSocket:
/// - The function automatically retries connecting to the server.
/// - The initial delay between retry attempts starts at 250ms, doubling on
///   each failure, up to a maximum delay of 16 seconds (capped at 6 retries).
/// - Once reconnected, it re-authenticates and resends the subscription.
///
/// # Errors
///
/// - Authentication failures are sent as `Err` when they occur.
/// - Any issue during message parsing or WebSocket communication will also
///   be sent as an error.
/// - In the case of unrecoverable errors during reconnection, the stream will
///   continue attempting indefinitely, maintaining the backoff strategy.
///
/// # Notes
///
/// - The `StockMsg` type is used for all incoming WebSocket messages, including
///   success or error responses and actual data payloads.
/// - The function uses the `tokio` library for asynchronous tasks and channel management.
/// - The `serde_json` library is used for JSON encoding and decoding.
pub async fn stream_crypto_data(
    alpaca: &Alpaca,
    params: CryptoStreamParams,
) -> Result<impl futures_core::Stream<Item = Result<StockMsg>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<StockMsg>>(1024);

    let endpoint = params.endpoint.to_string();
    let key = alpaca.apca_api_key_id.clone();
    let secret = alpaca.apca_api_secret_key.clone();
    let subscribe_json = params.subscription.action_json();

    tokio::spawn(async move {
        let mut attempt: u32 = 0;

        loop {
            let conn = connect_async(&endpoint).await;

            let (ws, _) = match conn {
                Ok(ok) => {
                    attempt = 0;
                    ok
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow!("connect: {e}"))).await;
                    attempt += 1;
                    let backoff_ms = (1u64 << attempt.min(6)) * 250;
                    sleep(Duration::from_millis(backoff_ms)).await;
                    continue;
                }
            };

            let (mut write, mut read) = ws.split();

            // Step 1: Send auth right away (the server will also emit a "connected" success)
            let auth = serde_json::json!({ "action": "auth", "key": key, "secret": secret });
            if let Err(e) = write.send(Message::Text(Utf8Bytes::from(auth.to_string()))).await {
                let _ = tx.send(Err(anyhow!("send auth: {e}"))).await;
                continue;
            }

            // Step 2: Wait until we see "authenticated"
            let mut authed = false;
            while let Some(incoming) = read.next().await {
                match incoming {
                    Ok(Message::Text(txt)) => {
                        match serde_json::from_str::<Vec<StockMsg>>(&txt) {
                            Ok(batch) => {
                                for msg in batch {
                                    match &msg {
                                        StockMsg::Success(s) if matches!(s.msg.as_deref(), Some("connected")) => {
                                            // ignore
                                        }
                                        StockMsg::Success(s) if matches!(s.msg.as_deref(), Some("authenticated")) => {
                                            authed = true;
                                        }
                                        StockMsg::Error(e) => {
                                            let _ = tx.send(Err(anyhow!(
                                                "auth/handshake error: code={:?} msg={:?}",
                                                e.code, e.msg
                                            ))).await;
                                            // Break to reconnect loop.
                                            authed = false;
                                            break;
                                        }
                                        _ => {
                                            // deliver anything else (rare during auth) to consumers
                                            let _ = tx.send(Ok(msg)).await;
                                        }
                                    }
                                }
                                if authed { break; }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow!("decode during auth: {e}"))).await;
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(_) => {} // ignore non-text frames
                    Err(e) => {
                        let _ = tx.send(Err(anyhow!("read during auth: {e}"))).await;
                        break;
                    }
                }
            }

            if !authed {
                // reconnect with backoff
                attempt += 1;
                let backoff_ms = (1u64 << attempt.min(6)) * 250;
                sleep(Duration::from_millis(backoff_ms)).await;
                continue;
            }

            // Step 3: Send subscribe
            if let Err(e) = write.send(Message::Text(Utf8Bytes::from(subscribe_json.to_string()))).await {
                let _ = tx.send(Err(anyhow!("send subscribe: {e}"))).await;
                // reconnect
                attempt += 1;
                let backoff_ms = (1u64 << attempt.min(6)) * 250;
                sleep(Duration::from_millis(backoff_ms)).await;
                continue;
            }

            // Step 4: Main stream loop
            while let Some(incoming) = read.next().await {
                match incoming {
                    Ok(Message::Text(txt)) => {
                        match serde_json::from_str::<Vec<StockMsg>>(&txt) {
                            Ok(batch) => {
                                for msg in batch {
                                    let _ = tx.send(Ok(msg)).await;
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow!("decode: {e}"))).await;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        // remote closed; break to reconnect
                        break;
                    }
                    Ok(_) => {} // ignore ping/pong/binary
                    Err(e) => {
                        let _ = tx.send(Err(anyhow!("read: {e}"))).await;
                        break;
                    }
                }
            }

            // Step 5: Reconnect with backoff
            attempt += 1;
            let backoff_ms = (1u64 << attempt.min(6)) * 250;
            sleep(Duration::from_millis(backoff_ms)).await;
        }
    });

    Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
}


#[tokio::test]
async fn test_crypto_ws(){
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();

    let mut stream = stream_crypto_data(&alpaca, CryptoStreamParams::builder()
        .subscription(Subscribe{
            trades: vec!["BTC/USD".to_string()],
            quotes: vec!["BTC/USD".to_string()],
            bars: vec!["BTC/USD".to_string()],
            orderbooks: vec!["BTC/USD".to_string()],
            ..Default::default()
        }).build()).await.unwrap();

    let mut got_quote = false;
    let mut got_bar = false;
    let mut got_ack = false;

    let _ = timeout(Duration::from_secs(360), async {
        while let Some(item) = stream.next().await {
            match item.as_ref().unwrap() {
                StockMsg::Subscription(ack) => {
                    assert!(ack.trades.contains(&"BTC/USD".to_string()));
                    got_ack = true;
                }
                StockMsg::Trade(t) => {
                    assert_eq!(t.symbol, "BTC/USD");
                }
                StockMsg::Quote(q) => {
                    assert_eq!(q.symbol, "BTC/USD");
                    assert!(q.ask_price > 0.0 && q.bid_price > 0.0);
                    got_quote = true;
                }
                StockMsg::Bar(b) => {
                    assert_eq!(b.symbol, "BTC/USD");
                    got_bar = true;
                }
                StockMsg::Orderbook(o) => {
                    assert_eq!(o.symbol, "BTC/USD");
                }
                _ => {println!("Got unknown item: {item:?}");}
            }

            if got_ack && got_quote && got_bar {
                break;
            }
        }
    }).await;

    assert!(got_ack, "did not receive subscription ack");
    assert!(got_quote, "did not receive quote");
    assert!(got_bar, "did not receive bar");
}
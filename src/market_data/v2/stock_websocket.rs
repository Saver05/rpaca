//! Represents a trade record containing various details about a trade.
//!
//! This struct is used primarily for deserializing trade details from external data sources,
//! such as JSON feeds or APIs. Each field corresponds to a specific attribute within the
//! trade data, and custom serialization mappings are configured using the `#[serde(rename)]` attribute.
//!
//! # Fields
//! - `symbol` (`String`): The ticker symbol of the asset involved in the trade (e.g., "AAPL").
//!   It is deserialized from the `"S"` field in the source data.
//! - `trade_id` (`i64`): A unique identifier for the trade within the exchange. It is deserialized
//!   from the `"i"` field.
//! - `exchange` (`String`): The specific exchange where the trade occurred. It is deserialized
//!   from the `"x"` field.
//! - `price` (`f64`): The price per unit of the asset that was traded. It is deserialized from the
//!   `"p"` field.
//! - `size` (`i64`): The quantity of the asset that was traded. It is deserialized from the `"s"` field.
//!
//! # Examples
//! ```rust
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize, Clone, Serialize)]
//! pub struct Trade {
//!     #[serde(rename = "S")]
//!     pub symbol: String,
//!     #[serde(rename = "i")]
//!     pub trade_id: i64,
//!     #[serde(rename = "x")]
//!     pub exchange: String,
//!     #[serde(rename = "p")]
//!     pub price: f64,
//!     #[serde(rename = "s")]
//!     pub size: i64,
//! }
//!
//! let json_data = r#"{
//!     "S": "AAPL",
//!     "i": 12345,
//!     "x": "NYSE",
//!     "p": 150.25,
//!     "s": 50
//! }"#;
//!
//! let trade: Trade = serde_json::from_str(json_data).unwrap();
//! println!("{:?}", trade);
//! // Output: Trade { symbol: "AAPL", trade_id: 12345, exchange: "NYSE", price: 150.25, size: 50 }
//! ```
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::Utf8Bytes;
use typed_builder::TypedBuilder;
use crate::auth::{Alpaca, TradingType};

/// The `Subscribe` struct is used to define a subscription payload for various data streams,
/// such as trades, quotes, bars, daily bars, updated bars, statuses, luld events, and imbalances.
///
/// # Fields
///
/// - `trades` (`Vec<String>`):
///   A vector of strings specifying the symbols for which trade updates are subscribed to.
///   This field is serialized only if it is non-empty.
///
/// - `quotes` (`Vec<String>`):
///   A vector of strings specifying the symbols for which quote updates are subscribed to.
///   This field is serialized only if it is non-empty.
///
/// - `bars` (`Vec<String>`):
///   A vector of strings specifying the symbols for which bar (aggregated price/volume data) updates are subscribed to.
///   This field is serialized only if it is non-empty.
///
/// - `daily_bars` (`Vec<String>`):
///   A vector of strings specifying the symbols for which daily bar updates are subscribed to.
///   This field is serialized only if it is non-empty and is serialized with the key `dailyBars`.
///
/// - `updated_bars` (`Vec<String>`):
///   A vector of strings specifying the symbols for which updated bar data is subscribed to.
///   This field is serialized only if it is non-empty and is serialized with the key `updatedBars`.
///
/// - `statuses` (`Vec<String>`):
///   A vector of strings specifying the symbols for which status updates are subscribed to.
///   This field is serialized only if it is non-empty.
///
/// - `lulds` (`Vec<String>`):
///   A vector of strings specifying the symbols for which LULD (Limit Up / Limit Down) updates are subscribed to.
///   This field is serialized only if it is non-empty.
///
/// - `imbalances` (`Vec<String>`):
///   A vector of strings specifying the symbols for which imbalance updates are subscribed to.
///   This field is serialized only if it is non-empty.
///
/// # Attributes
///
/// - The struct implements:
///   - `Debug`: Enables debugging and printing of the struct.
///   - `Default`: Provides a default implementation for the struct.
///   - `Clone`: Allows cloning of the struct.
///   - `Serialize`: Allows serialization of the struct, with conditional serialization for empty fields.
///
/// - Serialization-specific options:
///   - Fields are skipped during serialization if their corresponding vectors are empty (`skip_serializing_if = "Vec::is_empty"`).
///   - Some fields (`daily_bars` and `updated_bars`) use custom field names (`dailyBars` and `updatedBars`) during serialization.
///
/// # Example
///
/// ```rust
/// use serde_json;
/// use rpaca::market_data::v2::stock_websocket::Subscribe;
///
/// let subscription = Subscribe {
///     trades: vec!["AAPL".to_string(), "GOOG".to_string()],
///     quotes: vec![],
///     bars: vec!["TSLA".to_string()],
///     daily_bars: vec![],
///     updated_bars: vec![],
///     statuses: vec![],
///     lulds: vec!["AMD".to_string()],
///     imbalances: vec![],
/// };
///
/// let serialized = serde_json::to_string(&subscription).unwrap();
/// println!("{}", serialized);
/// ```
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
    pub statuses: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lulds: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imbalances: Vec<String>,
}

impl Subscribe {
    /// Creates a new instance of the type using its default implementation.
    ///
    /// This function is a simple wrapper around the `default` method and allows
    /// for convenient initialization of the type.
    ///
    /// # Returns
    /// A new instance of the type initialized with default values.
    pub fn new() -> Self {
        Self::default()
    }
    /// Constructs and returns a JSON object representing a subscription action with various data feeds.
    ///
    /// # Returns
    /// A `serde_json::Value` object containing the subscription action and relevant data feed details.
    /// The JSON object includes the following fields:
    /// - `"action"`: Always set to `"subscribe"`.
    /// - `"trades"`: Represents trade subscription status (likely a boolean or other serializable type).
    /// - `"quotes"`: Represents quote subscription status.
    /// - `"bars"`: Represents bar (e.g., candlestick) subscription status.
    /// - `"dailyBars"`: Represents daily bar subscription status.
    /// - `"updatedBars"`: Represents updated bar subscription status.
    /// - `"statuses"`: Represents subscription status for status updates.
    /// - `"lulds"`: Represents subscription status for Limit Up/Limit Down (LULD) updates.
    /// - `"imbalances"`: Represents subscription status for imbalance updates.
    ///
    /// # Notes
    /// - This method assumes the attributes (like `trades`, `quotes`, `bars`, etc.) are fields
    ///   of the struct and are serializable to JSON using `serde_json::json!`.
    /// - The caller is responsible for ensuring the struct fields are properly initialized
    ///   before invoking this method.
    ///
    /// ```
    pub fn action_json(&self) -> serde_json::Value {
        serde_json::json!({
            "action": "subscribe",
            "trades": self.trades,
            "quotes": self.quotes,
            "bars": self.bars,
            "dailyBars": self.daily_bars,
            "updatedBars": self.updated_bars,
            "statuses": self.statuses,
            "lulds": self.lulds,
            "imbalances": self.imbalances,
        })
    }
}

/// A struct representing an acknowledgment for a subscription, which includes details
/// about the subscribed data streams.
///
/// The struct is deserialized from an external source, such as JSON, and uses optional
/// fields for each type of stream. Any unspecified stream data will default to an empty
/// vector.
///
/// Fields:
/// - `trades` (`Vec<String>`): A collection of subscribed trade channels (e.g., stock symbols)
///   that the user has successfully acknowledged. Defaults to an empty vector.
/// - `quotes` (`Vec<String>`): A collection of subscribed quote channels (e.g., stock symbols)
///   that are acknowledged. Defaults to an empty vector.
/// - `bars` (`Vec<String>`): A list of subscribed bar channels (e.g., stock symbols for candlestick
///   data). Defaults to an empty vector.
/// - `daily_bars` (`Vec<String>`): A list of subscribed daily bar channels, deserialized from
///   `"dailyBars"`. Represents daily candlestick data symbols. Defaults to an empty vector.
/// - `updated_bars` (`Vec<String>`): A list of subscribed updated bar channels, deserialized
///   from `"updatedBars"`. Represents continuously updated candlestick data. Defaults to an empty vector.
/// - `statuses` (`Vec<String>`): A collection of subscribed status channels (e.g., market or stock statuses).
///   Defaults to an empty vector.
/// - `lulds` (`Vec<String>`): A collection of subscribed limit up/limit down (LULD) channels. Defaults
///   to an empty vector.
/// - `imbalances` (`Vec<String>`): A collection of subscribed imbalance data channels, typically used
///   for tracking market imbalances. Defaults to an empty vector.
/// - `corrections` (`Vec<String>`): A collection of subscribed correction data channels, which include
///   adjustments to previously reported trade or quote information. Defaults to an empty vector.
/// - `cancel_errors` (`Vec<String>`): A collection of subscribed cancel error channels, deserialized
///   from `"cancelErrors"`. Represents cancellations or errors related to orders. Defaults to an empty vector.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SubscriptionAck {
    #[serde(default)] pub trades: Vec<String>,
    #[serde(default)] pub quotes: Vec<String>,
    #[serde(default)] pub bars: Vec<String>,
    #[serde(rename = "dailyBars", default)] pub daily_bars: Vec<String>,
    #[serde(rename = "updatedBars", default)] pub updated_bars: Vec<String>,
    #[serde(default)] pub statuses: Vec<String>,
    #[serde(default)] pub lulds: Vec<String>,
    #[serde(default)] pub imbalances: Vec<String>,
    #[serde(default)] pub corrections: Vec<String>,
    #[serde(rename = "cancelErrors", default)] pub cancel_errors: Vec<String>,
}

/// A data structure representing a success message with an optional message and an optional code.
///
/// This structure is used to encapsulate information about a successful operation,
/// including an optional descriptive message (`msg`) and an optional numerical code (`code`).
/// It derives the `Debug`, `Deserialize`, and `Clone` traits to enable debugging, deserialization,
/// and cloning functionality.
///
/// # Fields
/// - `msg` (`Option<String>`): An optional string containing a success message. If `None`, no message is provided.
/// - `code` (`Option<i64>`): An optional integer representing a success code. If `None`, no code is provided.
///
/// # Examples
/// ```
/// use rpaca::market_data::v2::stock_websocket::SuccessMsg;
///
/// let success = SuccessMsg {
///     msg: Some("Operation completed successfully".to_string()),
///     code: Some(200),
/// };
///
/// println!("{:?}", success); // Output: SuccessMsg { msg: Some("Operation completed successfully"), code: Some(200) }
/// ```
///
/// This struct can be useful in scenarios where you need to return structured success information from a function or API.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SuccessMsg {
    pub msg: Option<String>,
    pub code: Option<i64>,
}

/// Represents an error message with an optional message string and an optional error code.
///
/// # Fields
/// - `msg` (`Option<String>`): An optional string containing an error message. It can be `None` if no message is provided.
/// - `code` (`Option<i64>`): An optional 64-bit integer representing an error code. It can be `None` if no code is provided.
///
/// # Derives
/// - `Debug`: Allows instances of `ErrorMsg` to be formatted using the `fmt::Debug` trait for debugging purposes.
/// - `Deserialize`: Enables deserialization of `ErrorMsg` from various formats (e.g., JSON).
/// - `Clone`: Allows cloning of `ErrorMsg` instances to create deep copies.
///
/// # Example
/// ```rust
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, Clone, Serialize)]
/// pub struct ErrorMsg {
///     pub msg: Option<String>,
///     pub code: Option<i64>,
/// }
///
/// let error = ErrorMsg {
///     msg: Some(String::from("An error occurred")),
///     code: Some(404),
/// };
///
/// println!("{:?}", error);
/// ```
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ErrorMsg {
    pub msg: Option<String>,
    pub code: Option<i64>,
}

/// Represents a trade record with various details about the trade.
///
/// This struct is used for deserialization of trade data, typically from JSON,
/// using Serde. Each field is mapped to a specific key in the source data
/// which is specified using the `#[serde(rename = "...")]` attribute.
///
/// Fields:
/// - `symbol`: The symbol or ticker of the asset being traded (e.g., "AAPL").
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Trade {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "i")] pub trade_id: i64,
    #[serde(rename = "x")] pub exchange: String,
    #[serde(rename = "p")] pub price: f64,
    #[serde(rename = "s")] pub size: i64,
    #[serde(rename = "c")] pub conditions: Vec<String>,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "z")] pub tape: String,
}

/// Represents a market quote for a specific financial instrument, including bid and ask details.
///
/// This struct is used to deserialize JSON data about market quotes and provides information such as
/// the symbol, bid/ask prices and sizes, exchange identifiers, and additional metadata.
///
/// Fields:
/// - `symbol` (`String`):
///   The ticker symbol of the financial instrument.
///
/// - `ask_exchange` (`String`):
///   Identifier for the exchange that provided the ask price.
///
/// - `ask_price` (`f64`):
///   The current asking price for the financial instrument.
///
/// - `ask_size` (`i64`):
///   The size (quantity) available at the ask price.
///
/// - `bid_exchange` (`String`):
///   Identifier for the exchange that provided the bid price.
///
/// - `bid_price` (`f64`):
///   The current bidding price for the financial instrument.
///
/// - `bid_size` (`i64`):
///   The size (quantity) available at the bid price.
///
/// - `conditions` (`Vec<String>`):
///   A list of conditions or qualifiers tied to the quote (if any).
///
/// - `timestamp` (`String`):
///   The time when the quote was recorded, formatted as an ISO 8601 string.
///
/// - `tape` (`String`):
///   The identifier for the tape (or market data stream) on which the quote is published.
///
/// This struct derives traits for `Debug`, `Clone`, and implements deserialization using `serde`.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Quote {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "ax")] pub ask_exchange: String,
    #[serde(rename = "ap")] pub ask_price: f64,
    #[serde(rename = "as")] pub ask_size: i64,
    #[serde(rename = "bx")] pub bid_exchange: String,
    #[serde(rename = "bp")] pub bid_price: f64,
    #[serde(rename = "bs")] pub bid_size: i64,
    #[serde(rename = "c")] pub conditions: Vec<String>,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "z")] pub tape: String,
}

///
/// A struct representing a financial trading bar (candlestick),
/// commonly used in financial data to depict price movements over a specific time period.
///
/// Each bar provides details about the symbol, open price, high price,
/// low price, close price, volume, volume-weighted average price, number of trades,
/// and the timestamp for the bar.
///
/// Fields:
///
/// * `symbol` (String): The trading symbol the data is associated with.
///   This is renamed in serialized data as "S".
///
/// * `open` (f64): The opening price of the symbol for the time period.
///   This is renamed in serialized data as "o".
///
/// * `high` (f64): The highest price of the symbol for the time period.
///   This is renamed in serialized data as "h".
///
/// * `low` (f64): The lowest price of the symbol for the time period.
///   This is renamed in serialized data as "l".
///
/// * `close` (f64): The closing price of the symbol for the time period.
///   This is renamed in serialized data as "c".
///
/// * `volume` (i64): The total trading volume during the time period.
///   This is renamed in serialized data as "v".
///
/// * `volume_weighted_avg_price` (f64): The volume-weighted average price for the symbol
///   during the time period. This is renamed in serialized data as "vw".
///
/// * `number_of_trades` (i64): The total number of trades that occurred during the time period.
///   This is renamed in serialized data as "n".
///
/// * `timestamp` (String): The timestamp indicating the end of the bar's time period
///   in ISO 8601 format. This is renamed in serialized data as "t".
///
/// Notes:
/// - This struct implements the `Debug`, `Deserialize`, and `Clone` traits.
/// - Compatible with Serde for convenient serialization and deserialization of the data format.
///
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Bar {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "o")] pub open: f64,
    #[serde(rename = "h")] pub high: f64,
    #[serde(rename = "l")] pub low: f64,
    #[serde(rename = "c")] pub close: f64,
    #[serde(rename = "v")] pub volume: i64,
    #[serde(rename = "vw")] pub volume_weighted_avg_price: f64,
    #[serde(rename = "n")] pub number_of_trades: i64,
    #[serde(rename = "t")] pub timestamp: String,
}

/// Represents a trade correction, which includes details of both the original and corrected trades.
///
/// This struct is used to deserialize information about trade corrections from an external source,
/// providing fields for the symbol, exchange code, trade prices, trade sizes, associated conditions,
/// timestamps, and tape identifiers.
///
/// # Fields
///
/// * `symbol` (`String`):
///   The symbol or ticker of the security related to the trade correction.
///
/// * `exchange_code` (`String`):
///   The exchange code identifying where the trade was executed.
///
/// * `original_trade_id` (`String`):
///   The unique identifier of the original trade before the correction.
///
/// * `original_trade_price` (`f64`):
///   The price associated with the original trade.
///
/// * `original_trade_size` (`i64`):
///   The size or quantity of the original trade.
///
/// * `original_trade_conditions` (`Vec<String>`):
///   A vector of conditions or attributes associated with the original trade (e.g., trade type tags).
///
/// * `corrected_trade_id` (`String`):
///   The unique identifier of the corrected trade after the modification.
///
/// * `corrected_trade_price` (`f64`):
///   The updated price associated with the corrected trade.
///
/// * `corrected_trade_size` (`i64`):
///   The updated size or quantity of the corrected trade.
///
/// * `corrected_trade_conditions` (`Vec<String>`):
///   A vector of conditions or attributes associated with the corrected trade.
///
/// * `timestamp` (`String`):
///   The timestamp (in string format) when the correction was applied.
///
/// * `tape` (`String`):
///   The tape identifier (e.g., A, B, or C) indicating where the information originated.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TradeCorrections {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "x")] pub exchange_code: String,
    #[serde(rename = "oi")] pub original_trade_id: String,
    #[serde(rename = "op")] pub original_trade_price: f64,
    #[serde(rename = "os")] pub original_trade_size: i64,
    #[serde(rename = "oc")] pub original_trade_conditions: Vec<String>,
    #[serde(rename = "ci")] pub corrected_trade_id: String,
    #[serde(rename = "cp")] pub corrected_trade_price: f64,
    #[serde(rename = "cs")] pub corrected_trade_size: i64,
    #[serde(rename = "cc")] pub corrected_trade_conditions: Vec<String>,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "z")] pub tape: String,
}

/// A data structure representing trade cancels and error information.
///
/// This structure captures details such as the trading symbol, trade ID, exchange, price, size,
/// action, timestamp, and tape associated with the event.
///
/// The struct is both `Debug` and `Clone` enabled for debugging purposes and easy duplication.
/// It also derives `Deserialize` to allow the struct to be deserialized from formats like JSON
/// using Serde. Field names are deserialized based on custom mappings provided with the `#[serde(rename = ...)]` attribute.
///
/// Fields:
/// - `symbol` (`String`): Represents the trading symbol associated with the trade.
///   Serialized/Deserialized as "S".
/// - `trade_id` (`i64`): The unique ID of the trade.
///   Serialized/Deserialized as "i".
/// - `trade_exchange` (`String`): Indicates the exchange where the trade was executed.
///   Serialized/Deserialized as "x".
/// - `trade_price` (`f64`): The price at which the trade occurred.
///   Serialized/Deserialized as "p".
/// - `trade_size` (`i64`): The size or volume of the trade.
///   Serialized/Deserialized as "s".
/// - `action` (`String`): The action associated with the trade (e.g., canceled, error, etc.).
///   Serialized/Deserialized as "a".
/// - `timestamp` (`String`): The timestamp of when the trade occurred or the event happened.
///   Serialized/Deserialized as "t".
/// - `tape` (`String`): The tape indicator, providing additional information about the trade.
///   Serialized/Deserialized as "z".
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TradeCancelsAndErrors{
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "i")] pub trade_id: i64,
    #[serde(rename = "x")] pub trade_exchange: String,
    #[serde(rename = "p")] pub trade_price: f64,
    #[serde(rename = "s")] pub trade_size: i64,
    #[serde(rename = "a")] pub action: String,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "z")] pub ape: String,
}

/// Represents the Limit Up-Limit Down (LULD) details for a specific financial instrument.
///
/// The `LimitUpLimitDown` structure is used to capture data related to the LULD mechanism,
/// which is designed to prevent excessive volatility in financial markets by setting
/// upper and lower price limits for a given symbol.
///
/// # Fields
///
/// * `symbol` (*String*): The ticker symbol of the financial instrument.
///   - Serialized as "S" in the data payload.
///
/// * `limit_up_price` (*f64*): The upper price limit for the symbol.
///   - Serialized as "u" in the data payload.
///
/// * `limit_down_price` (*f64*): The lower price limit for the symbol.
///   - Serialized as "d" in the data payload.
///
/// * `indicator` (*String*): An indicator providing additional information about the limits,
///   such as the type of restriction in force.
///   - Serialized as "i" in the data payload.
///
/// * `timestamp` (*String*): The timestamp indicating when the LULD information was generated.
///   - Serialized as "t" in the data payload.
///
/// * `tape` (*String*): An identifier for the market tape or data source for this information.
///   - Serialized as "z" in the data payload.
///
/// # Derives
///
/// This struct supports the following derived traits:
///
/// * `Debug`: Facilitates formatting for debugging purposes.
/// * `Deserialize`: Enables JSON deserialization into this struct.
/// * `Clone`: Allows creating deep copies of `LimitUpLimitDown` instances.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LimitUpLimitDown{
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "u")] pub limit_up_price: f64,
    #[serde(rename = "d")] pub limit_down_price: f64,
    #[serde(rename = "i")] pub indicator: String,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "z")] pub tape: String,
}

/// Struct representing the trading status of a financial instrument.
///
/// This struct captures details about the trading status of a given instrument 
/// along with associated metadata such as the timestamp of the status and 
/// other descriptive codes and messages.
///
/// # Fields
///
/// * `symbol` (`String`): The financial instrument or stock symbol.
///   This field is mapped from the `S` key in the serialized data.
///
/// * `status_code` (`String`): A code representing the trading status.
///   This field is mapped from the `sc` key in the serialized data.
///
/// * `status_message` (`String`): A descriptive message associated with the trading status.
///   This field is mapped from the `sm` key in the serialized data.
///
/// * `reason_code` (`String`): A code that indicates the reason for the current trading status.
///   This field is mapped from the `rc` key in the serialized data.
///
/// * `reason_message` (`String`): A descriptive message explaining the reason for the trading status.
///   This field is mapped from the `rm` key in the serialized data.
///
/// * `timestamp` (`String`): A timestamp indicating when the trading status was recorded.
///   This field is mapped from the `t` key in the serialized data.
///
/// * `tape` (`String`): An identifier for the data source or market tape where the status was recorded.
///   This field is mapped from the `z` key in the serialized data.
///
/// # Traits
///
/// This struct derives the following traits:
/// * `Debug`: Allows the struct to be formatted using the `{:?}` formatter.
/// * `Deserialize`: Enables deserialization of the struct from formats supported by `serde`.
/// * `Clone`: Enables cloning of the struct to create duplicate instances.
/// * `Serialize`: Enables serialization of the struct to formats supported by `serde`.
///
/// # Examples
///
/// ```rust
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Clone, Serialize)]
/// pub struct TradingStatus {
///     #[serde(rename = "S")] pub symbol: String,
///     #[serde(rename = "sc")] pub status_code: String,
///     #[serde(rename = "sm")] pub status_message: String,
///     #[serde(rename = "rc")] pub reason_code: String,
///     #[serde(rename = "rm")] pub reason_message: String,
///     #[serde(rename = "t")] pub timestamp: String,
///     #[serde(rename = "z")] pub tape: String,
/// }
/// ```
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TradingStatus{
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "sc")] pub status_code: String,
    #[serde(rename = "sm")] pub status_message: String,
    #[serde(rename = "rc")] pub reason_code: String,
    #[serde(rename = "rm")] pub reason_message: String,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "z")] pub tape: String,
}

/// Represents an `OrderImbalances` structure that contains information about market order imbalances.
///
/// This structure is deserialized from external data (e.g., JSON) using Serde's `Deserialize` trait
/// and includes the following fields:
///
/// # Fields
/// - `symbol` (`String`): The market symbol or ticker associated with the order imbalance.
///   This field is deserialized from the `S` key.
/// - `price` (`f64`): The price value associated with the order imbalance.
///   This field is deserialized from the `p` key.
/// - `timestamp` (`String`): The timestamp indicating when the data was collected/recorded.
///   This field is deserialized from the `t` key.
/// - `tape` (`String`): The exchange or data tape identifier where the imbalance data is sourced from.
///   This field is deserialized from the `z` key.
///
/// The structure derives the following traits:
/// - `Debug`: For easy debugging and formatting.
/// - `Clone`: To allow cloning of `OrderImbalances` instances.
/// - `Deserialize`: To facilitate deserialization from structured data formats, such as JSON.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct OrderImbalances{
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "p")] pub price: f64,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "z")] pub tape: String,
}

/// Represents a message related to stock market data or administrative events.
///
/// The `StockMsg` enum is deserialized from a JSON object containing a `T` field,
/// which determines the type of message. Each variant corresponds to a specific
/// message type.
///
/// # Variants
///
/// ## Market Data
///
/// - `Trade(Trade)`:
///   Represents a trade message. Serialized with `"T": "t"`.
///
/// - `Quote(Quote)`:
///   Represents a quote message. Serialized with `"T": "q"`.
///
/// - `Bar(Bar)`:
///   Represents an aggregated time period (bar) message. Serialized with `"T": "b"`.
///
/// - `DailyBar(Bar)`:
///   Represents a daily bar message. Serialized with `"T": "d"`.
///
/// - `UpdatedBar(Bar)`:
///   Represents an updated bar message. Serialized with `"T": "u"`.
///
/// - `TradeCorrections(TradeCorrections)`:
///   Represents corrections to previously reported trades. Serialized with `"T": "c"`.
///
/// - `TradeCancelsAndErrors(TradeCancelsAndErrors)`:
///   Represents canceled or erroneous trades. Serialized with `"T": "x"`.
///
/// - `LimitUpLimitDown(LimitUpLimitDown)`:
///   Represents messages pertaining to Limit-Up/Limit-Down (LULD) events.
///   Serialized with `"T": "l"`.
///
/// - `TradingStatus(TradingStatus)`:
///   Represents trading status updates for securities. Serialized with `"T": "s"`.
///
/// - `OrderImbalances(OrderImbalances)`:
///   Represents messages about order imbalances. Serialized with `"T": "i"`.
///
/// ## Administrative
///
/// - `Subscription(SubscriptionAck)`:
///   Acknowledges a subscription request for specific data streams.
///   Serialized with `"T": "subscription"`.
///
/// - `Success(SuccessMsg)`:
///   Represents a success message, typically in response to a successful request.
///   Serialized with `"T": "success"`.
///
/// - `Error(ErrorMsg)`:
///   Represents an error message, usually in response to a failed or invalid request.
///   Serialized with `"T": "error"`.
///
/// # Serialization and Deserialization
///
/// Uses the `Serde` crate for deserialization and is tagged with a `T` field.
/// The `#[serde(rename = "...")]` attribute maps the variant to the expected
/// string identifier in the JSON data.
#[derive(Debug, Deserialize,Clone, Serialize)]
#[serde(tag = "T")]
pub enum StockMsg {
    // market data
    #[serde(rename = "t")] Trade(Trade),
    #[serde(rename = "q")] Quote(Quote),
    #[serde(rename = "b")] Bar(Bar),
    #[serde(rename = "d")] DailyBar(Bar),
    #[serde(rename = "u")] UpdatedBar(Bar),
    #[serde(rename = "c")] TradeCorrections(TradeCorrections),
    #[serde(rename = "x")] TradeCancelsAndErrors(TradeCancelsAndErrors),
    #[serde(rename = "l")] LimitUpLimitDown(LimitUpLimitDown),
    #[serde(rename = "s")] TradingStatus(TradingStatus),
    #[serde(rename = "i")] OrderImbalances(OrderImbalances),

    // administrative
    #[serde(rename = "subscription")] Subscription(SubscriptionAck),
    #[serde(rename = "success")] Success(SuccessMsg),
    #[serde(rename = "error")] Error(ErrorMsg),
    
}
/// Represents parameters required to configure a stock data stream.
///
/// This struct contains the endpoint, feed path, and subscription information
/// necessary to establish a connection to the stock data stream service (e.g., Alpaca).
/// It uses the `TypedBuilder` crate to easily create instances with default values
/// for the `endpoint` and `feed_path` fields.
///
/// # Fields
///
/// * `endpoint` (String):
///   - The WebSocket URL of the stock data stream server.
///   - Defaults to `"wss://stream.data.alpaca.markets/"`.
///   - Example: `"wss://stream.data.sandbox.alpaca.markets"`.
///
/// * `feed_path` (String):
///   - The subpath identifying the specific data feed to be accessed.
///   - Defaults to `"v2/iex"`.
///   - Examples of possible values:
///     - `"v2/iex"`: IEX data feed.
///     - `"v2/sip"`: SIP (Securities Information Processor) data feed.
///     - `"v2/delayed_sip"`: Delayed SIP feed.
///     - `"v1beta1/boats"`: Experimental "boats" data feed.
///     - `"v1beta1/overnight"`: Experimental overnight data feed.
///
/// * `subscription` (Subscribe):
///   - Defines specific subscription details (e.g., ticker symbols or channels)
///     for the stock data stream.
///   - This field is required and does not have a default value.
///
/// # Usage
///
/// ```
/// use rpaca::market_data::v2::stock_websocket::StockStreamParams;
/// use rpaca::market_data::v2::stock_websocket::Subscribe;
///
/// let params = StockStreamParams::builder()
///     .subscription(Subscribe::new(/* subscription details */))
///     .build();
///
/// println!("{:?}", params);
/// ```
#[derive(Debug, TypedBuilder, Serialize)]
pub struct StockStreamParams{
    #[builder(default = "wss://stream.data.alpaca.markets/".to_string())]
    pub endpoint: String, // e.g., "wss://stream.data.sandbox.alpaca.markets"
    #[builder(default = "v2/iex".to_string())]
    pub feed_path: String, // e.g., "v2/iex" | "v2/sip" | "v2/delayed_sip" | "v1beta1/boats" | "v1beta1/overnight"
    pub subscription: Subscribe,
}

/// Streams real-time stock data using WebSocket connectivity to the specified Alpaca endpoint.
///
/// This function establishes a WebSocket connection to the provided stock data feed endpoint,
/// handles authentication, subscribes to the desired streams, and continuously streams
/// messages back to the caller until the connection is closed or interrupted. If the connection
/// fails or is terminated, the function automatically attempts to reconnect using an exponential
/// backoff strategy.
///
/// # Parameters
///
/// - `alpaca`: A reference to an [`Alpaca`] instance containing the API key/secret required for
///   authentication.
/// - `params`: The [`StockStreamParams`] struct specifying the endpoint URL, feed path, and the
///   desired subscription actions.
///
/// # Returns
///
/// Returns a `Result` wrapping an async [`Stream`], where each item is either:
/// - `Ok(StockMsg)`: A successfully received stock message (such as trade, quote, or other events).
/// - `Err(anyhow::Error)`: An error that occurred during the streaming process, such as connection
///   issues or decoding failures.
///
/// # Behavior
///
/// 1. The function opens a WebSocket connection to the specified feed path.
/// 2. Authenticates the connection with the given API key and secret.
/// 3. Subscribes to the requested stock data streams using the subscription actions provided
///    in `params`.
/// 4. Continuously listens for incoming messages and forwards them to the consumer via a
///    channel-backed [`Stream`].
/// 5. Automatically reconnects on failure with an exponentially increasing backoff up to a maximum limit.
///
/// # Errors
///
/// The function returns an error in the following scenarios:
/// - WebSocket connection failures (e.g., unreachable endpoint, network disruptions).
/// - Authentication errors (e.g., invalid API key or secret).
/// - Decoding issues when parsing incoming messages as [`StockMsg`].
///
/// # Reconnection
///
/// If the connection fails (e.g., due to network errors or server-side issues), the function
/// attempts to reconnect with an exponential backoff (up to 6 retries, capping at approximately
/// 16 seconds between attempts). The stream continues to emit data seamlessly if reconnected
/// successfully.
///
/// # Notes
///
/// - The connection remains active and streams data until interrupted or closed by the client/server.
/// - The function uses [`tokio::sync::mpsc`] for channel-based communication and wraps the receiver
///   with a [`tokio_stream::wrappers::ReceiverStream`] for consumption.
///
/// [`Alpaca`]: struct.Alpaca.html
/// [`StockStreamParams`]: struct.StockStreamParams.html
/// [`Stream`]: https://docs.rs/futures-core/latest/futures_core/stream/trait.Stream.html
/// [`StockMsg`]: enum.StockMsg.html
pub async fn stream_stock_data(
    alpaca: &Alpaca,
    params: StockStreamParams,
) -> Result<impl futures_core::Stream<Item = Result<StockMsg>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<StockMsg>>(1024);

    let endpoint = params.endpoint.to_string();
    let feed_path = params.feed_path.to_string();
    let key = alpaca.apca_api_key_id.clone();
    let secret = alpaca.apca_api_secret_key.clone();
    let subscribe_json = params.subscription.action_json();

    tokio::spawn(async move {
        let mut attempt: u32 = 0;

        loop {
            let url = format!("{}/{}", endpoint.trim_end_matches('/'), feed_path);
            let conn = connect_async(&url).await;

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
async fn test_stock_ws(){
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();

    let mut stream = stream_stock_data(&alpaca, StockStreamParams::builder()
        .endpoint("wss://stream.data.alpaca.markets/".to_string())
        .feed_path("v2/test".to_string())
        .subscription(Subscribe{
            trades: vec!["FAKEPACA".to_string()],
            quotes: vec!["FAKEPACA".to_string()],
            bars: vec!["FAKEPACA".to_string()],
            ..Default::default()
        }).build()).await.unwrap();

    let mut got_trade = false;
    let mut got_quote = false;
    let mut got_bar = false;
    let mut got_ack = false;

    let _ = timeout(Duration::from_secs(90), async {
        while let Some(item) = stream.next().await {
            match item.as_ref().unwrap() {
                    StockMsg::Subscription(ack) => {
                        assert!(ack.trades.contains(&"FAKEPACA".to_string()));
                        got_ack = true;
                    }
                    StockMsg::Trade(t) => {
                        assert_eq!(t.symbol, "FAKEPACA");
                        got_trade = true;
                    }
                    StockMsg::Quote(q) => {
                        assert_eq!(q.symbol, "FAKEPACA");
                        assert!(q.ask_price > 0.0 && q.bid_price > 0.0);
                        got_quote = true;
                    }
                    StockMsg::Bar(b) => {
                        assert_eq!(b.symbol, "FAKEPACA");
                        got_bar = true;
                    }
                    _ => {println!("Got unknown item: {item:?}");}
            }

            if got_ack && got_trade && got_quote && got_bar {
                break;
            }
        }
    }).await;

    assert!(got_ack, "did not receive subscription ack");
    assert!(got_trade, "did not receive trade");
    assert!(got_quote, "did not receive quote");
    assert!(got_bar, "did not receive bar");
}

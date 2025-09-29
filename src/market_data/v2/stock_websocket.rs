use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::Utf8Bytes;
use typed_builder::TypedBuilder;
use crate::auth::{Alpaca, TradingType};

/// Build the subscription payload for Alpaca's data stream.
/// Only non-empty vectors are serialized.
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
    pub fn new() -> Self {
        Self::default()
    }
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

/// Control/ack payloads
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct SuccessMsg {
    pub msg: Option<String>,
    pub code: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ErrorMsg {
    pub msg: Option<String>,
    pub code: Option<i64>,
}

/// Market data payloads (minimal fields—add more as needed)
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct TradeCorrections {
    #[serde(rename = "S")] symbol: String,
    #[serde(rename = "x")] exchange_code: String,
    #[serde(rename = "oi")] original_trade_id: String,
    #[serde(rename = "op")] original_trade_price: f64,
    #[serde(rename = "os")] original_trade_size: i64,
    #[serde(rename = "oc")] original_trade_conditions: Vec<String>,
    #[serde(rename = "ci")] corrected_trade_id: String,
    #[serde(rename = "cp")] corrected_trade_price: f64,
    #[serde(rename = "cs")] corrected_trade_size: i64,
    #[serde(rename = "cc")] corrected_trade_conditions: Vec<String>,
    #[serde(rename = "t")] timestamp: String,
    #[serde(rename = "z")] tape: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TradeCancelsAndErrors{
    #[serde(rename = "S")] symbol: String,
    #[serde(rename = "i")] trade_id: i64,
    #[serde(rename = "x")] trade_exchange: String,
    #[serde(rename = "p")] trade_price: f64,
    #[serde(rename = "s")] trade_size: i64,
    #[serde(rename = "a")] action: String,
    #[serde(rename = "t")] timestamp: String,
    #[serde(rename = "z")] tape: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitUpLimitDown{
    #[serde(rename = "S")] symbol: String,
    #[serde(rename = "u")] limit_up_price: f64,
    #[serde(rename = "d")] limit_down_price: f64,
    #[serde(rename = "i")] indicator: String,
    #[serde(rename = "t")] timestamp: String,
    #[serde(rename = "z")] tape: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TradingStatus{
    #[serde(rename = "S")] symbol: String,
    #[serde(rename = "sc")] status_code: String,
    #[serde(rename = "sm")] status_message: String,
    #[serde(rename = "rc")] reason_code: String,
    #[serde(rename = "rm")] reason_message: String,
    #[serde(rename = "t")] timestamp: String,
    #[serde(rename = "z")] tape: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderImbalances{
    #[serde(rename = "S")] symbol: String,
    #[serde(rename = "p")] price: f64,
    #[serde(rename = "t")] timestamp: String,
    #[serde(rename = "z")] tape: String,
}

/// All incoming messages are arrays of JSON objects with a "T" tag.
/// We tag on `T` to deserialize directly into variants.
#[derive(Debug, Deserialize,Clone)]
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
#[derive(Debug, TypedBuilder, Serialize)]
pub struct StockStreamParams{
    #[builder(default = "wss://stream.data.alpaca.markets/".to_string())]
    pub endpoint: String, // e.g., "wss://stream.data.sandbox.alpaca.markets"
    #[builder(default = "v2/iex".to_string())]
    pub feed_path: String, // e.g., "v2/iex" | "v2/sip" | "v2/delayed_sip" | "v1beta1/boats" | "v1beta1/overnight"
    pub subscription: Subscribe,
}

/// Connect, auth, subscribe, stream messages. Reconnects on drop/errors and resubscribes.
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

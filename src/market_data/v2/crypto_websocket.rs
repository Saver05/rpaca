use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tungstenite::Utf8Bytes;
use typed_builder::TypedBuilder;
use crate::auth::{Alpaca, TradingType};

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
            "orderbooks": self.orderbooks,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubscriptionAck {
    #[serde(default)] pub trades: Vec<String>,
    #[serde(default)] pub quotes: Vec<String>,
    #[serde(default)] pub bars: Vec<String>,
    #[serde(rename = "dailyBars", default)] pub daily_bars: Vec<String>,
    #[serde(rename = "updatedBars", default)] pub updated_bars: Vec<String>,
    #[serde(default)] pub orderbooks: Vec<String>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Trade {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "p")] pub price: f64,
    #[serde(rename = "s")] pub size: f64,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "i")] pub trade_id: NumF64,
    #[serde(rename = "tks")] pub taker_side: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Quote{
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "bp")] pub bid_price: f64,
    #[serde(rename = "bs")] pub bid_size: f64,
    #[serde(rename = "ap")] pub ask_price: f64,
    #[serde(rename = "as")] pub ask_size: f64,
    #[serde(rename = "t")] pub timestamp: String,
}

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

#[derive(Debug, Deserialize, Clone)]
pub struct Level {
    pub p: f64,
    pub s: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Orderbook {
    #[serde(rename = "S")] pub symbol: String,
    #[serde(rename = "t")] pub timestamp: String,
    #[serde(rename = "b")] pub bids: Vec<Level>,
    #[serde(rename = "a")] pub asks: Vec<Level>,
    #[serde(rename = "r")] pub reset: Option<bool>,
}

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

#[derive(Debug, TypedBuilder, Serialize)]
pub struct CryptoStreamParams{
    #[builder(default = "wss://stream.data.alpaca.markets/v1beta3/crypto/us".to_string())]
    pub endpoint: String, // e.g., "wss://stream.data.sandbox.alpaca.markets"
    pub subscription: Subscribe,
}

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
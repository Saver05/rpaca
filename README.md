# 📦 rpaca – A Rust Client for the Alpaca Trading API

`rpaca` is a lightweight, idiomatic Rust client for interacting with the [Alpaca](https://alpaca.markets) trading API.
It enables easy integration with account info, orders, assets, positions, and more all with strong typing and clean
abstractions.

---


This project is currently in development breaking changes may happen.

## ✨ Features

- ✅ Access account information and configurations
- ✅ Submit and manage stock
- ✅ View and manage portfolios and positions
- ✅ Fetch market calendar and clock data
- ✅ Work with watchlists
- 🛠️ Access Crypto Funding Endpoints
- 🚧 Any Market Data or Broker Endpoints (Coming Soon)

---

## Getting Started

### Add to your `Cargo.toml`

```toml
[dependencies]
rpaca = "0.5.0"  
```

Getting basic account information

```
use rpaca::auth::{Alpaca, TradingType};
use rpaca::trading::v2::get_account_info::get_account_info;

#[tokio::main]
async fn main()  {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_account_info(&alpaca).await {
        Ok(account) => println!("Account: {account:?}"),
        Err(e) => println!("Error: {e:?}"),
    }
}
```  

Creating an order

```
use rpaca::auth::{Alpaca, TradingType};
use rpaca::trading::v2::orders::{create_order, OrderRequest};

#[tokio::main]
async fn main()  {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    let order = OrderRequest{
        symbol: "AAPL".to_string(),
        qty: Some("1".to_string()),
        notional: None,
        side: "buy".to_string(),
        order_type: "market".to_string(),
        time_in_force: "day".to_string(),
        limit_price: None,
        stop_price: None,
        trail_price: None,
        trail_percent: None,
        extended_hours: None,
        client_order_id: None,
        order_class: None,
        legs: None,
        take_profit: None,
        stop_loss: None,
    };
    match create_order(&alpaca, order).await {
        Ok(order) =>{
            println!("Order created: {:?}", order);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

```
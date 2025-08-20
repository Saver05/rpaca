# 📦 rpaca – A Rust Client for the Alpaca Trading API

`rpaca` is a lightweight, idiomatic Rust client for interacting with the [Alpaca](https://alpaca.markets) trading API.
It enables easy integration with account info, orders, assets, positions, and market data with strong typing and clean
abstractions.

[![Crates.io](https://img.shields.io/crates/v/rpaca)](https://crates.io/crates/rpaca)
[![Documentation](https://docs.rs/rpaca/badge.svg)](https://docs.rs/rpaca)
[![License](https://img.shields.io/crates/l/rpaca)](https://github.com/Saver05/rpaca/blob/main/LICENSE)

## 🚀 Overview

`rpaca` provides a Rust interface to the Alpaca API, allowing developers to:

- Access and manage trading accounts
- Submit and track orders for stocks and other assets
- Monitor positions and portfolio performance
- Access market data for stocks and options
- Work with watchlists and other account features

The library supports both paper trading (for testing) and live trading environments.

## ✨ Features

### Trading

- ✅ **Authentication** - Simple API key authentication with support for environment variables
- ✅ **Account Management** - Access account information and configurations
- ✅ **Order Management** - Submit and manage stock orders with various order types
- ✅ **Portfolio Management** - View and manage positions and overall portfolio
- ✅ **Market Data** - Access stock and options data
- ✅ **Calendar & Clock** - Fetch market calendar and clock data
- ✅ **Watchlists** - Create and manage watchlists
- 🚧 **Crypto** - Access to cryptocurrency endpoints (in development)

### Market Data

- ✅ **Stock** - Get all stock information
- 🚧 Everything else work in progress

## 📋 Installation

Add `rpaca` to your `Cargo.toml`:

```toml
[dependencies]
rpaca = "0.5.0"
```

## 🔑 Authentication

`rpaca` requires Alpaca API credentials. You can create an account at [Alpaca](https://alpaca.markets/) to obtain your
API key and secret.

### Using Environment Variables

Create a `.env` file in your project root:

```
APCA_API_KEY_ID=your_api_key
APCA_API_SECRET_KEY=your_api_secret
```

Then in your code:

```rust
use rpaca::auth::{Alpaca, TradingType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For paper trading (recommended for testing)
    let alpaca = Alpaca::from_env(TradingType::Paper)?;
    
    // For live trading
    // let alpaca = Alpaca::from_env(TradingType::Live)?;
    
    Ok(())
}
```

### Direct Authentication

```rust
use rpaca::auth::{Alpaca, TradingType};

fn main() {
    let client = Alpaca::new(
        "your_api_key".to_string(),
        "your_api_secret".to_string(),
        TradingType::Paper
    );
}
```

## 📝 Examples

### Getting Account Information

```rust
use rpaca::auth::{Alpaca, TradingType};
use rpaca::trading::v2::get_account_info::get_account_info;

#[tokio::main]
async fn main() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    
    match get_account_info(&alpaca).await {
        Ok(account) => println!("Account: {account:?}"),
        Err(e) => println!("Error: {e:?}"),
    }
}
```

### Creating an Order

```rust
use rpaca::auth::{Alpaca, TradingType};
use rpaca::trading::v2::orders::{create_order, OrderRequest};

#[tokio::main]
async fn main() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    
    match create_order(&alpaca, OrderRequest::builder()
        .symbol("AAPL")
        .qty("1")
        .side("buy")
        .order_type("market")
        .time_in_force("day")
        .build()).await {
        Ok(order) => {
            println!("Order created: {:?}", order);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
```

### Getting Market Data

```rust
use rpaca::auth::{Alpaca, TradingType};
use rpaca::market_data::v2::stock::get_bars;
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    let end = Utc::now();
    let start = end - Duration::days(7);

    match get_bars(&alpaca, vec!["AAPL".to_string()], "1Day", start, end, 100).await {
        Ok(bars) => println!("Bars: {:?}", bars),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## 📚 Documentation

For detailed documentation, visit [docs.rs/rpaca](https://docs.rs/rpaca).

## 🛠️ Development Status

This project is currently in active development. While the core functionality is stable, breaking changes may occur in
future versions as we continue to expand the API coverage.

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests on
the [GitHub repository](https://github.com/Saver05/rpaca).

## 📜 License

This project is licensed under either of:

- [MIT License](https://opensource.org/licenses/MIT)
- [Apache License, Version 2.0](https://opensource.org/licenses/Apache-2.0)

at your option.
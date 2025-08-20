#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

//! # rpaca
//!
//! A Rust client library for the Alpaca trading API.
//!
//! This library provides a convenient interface for interacting with the Alpaca API,
//! allowing users to access market data and execute trades programmatically.
//!
//! ## Features
//!
//! - Authentication with Alpaca API keys
//! - Support for both paper trading and live trading environments
//! - Access to market data (stocks, options)
//! - Trading functionality (orders, positions, account management)
//!
//! ## Example
//!
//! ```rust,no_run
//! use rpaca::auth::{Alpaca, TradingType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new Alpaca client using environment variables
//!     let client = Alpaca::from_env(TradingType::Paper)?;
//!     
//!     // Use the client to interact with the Alpaca API
//!     // ...
//!     
//!     Ok(())
//! }
//! ```

/// Authentication module for Alpaca API
pub mod auth;

/// Market data module for accessing stock and option information
pub mod market_data;

/// Internal request handling module
mod request;

/// Trading module for managing orders, positions, and account information
pub mod trading;

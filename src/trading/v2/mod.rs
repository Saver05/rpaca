//! Trading v2 API module.
//!
//! This module contains implementations for the v2 version of Alpaca's trading API,
//! providing access to account information, orders, positions, and other trading functionality.
//! It includes endpoints for managing all aspects of trading with Alpaca.

pub mod account_activities;
pub mod account_configurations;
pub mod assets;
pub mod calendar;
pub mod clock;
pub mod crypto_funding;
pub mod get_account_info;
pub mod orders;
pub mod portfolio;
pub mod positions;
pub mod watchlists;

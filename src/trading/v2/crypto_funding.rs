use crate::auth::Alpaca;
use crate::request::create_trading_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;
#[derive(Debug, Deserialize)]
pub struct Wallet {
    pub chain: String,
    pub address: String,
    pub created_at: String,
}
/// Retrieves crypto wallet information for a specific asset.
///
/// This function fetches information about a crypto wallet associated with the specified asset,
/// including the blockchain address and creation timestamp.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `asset` - The cryptocurrency asset symbol (e.g., "BTC")
///
/// # Returns
/// * `Result<Wallet, Box<dyn std::error::Error>>` - The wallet information or an error
pub async fn retrieve_crypto_wallets(
    alpaca: &Alpaca,
    asset: String,
) -> Result<Wallet, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/wallets?asset={}", asset);
    let response = create_trading_request::<()>(alpaca, Method::GET, &*endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting wallet failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[derive(Debug, Deserialize)]
pub struct CryptoTransfers {
    pub id: Uuid,
    pub tx_hash: String,
    pub direction: String,
    pub status: String,
    pub amount: String,
    pub usd_value: String,
    pub network_fee: String,
    pub fees: String,
    pub chain: String,
    pub asset: String,
    pub from_address: String,
    pub to_address: String,
    pub created_at: String,
}

/// Retrieves a list of all crypto transfers for the account.
///
/// This function fetches information about all cryptocurrency transfers associated with the account,
/// including deposits, withdrawals, and their statuses.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
///
/// # Returns
/// * `Result<Vec<CryptoTransfers>, Box<dyn std::error::Error>>` - A list of crypto transfers or an error
pub async fn retrieve_crypto_transfers(
    alpaca: &Alpaca,
) -> Result<Vec<CryptoTransfers>, Box<dyn std::error::Error>> {
    let response =
        create_trading_request::<()>(alpaca, Method::GET, "/v2/wallets/transfers", None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to get crypto transfers: {text}").into());
    }
    Ok(response.json().await?)
}

#[derive(Debug, Serialize, TypedBuilder)]
pub struct CryptoWithdrawalParams {
    pub amount: String,
    pub address: String,
    pub asset: String,
}

/// Requests a cryptocurrency withdrawal from the account.
///
/// This function initiates a withdrawal of cryptocurrency from the account to an external address.
/// The withdrawal request will be processed according to Alpaca's security and compliance procedures.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `params` - Parameters for the withdrawal including amount, destination address, and asset
///
/// # Returns
/// * `Result<CryptoTransfers, Box<dyn std::error::Error>>` - Information about the withdrawal request or an error
pub async fn request_withdrawl(
    alpaca: &Alpaca,
    params: CryptoWithdrawalParams,
) -> Result<CryptoTransfers, Box<dyn std::error::Error>> {
    let response =
        create_trading_request(alpaca, Method::POST, "/v2/wallets/transfers", Some(params)).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to create withdrawl: {text}").into());
    }
    Ok(response.json().await?)
}

/// Retrieves information about a specific crypto transfer by its ID.
///
/// This function fetches detailed information about a single cryptocurrency transfer
/// identified by its unique transfer ID.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `transfer_id` - The unique identifier of the transfer to retrieve
///
/// # Returns
/// * `Result<CryptoTransfers, Box<dyn std::error::Error>>` - Information about the specific transfer or an error
pub async fn retrieve_crypto_transfer(
    alpaca: &Alpaca,
    transfer_id: String,
) -> Result<CryptoTransfers, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/wallets/transfers/{transfer_id}");
    let response = create_trading_request::<()>(alpaca, Method::GET, &*endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to get transfer: {text}").into());
    }
    Ok(response.json().await?)
}

#[derive(Debug, Deserialize)]
pub struct WhitelistedAddresses {
    pub id: String,
    pub chain: String,
    pub asset: String,
    pub address: String,
    pub status: String,
    pub created_at: String,
}

/// Retrieves a list of whitelisted cryptocurrency addresses for the account.
///
/// This function fetches all cryptocurrency addresses that have been whitelisted for withdrawals
/// from the account. Only addresses on this list can be used as destinations for withdrawals.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
///
/// # Returns
/// * `Result<Vec<WhitelistedAddresses>, Box<dyn std::error::Error>>` - A list of whitelisted addresses or an error
pub async fn get_whitelisted_addresses(
    alpaca: &Alpaca,
) -> Result<Vec<WhitelistedAddresses>, Box<dyn std::error::Error>> {
    let response =
        create_trading_request::<()>(alpaca, Method::GET, "/v2/wallets/whitelists", None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to get whitelisted addresses: {text}").into());
    }
    Ok(response.json().await?)
}

#[derive(Debug, Serialize, TypedBuilder)]
pub struct AddWhitelistedAddressParams {
    pub address: String,
    pub asset: String,
}

/// Adds a new cryptocurrency address to the whitelist.
///
/// This function adds a new cryptocurrency address to the account's whitelist,
/// allowing it to be used as a destination for future withdrawals. The address
/// will need to go through Alpaca's verification process before it can be used.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `params` - Parameters including the address to whitelist and the associated asset
///
/// # Returns
/// * `Result<WhitelistedAddresses, Box<dyn std::error::Error>>` - Information about the newly whitelisted address or an error
pub async fn add_whitelisted_address(
    alpaca: &Alpaca,
    params: AddWhitelistedAddressParams,
) -> Result<WhitelistedAddresses, Box<dyn std::error::Error>> {
    let response =
        create_trading_request(alpaca, Method::POST, "/v2/wallets/whitelists", Some(params))
            .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to add whitelisted address: {text}").into());
    }
    Ok(response.json().await?)
}

/// Removes a cryptocurrency address from the whitelist.
///
/// This function deletes a previously whitelisted cryptocurrency address from the account's
/// whitelist, preventing it from being used for future withdrawals.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `address_id` - The unique identifier of the whitelisted address to remove
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success (empty result) or an error
pub async fn delete_whitelisted_address(
    alpaca: &Alpaca,
    address_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = create_trading_request::<()>(
        alpaca,
        Method::DELETE,
        &format!("/v2/wallets/whitelists/{address_id}"),
        None,
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to delete whitelisted address: {text}").into());
    }
    Ok(())
}

#[derive(Debug, Serialize, TypedBuilder)]
pub struct EstimatedGasFeeParams {
    pub asset: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct EstimatedGasFee {
    pub fee: String,
}

/// Retrieves an estimate of the gas fee for a cryptocurrency transfer.
///
/// This function calculates the estimated gas fee (network transaction fee) for a
/// cryptocurrency transfer with the specified parameters. This is useful for planning
/// transfers and understanding the associated costs.
///
/// # Arguments
/// * `alpaca` - The Alpaca client instance with authentication information
/// * `params` - Parameters for the transfer including asset, addresses, and amount
///
/// # Returns
/// * `Result<EstimatedGasFee, Box<dyn std::error::Error>>` - The estimated gas fee or an error
pub async fn get_estimated_gas_fee(
    alpaca: &Alpaca,
    params: EstimatedGasFeeParams,
) -> Result<EstimatedGasFee, Box<dyn std::error::Error>> {
    let query = serde_urlencoded::to_string(&params)?;
    let response = create_trading_request::<()>(
        alpaca,
        Method::GET,
        &format!("/v2/wallets/fees/estimate?{query}"),
        None,
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to get estimated gas fee: {text}").into());
    }
    Ok(response.json().await?)
}

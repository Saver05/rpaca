use crate::auth::Alpaca;
use crate::request::create_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;
#[derive(Debug, Deserialize)]
pub struct Wallet {
    chain: String,
    address: String,
    created_at: String,
}
pub async fn retrieve_crypto_wallets(
    alpaca: &Alpaca,
    asset: String,
) -> Result<Wallet, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/wallets?asset={}", asset);
    let response = create_request::<()>(alpaca, Method::GET, &*endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting wallet failed: {text}").into());
    }
    Ok(response.json().await?)
}

#[derive(Debug, Deserialize)]
pub struct CryptoTransfers {
    id: Uuid,
    tx_hash: String,
    direction: String,
    status: String,
    amount: String,
    usd_value: String,
    network_fee: String,
    fees: String,
    chain: String,
    asset: String,
    from_address: String,
    to_address: String,
    created_at: String,
}

pub async fn retrieve_crypto_transfers(
    alpaca: &Alpaca,
) -> Result<Vec<CryptoTransfers>, Box<dyn std::error::Error>> {
    let response = create_request::<()>(alpaca, Method::GET, "/v2/wallets/transfers", None).await?;
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

pub async fn request_withdrawl(
    alpaca: &Alpaca,
    params: CryptoWithdrawalParams,
) -> Result<CryptoTransfers, Box<dyn std::error::Error>> {
    let response =
        create_request(alpaca, Method::POST, "/v2/wallets/transfers", Some(params)).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to create withdrawl: {text}").into());
    }
    Ok(response.json().await?)
}

pub async fn retrieve_crypto_transfer(
    alpaca: &Alpaca,
    transfer_id: String,
) -> Result<CryptoTransfers, Box<dyn std::error::Error>> {
    let endpoint = format!("/v2/wallets/transfers/{transfer_id}");
    let response = create_request::<()>(alpaca, Method::GET, &*endpoint, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to get transfer: {text}").into());
    }
    Ok(response.json().await?)
}

#[derive(Debug, Deserialize)]
pub struct WhitelistedAddresses {
    id: String,
    chain: String,
    asset: String,
    address: String,
    status: String,
    created_at: String,
}

pub async fn get_whitelisted_addresses(
    alpaca: &Alpaca,
) -> Result<Vec<WhitelistedAddresses>, Box<dyn std::error::Error>> {
    let response =
        create_request::<()>(alpaca, Method::GET, "/v2/wallets/whitelists", None).await?;
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

pub async fn add_whitelisted_address(
    alpaca: &Alpaca,
    params: AddWhitelistedAddressParams,
) -> Result<WhitelistedAddresses, Box<dyn std::error::Error>> {
    let response =
        create_request(alpaca, Method::POST, "/v2/wallets/whitelists", Some(params)).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to add whitelisted address: {text}").into());
    }
    Ok(response.json().await?)
}

pub async fn delete_whitelisted_address(
    alpaca: &Alpaca,
    address_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = create_request::<()>(
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

pub async fn get_estimated_gas_fee(
    alpaca: &Alpaca,
    params: EstimatedGasFeeParams,
) -> Result<EstimatedGasFee, Box<dyn std::error::Error>> {
    let query = serde_urlencoded::to_string(&params)?;
    let response = create_request::<()>(
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

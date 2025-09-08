use crate::auth::{Alpaca, TradingType};
use crate::request::create_trading_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use uuid::Uuid;
#[derive(Debug, Deserialize)]
pub struct WatchlistNoAssets {
    pub id: Uuid,
    pub account_id: Uuid,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
}

pub async fn get_watchlists(
    alpaca: &Alpaca,
) -> Result<Vec<WatchlistNoAssets>, Box<dyn std::error::Error>> {
    let response =
        create_trading_request::<()>(alpaca, Method::GET, "/v2/watchlists", None).await?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting all watchlists failed: {}", text).into());
    };

    Ok(response.json().await?)
}

#[derive(Debug, Serialize, TypedBuilder)]
pub struct CreateWatchlistParams {
    pub name: String,
    #[builder(default, setter(strip_option))]
    pub symbols: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct WatchlistAssets {
    pub id: Uuid,
    pub account_id: Uuid,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub assets: Vec<Asset>,
}
use crate::trading::v2::assets::Asset;
use serde::de::Deserializer;
use typed_builder::TypedBuilder;

fn null_to_empty_vec<'de, D>(deserializer: D) -> Result<Vec<Asset>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub async fn create_watchlist(
    alpaca: &Alpaca,
    params: CreateWatchlistParams,
) -> Result<WatchlistAssets, Box<dyn std::error::Error>> {
    let response = create_trading_request::<CreateWatchlistParams>(
        alpaca,
        Method::POST,
        "/v2/watchlists",
        Some(params),
    )
    .await?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Creating watchlist failed: {}", text).into());
    };
    Ok(response.json().await?)
}

pub async fn get_watchlist_by_id(
    alpaca: &Alpaca,
    id: Uuid,
) -> Result<WatchlistAssets, Box<dyn std::error::Error>> {
    let response =
        create_trading_request::<()>(alpaca, Method::GET, &format!("/v2/watchlists/{}", id), None)
            .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting watchlist by id failed: {}", text).into());
    };
    Ok(response.json().await?)
}

#[derive(Debug, Serialize, TypedBuilder)]
pub struct UpdateWatchlistParams {
    pub name: String,
    #[builder(default, setter(strip_option))]
    pub symbols: Option<Vec<String>>,
}

pub async fn update_watchlist_by_id(
    alpaca: &Alpaca,
    watchlist_id: Uuid,
    params: UpdateWatchlistParams,
) -> Result<WatchlistAssets, Box<dyn std::error::Error>> {
    let response = create_trading_request::<UpdateWatchlistParams>(
        alpaca,
        Method::PUT,
        &format!("/v2/watchlists/{}", watchlist_id),
        Some(params),
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Updating watchlist by id failed: {}", text).into());
    };
    Ok(response.json().await?)
}

pub async fn add_asset_to_watchlist(
    alpaca: &Alpaca,
    watchlist_id: Uuid,
    symbol: String,
) -> Result<WatchlistAssets, Box<dyn std::error::Error>> {
    let response = create_trading_request(
        alpaca,
        Method::POST,
        &format!("/v2/watchlists/{}", watchlist_id),
        Some(json!({ "symbol": symbol })),
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Adding asset to watchlist failed: {}", text).into());
    };
    let body = response.text().await?;

    let parsed: WatchlistAssets = from_str(&body)?;
    Ok(parsed)
}

pub async fn delete_watchlist_by_id(
    alpaca: &Alpaca,
    watchlist_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = create_trading_request::<()>(
        alpaca,
        Method::DELETE,
        &format!("/v2/watchlists/{}", watchlist_id),
        None,
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Deleting watchlist failed: {}", text).into());
    };
    Ok(())
}

pub async fn get_watchlist_by_name(
    alpaca: &Alpaca,
    name: String,
) -> Result<WatchlistAssets, Box<dyn std::error::Error>> {
    let response = create_trading_request::<()>(
        alpaca,
        Method::GET,
        &format!("/v2/watchlists:by_name?name={}", name),
        None,
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting watchlist by name failed: {}", text).into());
    };
    Ok(response.json().await?)
}

pub async fn update_watchlist_by_name(
    alpaca: &Alpaca,
    name: String,
    params: UpdateWatchlistParams,
) -> Result<WatchlistAssets, Box<dyn std::error::Error>> {
    let response = create_trading_request::<UpdateWatchlistParams>(
        alpaca,
        Method::PUT,
        &format!("/v2/watchlists:by_name?name={}", name),
        Some(params),
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Updating watchlist by name failed: {}", text).into());
    };
    Ok(response.json().await?)
}

pub async fn add_asset_to_watchlist_by_name(
    alpaca: &Alpaca,
    name: String,
    symbol: String,
) -> Result<WatchlistAssets, Box<dyn std::error::Error>> {
    let response = create_trading_request(
        alpaca,
        Method::POST,
        &format!("/v2/watchlists:by_name?name={}", name),
        Some(json!({ "symbol": symbol })),
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Adding asset to watchlist by name failed: {}", text).into());
    };
    Ok(response.json().await?)
}

pub async fn delete_watchlist_by_name(
    alpaca: &Alpaca,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = create_trading_request::<()>(
        alpaca,
        Method::DELETE,
        &format!("/v2/watchlists:by_name?name={}", name),
        None,
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Deleting watachlist by name failed: {}", text).into());
    };
    Ok(())
}

pub async fn delete_symbol_from_watchlist(
    alpaca: &Alpaca,
    watchlist_id: Uuid,
    symbol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = create_trading_request::<()>(
        alpaca,
        Method::DELETE,
        &format!("/v2/watchlists/{}/{}", watchlist_id, symbol),
        None,
    )
    .await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Deleting symbol from watchlist failed: {}", text).into());
    };
    Ok(response.json().await?)
}

pub async fn delete_all_watchlists(alpaca: &Alpaca) -> Result<(), Box<dyn std::error::Error>> {
    let watchlists = get_watchlists(alpaca).await?;
    for watchlist in watchlists {
        if let Err(e) = delete_watchlist_by_id(alpaca, watchlist.id).await {
            eprintln!("Error deleting watchlist: {e}");
        }
    }
    Ok(())
}
#[tokio::test]
async fn test_watchlists() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    match get_watchlists(&alpaca).await {
        Ok(watchlists) => {
            assert_eq!(watchlists.len(), 0)
        }
        Err(e) => panic!("Error getting watchlists: {}", e),
    }
    let watchlist = create_watchlist(
        &alpaca,
        CreateWatchlistParams::builder()
            .name("test".to_string())
            .build(),
    )
    .await
    .unwrap();
    assert_eq!(watchlist.name, "test");
    let watchlist = get_watchlist_by_id(&alpaca, watchlist.id).await.unwrap();
    assert_eq!(watchlist.name, "test");
    let watchlist = get_watchlist_by_name(&alpaca, "test".to_string())
        .await
        .unwrap();
    assert_eq!(watchlist.name, "test");
    let watchlist = update_watchlist_by_id(
        &alpaca,
        watchlist.id,
        UpdateWatchlistParams::builder()
            .name("test2".to_string())
            .build(),
    )
    .await
    .unwrap();
    assert_eq!(watchlist.name, "test2");
    let watchlist = update_watchlist_by_name(
        &alpaca,
        "test2".to_string(),
        UpdateWatchlistParams::builder()
            .name("test3".to_string())
            .build(),
    )
    .await
    .unwrap();
    assert_eq!(watchlist.name, "test3");
    let watchlist = match add_asset_to_watchlist(&alpaca, watchlist.id, "AAPL".to_string()).await {
        Ok(watchlist) => watchlist,
        Err(e) => panic!("Error adding asset to watchlist: {}", e),
    };
    assert_eq!(watchlist.assets.len(), 1);
    let watchlist =
        add_asset_to_watchlist_by_name(&alpaca, "test3".to_string(), "GOOG".to_string())
            .await
            .unwrap();
    assert_eq!(watchlist.assets.len(), 2);
    let watchlists = get_watchlists(&alpaca).await.unwrap();
    assert_eq!(watchlists.len(), 1);
    delete_watchlist_by_id(&alpaca, watchlist.id).await.unwrap();
    match get_watchlists(&alpaca).await {
        Ok(watchlists) => assert_eq!(watchlists.len(), 0),
        Err(e) => panic!("Error getting watchlists: {}", e),
    }
    let watchlist = match create_watchlist(
        &alpaca,
        CreateWatchlistParams::builder()
            .name("test".to_string())
            .symbols(vec!["AAPL".to_string(), "GOOG".to_string()])
            .build(),
    )
    .await
    {
        Ok(watchlist) => {
            assert_eq!(watchlist.assets.len(), 2);
            assert_eq!(watchlist.name, "test");
            watchlist
        }
        Err(e) => panic!("Error creating watchlist with symbols: {}", e),
    };
    let watchlist = update_watchlist_by_id(
        &alpaca,
        watchlist.id,
        UpdateWatchlistParams::builder()
            .name("test2".to_string())
            .symbols(vec!["AAPL".to_string(), "GOOG".to_string()])
            .build(),
    )
    .await
    .unwrap();
    let watchlist = match get_watchlist_by_id(&alpaca, watchlist.id).await {
        Ok(watchlist) => watchlist,
        Err(e) => panic!("Error getting watchlist by id: {}", e),
    };
    assert_eq!(watchlist.assets.len(), 2);
    match delete_watchlist_by_name(&alpaca, "test2".to_string()).await {
        Ok(_) => (),
        Err(e) => panic!("Error deleting watchlist by name: {e}"),
    };
    match get_watchlists(&alpaca).await {
        Ok(watchlists) => assert_eq!(watchlists.len(), 0),
        Err(e) => panic!("Error getting watchlists: {}", e),
    }

    delete_all_watchlists(&alpaca).await.unwrap();
}

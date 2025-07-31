use crate::auth;
use crate::auth::TradingType;
use auth::Alpaca;
use reqwest::{Method, Response};
use serde::Serialize;

pub async fn create_request<T: Serialize>(
    alpaca: &Alpaca,
    method: Method,
    endpoint: &str,
    body: Option<T>,
) -> Result<Response, reqwest::Error> {
    let url = format!("{}{}", alpaca.get_trading_url(), endpoint);
    let client = alpaca.get_http_client();

    let mut request_builder = client
        .request(method, &url)
        .header("APCA-API-KEY-ID", alpaca.get_apca_api_key_id())
        .header("APCA-API-SECRET-KEY", alpaca.get_apca_api_secret());

    if let Some(json_body) = body {
        request_builder = request_builder.json(&json_body);
    }

    request_builder.send().await
}

#[tokio::test]
async fn test_auth_connection() {
    let alpaca = Alpaca::from_env(TradingType::Paper).expect("Failed to read env");
    match create_request::<()>(&alpaca, Method::GET, "/v2/account", None).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => assert_ne!(text, "{\"message\":\"forbidden.\"}\n"),
            Err(e) => {
                eprintln!("Failed to read response: {}", e);
                assert!(false);
            }
        },
        Err(e) => {
            eprintln!("Request failed: {}", e);
            assert!(false);
        }
    }
}

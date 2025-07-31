use crate::auth::{Alpaca, TradingType};
use crate::request::create_request;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CalendarParams {
    start: Option<String>,
    end: Option<String>,
    date_type: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Calendar {
    date: String,
    open: String,
    close: String,
    settlement_date: String,
}

pub async fn get_calendar(
    alpaca: &Alpaca,
    params: CalendarParams,
) -> Result<Vec<Calendar>, Box<dyn std::error::Error>> {
    let base_endpoint = "/v2/calendar";

    // Convert the params struct to a query string
    let query_string = serde_qs::to_string(&params)?;
    let endpoint_with_query = format!("{base_endpoint}?{query_string}");
    let response = create_request::<()>(alpaca, Method::GET, &*endpoint_with_query, None).await?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Getting calendar failed: {text}").into());
    }

    Ok(response.json().await?)
}

#[tokio::test]
async fn test_calendar() {
    let alpaca = Alpaca::from_env(TradingType::Paper).unwrap();
    let params = CalendarParams {
        ..Default::default()
    };
    match get_calendar(&alpaca, params).await {
        Ok(calendar) => {
            assert_eq!(calendar[0].close, "16:00");
            assert_eq!(calendar[0].date, "1970-01-02");
        }
        Err(e) => panic!("Error: {}", e),
    }
}

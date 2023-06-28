use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use anyhow::Result;
use super::ExchangeInfo;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TickerResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    price_change_percent: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    high_price: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    low_price: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    volume: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    last_price: f64,
}

#[derive(Deserialize)]
struct AvgResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    price: f64
}

pub async fn binance() -> Result<ExchangeInfo>{
    let avg_res: AvgResponse = reqwest::get("https://api3.binance.com/api/v3/avgPrice?symbol=APEUSDT").await?.json().await?;
    let ticker_res: TickerResponse = reqwest::get("https://api3.binance.com/api/v3/ticker/24hr?symbol=APEUSDT").await?.json().await?;

    Ok(ExchangeInfo {
        exchange: "Binance".to_owned(),
        url: "https://www.binance.com/es/trade/APE_USDT".to_owned(),
        highest_price: ticker_res.high_price,
        lowest_price: ticker_res.low_price,
        avg_price: avg_res.price,
        last_price: ticker_res.last_price,
        volume: ticker_res.volume,
        growth: ticker_res.price_change_percent
    })
}

//
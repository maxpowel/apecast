use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use anyhow::Result;
use super::ExchangeInfo;


#[derive(Deserialize)]
struct TickerResponse {
    result: TickerData
}

#[derive(Deserialize)]
struct TickerData {
    data: Vec<Ticker>
}

#[derive(Deserialize)]
struct Ticker {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    h: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    l: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    a: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    v: f64,
}

#[derive(Deserialize)]
struct CandlestickResponse {
    result: CandlestickData,
}

#[derive(Deserialize)]
struct CandlestickData {
    data: Vec<Candlestick>
}

#[derive(Deserialize)]
struct Candlestick {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    o: f64,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    c: f64
}

pub async fn crypto() -> Result<ExchangeInfo>{
    let ticker_res: TickerResponse = reqwest::get("https://api.crypto.com/v2/public/get-ticker?instrument_name=APE_USDT").await?.json().await?;
    let ticker = ticker_res.result.data.last().unwrap();
    let candlestick_res: CandlestickResponse = reqwest::get("https://api.crypto.com/v2/public/get-candlestick?instrument_name=APE_USDT&timeframe=1D").await?.json().await?;
    let candlestick = candlestick_res.result.data.last().unwrap();
    Ok(ExchangeInfo {
        exchange: "Crypto".to_owned(),
        url: "https://crypto.com/exchange/trade/APE_USDT".to_owned(),
        highest_price: ticker.h,
        lowest_price: ticker.l,
        avg_price: (candlestick.c + candlestick.o)/2.0,
        last_price: ticker.a,
        volume: ticker.v,
        growth: if candlestick.c > candlestick.o {
            (100.0 - candlestick.c / candlestick.o * 100.0)* -1.0
        } else {
            100.0 - candlestick.o / candlestick.c * 100.0
        }
    })
}

//
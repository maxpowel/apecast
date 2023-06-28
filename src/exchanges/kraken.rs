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
    #[serde(rename = "APEUSD")]
    apeusd: Ticker
}


#[derive(Deserialize)]
struct Ticker {
    h: Vec<String>,
    l: Vec<String>,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    o: f64,
    c: Vec<String>,
    v: Vec<String>,
}


pub async fn kraken() -> Result<ExchangeInfo>{
    let ticker_res: TickerResponse = reqwest::get("https://api.kraken.com/0/public/Ticker?pair=APEUSD").await?.json().await?;
    let ticker = ticker_res.result.apeusd;
    let last_price = ticker.c[0].parse().unwrap();
    let opening_price = ticker.o;
    Ok(ExchangeInfo {
        exchange: "Kraken".to_owned(),
        url: "https://www.kraken.com/prices/apecoin".to_owned(),
        highest_price: ticker.h[1].parse().unwrap(),
        lowest_price: ticker.l[1].parse().unwrap(),
        avg_price: (opening_price + last_price)/2.0,
        last_price,
        volume: ticker.v[1].parse().unwrap(),
        growth: if last_price > opening_price {
            (100.0 - last_price / opening_price * 100.0)* -1.0
        } else {
            100.0 - opening_price / last_price * 100.0
        }
    })
}

//
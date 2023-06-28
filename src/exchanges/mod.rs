
pub mod binance;
pub mod crypto;
pub mod kraken;

use anyhow::Result;
use log::error;

pub async fn exchange_info() -> Result<Vec<ExchangeInfo>> {
    let mut joins = vec![];
    joins.push(tokio::spawn(async {binance::binance().await}));
    joins.push(tokio::spawn(async {crypto::crypto().await}));
    joins.push(tokio::spawn(async {kraken::kraken().await}));

    let info: Vec<ExchangeInfo> = futures::future::join_all(joins).await.into_iter().filter_map(|res| match res {
        Ok(res) => {
            match res {
                Ok(info) => {
                    Some(info)
                },
                Err(err) => {
                    error!("Exchange info error: {:?}: ", err);
                    None
                }
            }
        },
        Err(err) => {
            error!("Join error: {:?}: ", err);
            None
        }
    }).collect();

    Ok(info)
}

#[derive(Debug)]
pub struct ExchangeInfo {
    pub exchange: String,
    pub url: String,
    pub avg_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub last_price: f64,
    pub volume: f64,
    pub growth: f64,
}

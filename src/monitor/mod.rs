mod beincrypto;
mod coindesk;
mod cointelegraph;
mod cryptodaily;
mod newsbtc;
mod utoday;
mod apecoin;

pub use apecoin::apecoin_monitor;
use std::pin::Pin;
use log::info;
use anyhow::Result;
use mongodb::Database;

use crate::{snapshot::get_proposals, storage::crawl};
//use crate::storage::get_database;
type Crawler = Pin<Box<dyn futures::Future<Output = Result<Option<String>>>>>;

pub async fn news(db: std::sync::Arc<Database>) -> Result<Vec<String>> {

    let crawlers: Vec<Crawler> = vec![
        Box::pin(beincrypto::latest(db.clone())),
        Box::pin(coindesk::latest(db.clone())),
        Box::pin(cointelegraph::latest(db.clone())),
        Box::pin(cryptodaily::latest(db.clone())),
        Box::pin(newsbtc::latest(db.clone())),
        Box::pin(utoday::latest(db.clone())),
    ];

    let new_news: Vec<String> = futures::future::join_all(crawlers).await
        .into_iter().filter_map(|f| if let Ok(f) = f {f} else {None}).collect();

    Ok(new_news)

}

pub async fn proposals(db: std::sync::Arc<Database>) -> Result<Vec<String>> {
    info!("Checking for new proposals");
    let mut props = vec![];
    for i in get_proposals().await? {
        let url = format!("https://snapshot.org/#/apecoin.eth/proposal/{}", i.id);
        if crawl(db.clone(), &url).await?{
            props.push(url);
        }
    }
    Ok(props)

}
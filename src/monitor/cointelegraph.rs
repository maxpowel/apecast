use anyhow::Result;
use mongodb::Database;
use scraper::{Html, Selector};
use log::info;
use crate::storage::crawl;


pub async fn latest(db: std::sync::Arc<Database>) -> Result<Option<String>> {
    info!("Running cointelegraph crawler");
    let response = reqwest::get("https://cointelegraph.com/tags/apecoin").await?.text().await?;
    let document = Html::parse_document(&response);

    let selector = Selector::parse("a.post-card-inline__title-link").unwrap();
    // Whe only need the most recent element
    if let Some(element) = document.select(&selector).next() {
        let url = element.value().attr("href");
        if let Some(url) = url {
            let url = format!("https://cointelegraph.com{}", url);
            if crawl(db.clone(), &url).await?{
                Ok(Some(url))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
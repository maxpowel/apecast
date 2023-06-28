use anyhow::Result;
use mongodb::Database;
use scraper::{Html, Selector};
use log::info;
use crate::storage::crawl;


pub async fn latest(db: std::sync::Arc<Database>) -> Result<Option<String>> {
    info!("Running utoday crawler");
    let response = reqwest::get("https://u.today/search/node?keys=apecoin").await?.text().await?;
    let document = Html::parse_document(&response);

    let selector = Selector::parse("a.category-item__title-link").unwrap();
    // Whe only need the most recent element
    if let Some(element) = document.select(&selector).next() {
        let url = element.value().attr("href");
        if let Some(url) = url {
            if crawl(db.clone(), url).await?{
                Ok(Some(url.to_owned()))
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
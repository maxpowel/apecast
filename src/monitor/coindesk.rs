use anyhow::Result;
use mongodb::Database;
use log::info;
use serde::Deserialize;
use crate::storage::crawl;

#[derive(Deserialize)]
struct Item {
    link: String
}

#[derive(Deserialize)]
struct ApiResponse {
    items: Vec<Item>
}


pub async fn latest(db: std::sync::Arc<Database>) -> Result<Option<String>> {
    info!("Running coindesk crawler");
    let response:ApiResponse = reqwest::get("https://api.queryly.com/json.aspx?queryly_key=d0ab87fd70264c0a&query=apecoin&batchsize=1").await?.json().await?;
    let url = format!("https://www.coindesk.com{}", response.items[0].link);
    if crawl(db.clone(), &url).await?{
        Ok(Some(url))
    } else {
        Ok(None)
    }

}
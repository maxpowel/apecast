use anyhow::Result;
use mongodb::Database;
use log::info;
use serde::Deserialize;
use crate::storage::crawl;

#[derive(Deserialize)]
struct Item {
    id: i32,
    name: String
}

#[derive(Deserialize)]
struct DataResponse {
    listings: Vec<Item>
}

#[derive(Deserialize)]
struct ApiResponse {
    data: DataResponse
}


pub async fn latest(db: std::sync::Arc<Database>) -> Result<Option<String>> {
    info!("Running thrivecoin crawler");
    let response:ApiResponse = reqwest::get("https://core.api.thrivecoin.com/v1/seasons/thankape-season-3/active_listings?page=1").await?.json().await?;
    let url = format!("thrivecoin-ape/{}", response.data.listings[0].id);
    if crawl(db.clone(), &url).await?{
        Ok(Some(format!("New thankape contribution available: {} https://thankape.com/seasons/thankape-season-3 ",response.data.listings[0].name)))
    } else {
        Ok(None)
    }

}
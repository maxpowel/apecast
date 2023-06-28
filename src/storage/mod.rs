mod model;
mod repository;

use mongodb::{options::ClientOptions, Client, Database};
use anyhow::Result;

pub use repository::{subscribe, subscribed, unsubscribe, subs, link, linked, crawl, message};

pub async fn get_database(mongo_uri: &str) -> Result<Database> {
    let client_options = ClientOptions::parse(mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    if let Some(db) =client.default_database() {
        Ok(db)
    } else {
        Err(anyhow::anyhow!("Cannot get a database, please ensure that you are providing your database name in your mongodb url"))
    }

}
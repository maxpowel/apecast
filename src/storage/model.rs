use serde::{Deserialize, Serialize};
//use chrono::{DateTime, Utc};
use teloxide::types::ChatId;

#[derive(Deserialize, Serialize, Debug)]
pub struct Proposal {
    pub id: String,
    pub title: String
}


#[derive(Deserialize, Serialize, Debug)]
pub struct UserSubscription {
    pub chat: ChatId,
    pub subs: Vec<String>,
    pub address: Option<String>,
    //#[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    //pub sub_time: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewsEntry {
    pub url: String
}
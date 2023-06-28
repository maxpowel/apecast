use anyhow::Result;
//use chrono::Utc;
use mongodb::{Database, bson::{to_bson, Document, to_document}, Cursor, options::UpdateOptions};
use teloxide::prelude::Message;
use teloxide::types::ChatId;
use std::sync::Arc;
use mongodb::bson::doc;
use super::model::{UserSubscription, NewsEntry};

/// Set the url as crawled and returns true if it was crawled for first time
pub async fn crawl(db: Arc<Database>, url: &str) -> Result<bool> {
    let col = db.collection::<NewsEntry>("news");
    let opts = UpdateOptions::builder().upsert(true).build();
    let res = col.update_one(
        doc!{"url": url}, 
        doc!{"$set": { "url": url } },
        opts).await?;
    
    Ok(res.upserted_id.is_some())
}

/// Insert a message for debugging tasks
pub async fn message(db: Arc<Database>, message: Message) -> Result<()> {
    let col = db.collection::<Document>("messages");

    col.insert_one(
        to_document(&message)?, 
        None).await?;
    
    Ok(())
}

/// Link an apecoin address
pub async fn link(db: Arc<Database> , chat_id: &ChatId, address: Option<&str>) -> Result<()> {
    let col = db.collection::<UserSubscription>("subs");
    if let Some(address) = address {
        col.update_one(
            doc!{"chat": to_bson(&chat_id).unwrap()}, 
            doc!{"$set": { "address": address } },
            None).await?;
    } else {
        col.update_one(
            doc!{"chat": to_bson(&chat_id).unwrap()}, 
            doc!{"$unset": { "address": true } },
            None).await?;
    }
    
    Ok(())
}

/// Get the linked apecoin address
pub async fn linked(db: Arc<Database> , chat_id: &ChatId) -> Result<Option<String>> {
    let col = db.collection::<UserSubscription>("subs");
    let res = col.find_one(doc!{
        "chat": to_bson(&chat_id).unwrap()
    }, None).await?;
    if let Some(res) = res {
        Ok(res.address)
    } else {
        Ok(None)
    }
    
}

/// Get subscribed events
pub async fn subscribed(db: Arc<Database>, chat_id: &ChatId) -> Result<Vec<String>> {
    let col = db.collection::<UserSubscription>("subs");
    let res = col.find_one(doc!{
        "chat": to_bson(&chat_id).unwrap()
    }, None).await?;

    let subs = if let Some(res) = res {
        res.subs
    } else {
        vec![]
    };

    Ok(subs)
}

/// Unsubscribe from an event
pub async fn unsubscribe(db: Arc<Database>, chat_id: &ChatId, event: &str) -> Result<()> {
    let col = db.collection::<UserSubscription>("subs");
    col.update_one(
        doc!{"chat": to_bson(&chat_id).unwrap()}, 
        doc!{"$pull": { "subs": event } },
        None).await?;

    Ok(())
}

/// Subscribe to an event
pub async fn subscribe(db: Arc<Database>, chat_id: ChatId, event: &str) -> Result<()> {
    let col = db.collection::<UserSubscription>("subs");
    let opts = UpdateOptions::builder().upsert(true).build();
    col.update_one(
        doc!{"chat": to_bson(&chat_id).unwrap()}, 
        doc!{"$addToSet": { "subs": event } },
        opts).await?;

    Ok(())

}

/// Get users subscribed to an event (used for broadcast events)
pub async fn subs(db: Arc<Database>, sub_type: &str) -> Result<Cursor<UserSubscription>> {
    let col: mongodb::Collection<UserSubscription> = db.collection::<UserSubscription>("subs");
    Ok(col.find(doc!{"subs": {"$in": [sub_type]}}, None).await?)
}
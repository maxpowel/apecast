//! This is a telegram bot for apes. An application not a library. For more information please go to https://github.com/maxpowel/apecast
mod telegram;
mod snapshot;
mod storage;
mod exchanges;
mod etherscan;
mod monitor;
mod search;

use meilisearch_sdk::Client;
use tokio::time;
use clap::Parser;
use anyhow::Result;
use wixet_bootstrap::init;
use log::{info, error};

use crate::storage::get_database;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mongodb connection uri
    #[arg(short, long, env, default_value="mongodb://localhost:27017/apecast")]
    mongodb: String,

    /// Telegram bot token
    #[arg(short, long, env)]
    telegram_token: String,

    /// etherscan bot token
    #[arg(short, long, env)]
    etherscan_token: Option<String>,

    /// Search uri
    #[arg(long, env, default_value="http://localhost:7700")]
    search_uri: String,

    /// Search token
    #[arg(long, env, default_value="masterKey")]
    search_token: String,
}


#[tokio::main]
async fn main() -> Result<()> {

    let (closer, exit) = init(Some("apecast.log")).await?;
    let args = Args::parse();

    match get_database(&args.mongodb).await {
        Ok(db) => {
            let db = std::sync::Arc::new(db);
            let search = std::sync::Arc::new(search::Search::new(&args.search_uri, &args.search_token));

            let etherscan = std::sync::Arc::new(etherscan::EtherscanClient::new(args.etherscan_token));
            let (bot, bot_exit, bot_join_handle) = telegram::new_bot(&args.telegram_token, etherscan, db.clone(), search).await;
            // Apecoin monitor
            let (broadcast_sender, broadcast_receiver) = flume::unbounded();
            let (monitor_join, monitor_exit) = monitor::apecoin_monitor(db.clone(), broadcast_sender)?;
            info!("Bot is up and running");
            // Check for news every hour
            let mut crawl_interval = time::interval(time::Duration::from_secs(3600));
            let mut run = true;
            while run {
                tokio::select!{
                    _ = exit.recv_async() => {
                        info!("Shutdown process started");
                        bot_exit.send_async(1).await?;
                        monitor_exit.send_async(1).await?;
                        run = false;
                    },
                    _ = crawl_interval.tick() => {
                        info!("Crawl time!");
                        match crate::monitor::news(db.clone()).await {
                            Ok(res) => {
                                    for news in res {
                                    if let Err(error) = crate::telegram::broadcast_message(bot.clone(), db.clone(), &news, "news").await {
                                        error!("Cannot broadcast news message: {:?}", error);
                                    }
                                }
                            }, 
                            Err(error) => {
                                error!("News error: {:?}", error);
                            }
                        }
                        info!("Looking for new proposals");
                        match crate::monitor::proposals(db.clone()).await {
                            Ok(props) => {
                                for prop in props {
                                    let msg = format!("New proposal! Check it out {}", prop);
                                    if let Err(error) = crate::telegram::broadcast_message(bot.clone(), db.clone(), &msg, "proposal").await {
                                        error!("Cannot broadcast proposals message: {:?}", error);
                                    }
                                }
                            }, 
                            Err(error) => {
                                error!("Proposals error: {:?}", error);
                            }
                        }
                        
                        info!("Checking thankape contributions");
                        match crate::monitor::contributions(db.clone()).await {
                            Ok(contribs) => {
                                if let Some(contribution) = contribs {
                                    if let Err(error) = crate::telegram::broadcast_message(bot.clone(), db.clone(), &contribution, "thankape").await {
                                        error!("Cannot broadcast thankape message: {:?}", error);
                                    }
                                }
                            }, 
                            Err(error) => {
                                error!("Contributions error: {:?}", error);
                            }
                        }
                        
                    },
                    msg = broadcast_receiver.recv_async() => {
                        match msg {
                            Ok(message) => {
                                crate::telegram::broadcast_message(bot.clone(), db.clone(), &message, "price").await?;
                            },
                            Err(error) => {
                                error!("Apecoin monitor error: {}", error);
                            }
                        }
                        
                    }
                };
            }
    
            bot_join_handle.await?;
            monitor_join.await?;
        }
        Err(err) => {
            error!("Database error: {}", err);
        }
    }

    closer.stop().await?;
    info!("Bye");
    Ok(())
}

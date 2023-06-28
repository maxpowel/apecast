use anyhow::Result;
use log::{info, error};
use crypto_com_exchange::{CryptoClient, TimeFrame, candlestick, SubscribeResult};
use std::sync::Arc;
use mongodb::Database;

pub struct Container {
    pub db: Arc<Database>,
    pub broadcast: flume::Sender<String>,
    pub unsubscriber: flume::Sender<String>
}

pub async fn event_handler(event: Result<SubscribeResult>, container: std::sync::Arc<Container>) {
    match event {
        Ok(message) => {
            match message {
                SubscribeResult::CandlestickResult(result) => {
                    let open = result.data[0].open;
                    let close = result.data[0].close;
                    let increment = if open < close {
                        // To the dip XD
                        (1.0f32 - close/open)*-100.0f32
                    } else {
                        // To the moon
                        (1.0f32 - open/close)*100.0f32
                    };

                    if increment.abs() >= 4.0f32 {
                        let message = format!("Apecoin price change: {}%, the current price is {}", increment, close);
                        container.broadcast.send_async(message).await.unwrap();
                        //Notified, ignore changes for a while
                        container.unsubscriber.send_async(candlestick(TimeFrame::FourHours, &result.instrument_name)).await.unwrap();
                    }
                },
                _ => {
                    info!("Unexpected message")
                }
                
            }
        },
        Err(error) => {
            error!("{:?}", error);
        }
    }
}



pub fn apecoin_monitor(db: Arc<Database>, broadcast: flume::Sender<String>) -> Result<(tokio::task::JoinHandle<()>, flume::Sender<i32>)>{

    let (exit_sender, exit_receiver) = flume::unbounded();

    let join_handler = tokio::spawn(async move {
        let instruments: Vec<String> = vec![candlestick(TimeFrame::FourHours, "APE_USDT")];
        let mut run = true;
        info!("Apecoin monitor ready");
        while run {
            // It is better to define this stuff inside the while to avoid closed channel errors
            let (unsubscriber_sender, unsubscriber_receiver) = flume::unbounded();
            let (subscriber_sender, subscriber_receiver) = flume::unbounded();
            let container = Arc::new(Container{
                db: db.clone(),
                broadcast: broadcast.clone(),
                unsubscriber: unsubscriber_sender
            });
            
            let mut c = CryptoClient::new(event_handler, container.clone());
            c.connect_market().await.unwrap();
            info!("Subscribing");
            c.subscribe(instruments.to_vec()).await.unwrap();
            info!("Subscribed");

            let urecv = unsubscriber_receiver.clone();
            let srecv = subscriber_receiver.clone();
            let mut listen = true;
            while listen {
                tokio::select!{
                    _ = exit_receiver.recv_async() => {
                        listen = false;
                        run = false;
                        c.disconnect().await.unwrap();
                    },
                    res = c.wait() => {
                        listen = false;
                        if let Err(error) = res {
                            // Some connection error happened o connection was closed. Reconnect
                            info!("Connection closed because of {:?}. Reconnecting", error);
                        } else {
                            // Peacefully exit
                            run = false;
                        }
                    },
                    unsubscribe = urecv.recv_async() => {
                        // Channel to unsubscribe
                        let channel = unsubscribe.unwrap();
                        info!("Channel {} will be silenced", channel);
                        c.unsubscribe(vec![channel.to_owned()]).await.unwrap();
                        // Subscribe in 4 hours
                        let spawn_sender = subscriber_sender.clone();
                        tokio::spawn(async move {
                            info!("Channel {} will avaiable again in 4 hours", channel);
                            tokio::time::sleep(tokio::time::Duration::from_secs(3600 * 4)).await;
                            info!("Channel {} will be resubscribed", channel);
                            spawn_sender.send_async(channel).await.unwrap();
                        });
                        
                    },
                    subscribe = srecv.recv_async() => {
                        // Channel to subscribe
                        let channel = subscribe.unwrap();
                        info!("Resubscribing channel {}", channel);
                        c.subscribe(vec![channel]).await.unwrap();
                    }
        
                };
            }
            
        }
    });

    

    Ok((join_handler, exit_sender))

}
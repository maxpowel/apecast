
use std::error::Error;
use std::sync::Arc;
use teloxide::{
    prelude::{Message, Dialogue, Update, Bot, Requester, Request, Dispatcher, dptree}, 
    types::{CallbackQuery, InlineQuery}, 
    utils::command::BotCommands,
    dispatching::{UpdateFilterExt, HandlerExt},
    payloads::SendMessageSetters
};
use mongodb::Database;
use chrono::{Utc, NaiveDateTime, DateTime};
use anyhow::Result;
use tokio_stream::StreamExt;
use crate::exchanges::exchange_info;

use teloxide::dispatching::dialogue::InMemStorage;

use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
    InputMessageContentText, ChatAction, InlineQueryResult, ParseMode
};

use crate::storage::subs;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    None,
    ReceiveApeAddress
}

type LinkDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;


#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Greeting command")]
    Start,

    #[command(description = "Link an ape address")]
    Link,

    #[command(description = "Balance of the linked ape address")]
    Balance,

    #[command(description = "List of active proposals")]
    Proposals,

    #[command(description = "Subscribe to events")]
    Subscribe,

    #[command(description = "Check active subscriptions")]
    Subscribed,

    #[command(description = "Unsubscribe to events")]
    Unsubscribe,

    #[command(description = "Bot status")]
    Status,

    #[command(description = "Price on different exchanges")]
    Price,

    #[command(description = "display this text.")]
    Help,
}

/// Initialize a bot instance
pub async fn new_bot(token: &str, etherscan: Arc<crate::etherscan::EtherscanClient>, db: Arc<Database>) -> (Arc<teloxide::Bot>, flume::Sender<i32>, tokio::task::JoinHandle<()>) {
    let bot = Arc::new(Bot::new(token));
    bot.set_my_commands(Command::bot_commands()).await.unwrap();
    let (telegram_sender, telegram_receiver) = flume::unbounded();
    (bot.clone(), telegram_sender, tokio::spawn(tele(bot.clone(), telegram_receiver, db, etherscan)))
}

/// Send an notification to all users subscribed to the subscription type
pub async fn broadcast_message(bot: Arc<Bot>, db: Arc<Database>, message: &str, sub_type: &str) -> Result<()> {
    let mut cursor = subs(db, sub_type).await?;
    while let Some(user) = cursor.try_next().await? {
        bot.send_message(user.chat, message).await?;
    }

    Ok(())
}

/// Command callback 
async fn callback_handler(bot: Arc<Bot>, q: CallbackQuery, db: Arc<Database>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(event) = q.data {
        if let Some((command, value)) = event.split_once(':') {
            let text = match command {
                "sub" => {
                    if value.eq("cancel") {
                        None
                    } else {
        
                        crate::storage::subscribe(db, q.message.as_ref().unwrap().chat.id, value).await?;
                        if value.eq("price") {
                            Some(format!("Subscribed to {}! You will receive a message when there are significant changes in the price of apecoin", value))
                        } else if value.eq("proposal") {
                            Some(format!("Subscribed to {}! You will receive a message when a new proposal is listed", value))
                        } else if value.eq("news") {
                            Some(format!("Subscribed to {}! You will receive a message when some relevant information about apecoin is published", value))
                        } else {
                            Some(format!("Subscribed to {}!", value))
                        }
                    }
                },
                "unsub" => {
                    if value.eq("cancel") {
                        None
                    } else {
        
                        crate::storage::unsubscribe(db, &q.message.as_ref().unwrap().chat.id, value).await?;
                        Some(format!("Unsubscribed from {}", value))
                    }
                },
                _ => {
                    Some("Sorry, could not understand you".to_owned())
                }
            };
            
            bot.answer_callback_query(q.id).await?;
            if let Some(text) = text {
                if let Some(Message { id, chat, .. }) = q.message {
                    bot.edit_message_text(chat.id, id, text).await?;
                } else if let Some(id) = q.inline_message_id {
                    bot.edit_message_text_inline(id, text).await?;
                }
            } else if let Some(Message { id, chat, .. }) = q.message {
                bot.delete_message(chat.id, id).await?;
            }
        }
    }

    Ok(())
}

// Process user query to show suggestions
async fn inline_query_handler(
    bot: Arc<Bot>,
    q: InlineQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    
    let props = crate::snapshot::get_proposals().await?;
    let results = props.into_iter().enumerate().filter_map(|(index, prop)| 
        if prop.title.contains(&q.query) {
        Some(InlineQueryResult::Article(InlineQueryResultArticle::new(
            index.to_string(),
            &prop.title,
            InputMessageContent::Text(InputMessageContentText::new(format!(
                "https://snapshot.org/#/apecoin.eth/proposal/{}",
                prop.id
            ))),
        )
        .description("Proposal")
        .thumb_url("https://cdn.stamp.fyi/space/apecoin.eth?s=160&cb=ec19915e02892e80".parse().unwrap())
        .url(format!("https://snapshot.org/#/apecoin.eth/proposal/{}", prop.id).parse().unwrap())))}
        else {None}
    );

    let response = bot.answer_inline_query(&q.id, results).send().await;
    if let Err(err) = response {
        log::error!("Error in handler: {:?}", err);
    }
    Ok(())
}

/// Task run by the telegram bot
async fn tele(bot: Arc<Bot>, exit: flume::Receiver<i32>, db: Arc<Database>, etherscan: Arc<crate::etherscan::EtherscanClient>) {

    let handler = dptree::entry()
    .branch(Update::filter_message()
    .enter_dialogue::<Message, InMemStorage<State>, State>().branch(dptree::case![State::ReceiveApeAddress].endpoint(receive_ape_address))
    .filter_command::<Command>().endpoint(answer))
    .branch(Update::filter_callback_query().endpoint(callback_handler))
    .branch(Update::filter_inline_query().endpoint(inline_query_handler))
    .branch(Update::filter_message().endpoint(any_message));

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
    .dependencies(dptree::deps![db.clone(), InMemStorage::<State>::new(), etherscan.clone()])
    .build();

    let token = dispatcher.shutdown_token();
    tokio::spawn(async move {
        exit.recv_async().await.unwrap();
        match token.shutdown() {
            Ok(f) => {
                log::info!("shutting down telegram");
                f.await;
                log::info!("telegram shutdown");
            }
            Err(_) => {
                log::info!("telegram was already shutdown");
            }
        };
    });

    dispatcher.dispatch().await

}

/// Inline keyboard for menu selection
fn make_keyboard(data_type: &str, options: Vec<&str>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for actions in options.chunks(3) {
        let row = actions
            .iter()
            .map(|action| InlineKeyboardButton::callback(action.to_owned(), format!("{}:{}", data_type, action)))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

// Dialogue callback
async fn receive_ape_address(bot: Arc<Bot>, dialogue: LinkDialogue, msg: Message, db: Arc<Database>) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            if text.starts_with("0x") {
                crate::storage::link(db, &msg.chat.id, text.split(' ').collect::<Vec<&str>>().first().copied()).await?;
                bot.send_message(msg.chat.id, format!("Linked address <b>{}</b>", text)).parse_mode(ParseMode::Html).await?;
                dialogue.exit().await?;
            } else if text.to_lowercase().starts_with("cancel") {
                dialogue.exit().await?;
            } else if text.to_lowercase().starts_with("none") {
                crate::storage::link(db, &msg.chat.id, None).await?;
                bot.send_message(msg.chat.id, "Address unlinked").await?;
                dialogue.exit().await?;
            } else {
                bot.send_message(msg.chat.id, "The address should starts with 0x. You can also write <b>cancel</b> to cancel or <b>none</b> to unlink").parse_mode(ParseMode::Html).await?;
            }
            
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}

async fn any_message(
    message: Message,
    db: Arc<Database>,

) -> Result<(), Box<dyn Error + Send + Sync>> {
    crate::storage::message(db, message).await?;
    Ok(())
}
async fn answer(
    bot: Arc<Bot>,
    message: Message,
    command: Command,
    db: Arc<Database>,
    etherscan: Arc<crate::etherscan::EtherscanClient>,
    dialogue: LinkDialogue,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Start => {
            bot.send_message(message.chat.id, "Hello! I can provide you some summarized information and links to interesting sutff, just take a look on this and I hope you find me useful!").await?;
            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Proposals => {
            bot.send_chat_action(message.chat.id, ChatAction::Typing).await?;
            let proposals: Vec<String> = crate::snapshot::get_proposals().await?.iter().map(|proposal| {
                let datetime = NaiveDateTime::from_timestamp_opt(proposal.end, 0).unwrap();
                let utc_datetime = DateTime::<Utc>::from_utc(datetime, Utc);
                let when = utc_datetime -Utc::now();
                let when = if when.num_days() > 0 {
                    format!("{} days", when.num_days())
                } else if when.num_hours() > 0 {
                    format!("{} hours", when.num_hours())
                } else {
                    format!("{} minutes", when.num_minutes())
                };
                
                format!("<a href=\"https://snapshot.org/#/apecoin.eth/proposal/{}\">{}</a> ends in <b>{}</b> ", proposal.id, proposal.title, when)
            }).collect();
            
            
            bot.send_message(message.chat.id, proposals.join("\n")).parse_mode(ParseMode::Html).await?;
        }
        Command::Subscribe => {
            let keyboard = make_keyboard("sub", vec!["price", "proposal", "news", "cancel"]);
            bot.send_message(message.chat.id, "What kind of event you want to subscribe?").reply_markup(keyboard).await?;
        }
        Command::Subscribed => {
            let subs = crate::storage::subscribed(db, &message.chat.id).await?;
            let text = if subs.is_empty() {
                "Not subscribed to anything...".to_owned()
            } else {
                format!("You are subscribed to: {}", subs.join(", "))
            };

            bot.send_message(message.chat.id, text).await?;
           
        }
        Command::Unsubscribe => {
            let mut subs = crate::storage::subscribed(db, &message.chat.id).await?;
            if subs.is_empty() {
                bot.send_message(message.chat.id, "You are not subscribed to any event").await?;    
            } else {
                subs.push("cancel".to_owned());
                let keyboard = make_keyboard("unsub", subs.iter().map(|s| s.as_str()).collect());
                bot.send_message(message.chat.id, "What kind of event you want to unsubscribe?:").reply_markup(keyboard).await?;
            }
            
        }
        Command::Status => {
            bot.send_message(message.chat.id, "Online").await?;
        }
        Command::Price => {
            bot.send_chat_action(message.chat.id, ChatAction::Typing).await?;
            let info: Vec<String> = exchange_info().await?.iter().map(|info| format!("<b>{}</b>: {} ({:.2}%)\n<i>High</> {} \n<i>Low</> {} \n<i>Last</> {} \n{} \n", info.exchange, info.avg_price, info.growth, info.highest_price, info.lowest_price, info.last_price, info.url)).collect();
            bot.send_message(message.chat.id, info.join("\n")).disable_web_page_preview(true).parse_mode(ParseMode::Html).await?;
            
        }
        Command::Link => {
            bot.send_message(message.chat.id, "Please tell me the ape address (ex: 0x4d224452801aced8b2f0aebe155379bb5d594381)").await?;
            dialogue.update(State::ReceiveApeAddress).await?;

        }
        Command::Balance => {
            bot.send_chat_action(message.chat.id, ChatAction::Typing).await?;
            let address = crate::storage::linked(db, &message.chat.id).await?;
            if let Some(address) = address {
                match etherscan.balance(&address).await {
                    Ok(balance) => {
                        bot.send_message(message.chat.id, format!("The balance of <b>{}</b> is <b>{}</b> apes", address, balance)).parse_mode(ParseMode::Html).await?;
                    },
                    Err(error) => {
                        bot.send_message(message.chat.id, format!("Cannot get balace because of <b>{:?}</b>", error)).parse_mode(ParseMode::Html).await?;
                    }
                }
            } else {
                bot.send_message(message.chat.id, "You dont have any linked address, please link one first with /link command").await?;
            }
            
        }
        Command::Help => {
            bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
        }
    };

    Ok(())
}
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId},
    utils::command::BotCommands,
};

use crate::handlers::subscription_update;

use super::State as GlobalState;
use super::{
    subscribe,
    utils::{set_local_commands, HanderError, HandlerResult},
};
use super::{Command as RootCommand, GlobalDialogue};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "add new subscription")]
    Subscribe,
    #[command(description = "cancel the operation")]
    Cancel,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    #[default]
    Idle,
    DisplayAllSubscriptions {
        message_data: (ChatId, MessageId),
    },
    Subscribe,
    UpdateSubscription {
        address: String,
    },
    UpdateAlertBeforeExpirationInMins {
        address: String,
    },
    UpdateMaxMessageFrequencyInBlocks {
        address: String,
    },
    Unsubscribe {
        address: String,
    },
}

pub async fn transition_to_display_all_subscriptions(
    chat_id: ChatId,
    bot: &Bot,
    message: Message,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    dialogue
        .update(GlobalState::ManageValidatorSubscriptions(
            State::DisplayAllSubscriptions {
                message_data: (message.chat.id, message.id),
            },
        ))
        .await?;
    set_local_commands(chat_id, bot, Command::bot_commands()).await
}

async fn make_subscriptions_markup(
    chat_id: i64,
    get_all_subscriptions: Arc<crate::BioauthSettings>,
) -> (InlineKeyboardMarkup, usize) {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let subscriptions = get_all_subscriptions.get_all_subscriptions(chat_id).await;
    let subscriptions_len = subscriptions.clone().len();
    for subscription in subscriptions {
        keyboard.push(vec![InlineKeyboardButton::callback(
            subscription.to_owned(),
            subscription.to_owned(),
        )]);
    }

    (InlineKeyboardMarkup::new(keyboard), subscriptions_len)
}

const START_MESSAGE: &str = {
    "
You don't have any subscriptions yet,

use /help command to display bot usage instructions.
"
};

async fn start(
    bot: Bot,
    msg: Message,
    dialogue: GlobalDialogue,
    get_all_subscriptions: Arc<crate::BioauthSettings>,
) -> HandlerResult {
    let (keyboard, len) = make_subscriptions_markup(msg.chat.id.0, get_all_subscriptions).await;
    let chat_id = msg.chat.id;

    let message = if len == 0 {
        bot.send_message(chat_id, START_MESSAGE)
            .reply_markup(keyboard)
            .await?
    } else {
        bot.send_message(chat_id, "Choose a subscription to manage it, or use /help command to display bot usage instructions.")
            .reply_markup(keyboard)
            .await?
    };

    transition_to_display_all_subscriptions(msg.chat.id, &bot, message, dialogue).await
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

const CANCEL_MESSAGE: &str = {
    "
Your validator subscriptions remain unchanged.

use /help command to display bot usage instructions.
"
};

async fn cancel(
    bot: Bot,
    msg: Message,
    message_data: (ChatId, MessageId),
    dialogue: GlobalDialogue,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, CANCEL_MESSAGE).await?;

    bot.edit_message_text(
        message_data.0,
        message_data.1,
        "You have canceled the action.",
    )
    .await?;

    super::transition_to_start(chat_id, &bot, dialogue).await
}

async fn callback_handler(
    bot: Bot,
    dialogue: GlobalDialogue,
    callback_query: CallbackQuery,
) -> HandlerResult {
    bot.answer_callback_query(callback_query.id).await?;

    if let Some(address) = callback_query.data {
        let text = format!("Subscription {address} selected");

        if let Some(Message { id, chat, .. }) = callback_query.message {
            bot.edit_message_text(chat.id, id, text).await?;

            subscription_update::transition_to_update_subscription(
                chat.id,
                &bot,
                address.clone(),
                dialogue,
            )
            .await?;

            bot.send_message(
                chat.id,
                "Use /help command to display bot usage instructions.",
            )
            .await?;
        }
    }

    Ok(())
}

pub fn schema() -> UpdateHandler<HanderError> {
    let root_command_handler = teloxide::filter_command::<RootCommand, _>()
        .branch(dptree::case![RootCommand::ManageValidatorSubscriptions].endpoint(start));

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Help].endpoint(help))
        .branch(dptree::case![Command::Subscribe].endpoint(subscribe::command))
        .branch(dptree::case![Command::Cancel].endpoint(cancel));

    dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
                .branch(dptree::case![GlobalState::Start].branch(root_command_handler))
                .branch(
                    dptree::case![GlobalState::ManageValidatorSubscriptions(x)].branch(
                        dptree::case![State::DisplayAllSubscriptions { message_data }]
                            .branch(command_handler),
                    ),
                ),
        )
        .branch(subscribe::schema())
        .branch(subscription_update::schema())
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<GlobalState>, GlobalState>()
                .branch(
                    dptree::filter(|state| {
                        matches!(state, GlobalState::ManageValidatorSubscriptions(_))
                    })
                    .endpoint(callback_handler),
                ),
        )
}

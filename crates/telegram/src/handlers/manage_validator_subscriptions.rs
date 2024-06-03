use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

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
    #[command(description = "Subscribe to public substrate address to watch validator status")]
    Subscribe,
    #[command(description = "cancel the operation")]
    Cancel,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    #[default]
    DisplayAllSubscriptions,
    Subscribe,
}

fn make_subscriptions_markup(subscriptions: Vec<String>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for chunk in subscriptions.chunks(2) {
        let row = chunk
            .iter()
            .map(|subscription| {
                InlineKeyboardButton::callback(subscription.to_owned(), subscription.to_owned())
            })
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

async fn start(
    bot: Bot,
    msg: Message,
    dialogue: GlobalDialogue,
    get_all_subscriptions: Arc<crate::GetAllSubscriptions>,
) -> HandlerResult {
    let subscriptions_vec = get_all_subscriptions
        .get_all_subscriptions(msg.chat.id.0)
        .await;
    tracing::info!(
        message = "manage_validator_subscriptions start subscriptions_vec",
        ?subscriptions_vec
    );
    let keyboard = make_subscriptions_markup(subscriptions_vec);

    bot.send_message(msg.chat.id, "Subscriptions:")
        .reply_markup(keyboard)
        .await?;

    dialogue
        .update(GlobalState::ManageValidatorSubscriptions(
            State::DisplayAllSubscriptions,
        ))
        .await?;

    set_local_commands(&msg, &bot, Command::bot_commands()).await?;

    Ok(())
}

async fn manage_validator_subscriptions_help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "TODO: ManageValidatorSubscriptions help")
        .await?;
    Ok(())
}

async fn manage_validator_subscriptions_cancel(
    bot: Bot,
    msg: Message,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "TODO: ManageValidatorSubscriptions cancel")
        .await?;

    dialogue.update(GlobalState::Start).await?;

    set_local_commands(&msg, &bot, super::Command::bot_commands()).await?;
    Ok(())
}

async fn callback_handler(bot: Bot, callback_query: CallbackQuery) -> HandlerResult {
    if let Some(version) = callback_query.data {
        let text = format!("You chose: {version}");

        bot.answer_callback_query(callback_query.id).await?;

        // Edit text of the message to which the buttons were attached
        if let Some(Message { id, chat, .. }) = callback_query.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = callback_query.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }
    }

    Ok(())
}

pub fn schema() -> UpdateHandler<HanderError> {
    let callback_query = Update::filter_callback_query().endpoint(callback_handler);
    let root_command_handler = teloxide::filter_command::<RootCommand, _>()
        .branch(dptree::case![RootCommand::ManageValidatorSubscriptions].endpoint(start))
        .branch(callback_query);

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Help].endpoint(manage_validator_subscriptions_help))
        .branch(
            dptree::case![Command::Subscribe]
                .endpoint(subscribe::manage_validator_subscriptions_subscribe),
        )
        .branch(dptree::case![Command::Cancel].endpoint(manage_validator_subscriptions_cancel));

    Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
        .branch(dptree::case![GlobalState::Start].branch(root_command_handler))
        .branch(
            dptree::case![GlobalState::ManageValidatorSubscriptions(x)]
                .branch(dptree::case![State::DisplayAllSubscriptions].branch(command_handler))
                .branch(subscribe::schema()),
        )
}

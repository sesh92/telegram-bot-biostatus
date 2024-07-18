use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

use super::utils::{set_local_commands, HandlerError, HandlerResult};
use super::State as GlobalState;
use super::{Command as RootCommand, GlobalDialogue};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "cancel the operation")]
    Cancel,
}

const ENABLE_AFFECTED_VALIDATOR: (&str, (&str, &str)) = (
    "enable_affected_validator",
    ("Enable affected validator notifications", ENABLED_MESSAGE),
);
const DISABLE_AFFECTED_VALIDATOR: (&str, (&str, &str)) = (
    "disable_affected_validator",
    ("Disable affected validator notifications", DISABLED_MESSAGE),
);

const BUTTONS: [(&str, (&str, &str)); 2] = [ENABLE_AFFECTED_VALIDATOR, DISABLE_AFFECTED_VALIDATOR];

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    #[default]
    ChooseNotifications,
    SubscribeTo {
        variant: String,
    },
}

pub async fn transition_to_choose_notifications(
    chat_id: ChatId,
    bot: &Bot,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    dialogue
        .update(GlobalState::ManageNotificationFromDeveloper(
            State::ChooseNotifications,
        ))
        .await?;
    set_local_commands(chat_id, bot, Command::bot_commands()).await
}

fn make_subscriptions_markup(
    subscriptions: &dev_subscriptions::DevSubscriptions,
) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    if subscriptions.affected_validator {
        keyboard.push(vec![InlineKeyboardButton::callback(
            DISABLE_AFFECTED_VALIDATOR.1 .0,
            DISABLE_AFFECTED_VALIDATOR.0,
        )]);
    } else {
        keyboard.push(vec![InlineKeyboardButton::callback(
            ENABLE_AFFECTED_VALIDATOR.1 .0,
            ENABLE_AFFECTED_VALIDATOR.0,
        )]);
    }

    InlineKeyboardMarkup::new(keyboard)
}

async fn start(
    bot: Bot,
    msg: Message,
    dialogue: GlobalDialogue,
    rw_dev_subscription_map: Arc<tokio::sync::RwLock<dev_subscriptions::DevSubscriptionMap>>,
) -> HandlerResult {
    let dev_subscription_map = rw_dev_subscription_map.read().await;
    let chat_id = msg.chat.id;
    let subscriptions = dev_subscription_map.get(&chat_id.0);
    let keyboard = make_subscriptions_markup(subscriptions);

    bot.send_message(
        chat_id,
        "use /help command to display bot usage instructions.",
    )
    .reply_markup(keyboard)
    .await?;

    transition_to_choose_notifications(msg.chat.id, &bot, dialogue).await
}

// async fn affected_validator(bot: Bot, msg: Message) -> HandlerResult {}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

const CANCEL_MESSAGE: &str = {
    "
You have canceled the action.

Your developer notification subscriptions remain unchanged.

use /help command to display bot usage instructions.
"
};

async fn cancel(bot: Bot, msg: Message, dialogue: GlobalDialogue) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, CANCEL_MESSAGE).await?;

    super::transition_to_start(chat_id, &bot, dialogue).await
}

const ENABLED_MESSAGE: &str = {
    "
Subscription successfully enabled

use /help command to display bot usage instructions.
"
};

const DISABLED_MESSAGE: &str = {
    "
Subscription successfully disabled

use /help command to display bot usage instructions.
"
};

async fn callback_handler(
    bot: Bot,
    dialogue: GlobalDialogue,
    callback_query: CallbackQuery,
    tx: tokio::sync::mpsc::Sender<crate::SubscriptionUpdate>,
) -> HandlerResult {
    if let Some(variant) = callback_query.data {
        bot.answer_callback_query(callback_query.id).await?;

        if let Some(Message { id, chat, .. }) = callback_query.message {
            let (_, (_, text)) = {
                let button = BUTTONS.iter().find(|(one, _)| one == &variant);

                match button {
                    Some(val) => val,
                    None => return Err(anyhow::format_err!("Unhandled command").into()),
                }
            };

            match variant.as_str() {
                "enable_affected_validator" => {
                    tx.send(crate::SubscriptionUpdate::AffectedValidatorEnable {
                        chat_id: chat.id.0,
                    })
                    .await?
                }
                "disable_affected_validator" => {
                    tx.send(crate::SubscriptionUpdate::AffectedValidatorDisable {
                        chat_id: chat.id.0,
                    })
                    .await?
                }
                _ => {}
            };

            bot.edit_message_text(chat.id, id, text.to_string()).await?;

            super::transition_to_start(chat.id, &bot, dialogue).await?;
        }
    }

    Ok(())
}

pub fn schema() -> UpdateHandler<HandlerError> {
    let root_command_handler = teloxide::filter_command::<RootCommand, _>()
        .branch(dptree::case![RootCommand::ManageDevSubscriptions].endpoint(start));

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Help].endpoint(help))
        .branch(dptree::case![Command::Cancel].endpoint(cancel));

    dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
                .branch(dptree::case![GlobalState::Start].branch(root_command_handler))
                .branch(
                    dptree::case![GlobalState::ManageNotificationFromDeveloper(x)]
                        .branch(dptree::case![State::ChooseNotifications].branch(command_handler)),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<GlobalState>, GlobalState>()
                .branch(
                    dptree::filter(|state| {
                        matches!(state, GlobalState::ManageNotificationFromDeveloper(_))
                    })
                    .endpoint(callback_handler),
                ),
        )
}

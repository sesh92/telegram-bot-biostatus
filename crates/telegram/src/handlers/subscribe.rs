use std::str::FromStr;

use subxt::utils::AccountId32;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::types::MessageId;
use teloxide::utils::command::BotCommands;

use super::subscription_update::transition_to_update_subscription;
use super::State as GlobalState;
use crate::SubscriptionUpdate;

use super::manage_validator_subscriptions;
use super::utils::{set_local_commands, HandlerError, HandlerResult};
use super::GlobalDialogue;

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

pub async fn transition_to_subscribe(
    chat_id: ChatId,
    bot: &Bot,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    dialogue
        .update(GlobalState::ManageValidatorSubscriptions(
            manage_validator_subscriptions::State::Subscribe,
        ))
        .await?;
    set_local_commands(chat_id, bot, Command::bot_commands()).await
}

const COMMAND_MESSAGE: &str = {
    "
Enter the validator address (must start with 'hm..'), or use /help command to display bot usage instructions.
"
};

pub async fn command(
    bot: Bot,
    msg: Message,
    message_data: (ChatId, MessageId),
    dialogue: GlobalDialogue,
) -> HandlerResult {
    bot.send_message(msg.chat.id, COMMAND_MESSAGE).await?;

    bot.edit_message_text(
        message_data.0,
        message_data.1,
        "Subscription action is activated",
    )
    .await?;

    transition_to_subscribe(msg.chat.id, &bot, dialogue).await
}

const SUBSCRIBED_MESSAGE: &str = {
    "
Validator address successfully added.
You will now receive notifications according to your settings.
You can manage the settings for this subscription.

Use /help command to display bot usage instructions
"
};

pub async fn receive_address(
    msg: Message,
    bot: Bot,
    dialogue: GlobalDialogue,
    tx: tokio::sync::mpsc::Sender<SubscriptionUpdate>,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let text = msg.text();
    let address = {
        let text = match text {
            Some(text) => text,
            None => {
                bot.send_message(msg.chat.id, "Enter address").await?;
                return Ok(());
            }
        };
        match AccountId32::from_str(text) {
            Ok(val) => val,
            Err(error) => {
                bot.send_message(msg.chat.id, format!("Invalid address {}", error))
                    .await?;
                return Ok(());
            }
        }
    };

    tx.send(SubscriptionUpdate::SubscribeToValidator {
        chat_id: chat_id.0,
        bioauth_public_key: address.0,
    })
    .await?;

    if let Some(address) = text {
        bot.send_message(msg.chat.id, SUBSCRIBED_MESSAGE).await?;
        transition_to_update_subscription(chat_id, &bot, address.to_owned(), dialogue).await?;
    }

    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

const CANCEL_MESSAGE: &str = {
    "
You have canceled the action.

use /help command to display bot usage instructions.
"
};

pub async fn cancel(bot: Bot, msg: Message, dialogue: GlobalDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, CANCEL_MESSAGE).await?;

    dialogue.exit().await?;

    set_local_commands(msg.chat.id, &bot, super::Command::bot_commands()).await
}

pub fn schema() -> UpdateHandler<HandlerError> {
    let commands = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Help].endpoint(help))
        .branch(dptree::case![Command::Cancel].endpoint(cancel));

    Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
        .branch(
            dptree::case![GlobalState::ManageValidatorSubscriptions(x)].branch(
                dptree::case![manage_validator_subscriptions::State::Subscribe]
                    .branch(commands)
                    .endpoint(receive_address),
            ),
        )
}

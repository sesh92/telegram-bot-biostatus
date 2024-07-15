use std::str::FromStr;
use std::sync::Arc;

use subxt::utils::AccountId32;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use super::{subscription_update, State as GlobalState};
use crate::SubscriptionUpdate;

use super::manage_validator_subscriptions;
use super::utils::{set_local_commands, HanderError, HandlerResult};
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

pub async fn transition_to_update_max_message_frequency_in_blocks(
    chat_id: ChatId,
    bot: &Bot,
    address: String,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    dialogue
        .update(GlobalState::ManageValidatorSubscriptions(
            manage_validator_subscriptions::State::UpdateMaxMessageFrequencyInBlocks { address },
        ))
        .await?;
    set_local_commands(chat_id, bot, Command::bot_commands()).await
}

const COMMAND_MESSAGE: &str = {
    "
Enter the new value of the maximum message frequency in blocks (where each block is roughly equivalent to 6 seconds).
"
};

pub async fn command(
    msg: Message,
    bot: Bot,
    address: String,
    dialogue: GlobalDialogue,
    bioauth_settings: Arc<crate::BioauthSettings>,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let bytes = AccountId32::from_str(&address.clone())?.0;
    let settings = bioauth_settings.get(&(chat_id.0, bytes)).await;
    bot.send_message(
        chat_id,
        format!(
            "Current maximum message frequency for {}: {} blocks",
            address, settings.max_message_frequency_in_blocks
        ),
    )
    .await?;
    bot.send_message(chat_id, COMMAND_MESSAGE).await?;

    transition_to_update_max_message_frequency_in_blocks(chat_id, &bot, address.clone(), dialogue)
        .await
}

pub async fn update_max_message_frequency_in_blocks(
    msg: Message,
    bot: Bot,
    address: String,
    dialogue: GlobalDialogue,
    tx: tokio::sync::mpsc::Sender<SubscriptionUpdate>,
) -> HandlerResult {
    let chat_id = msg.chat.id;

    match msg.text().map(|text| text.parse::<u32>()) {
        Some(Ok(in_blocks)) => {
            let bytes = AccountId32::from_str(&address.clone())?.0;

            tx.send(
                SubscriptionUpdate::UpdateSubscriptionMaxMessageFrequencyInBlocks {
                    chat_id: chat_id.0,
                    bioauth_public_key: bytes,
                    in_blocks,
                },
            )
            .await?;

            subscription_update::transition_to_update_subscription(
                chat_id,
                &bot,
                address.clone(),
                dialogue,
            )
            .await?;

            bot.send_message(chat_id, "Maximum message frequency updated.")
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Enter the new value.")
                .await?;
        }
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

Your validator subscriptions remain unchanged.

use /help command to display bot usage instructions.
"
};

pub async fn cancel(
    bot: Bot,
    msg: Message,
    address: String,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, CANCEL_MESSAGE).await?;

    subscription_update::transition_to_update_subscription(chat_id, &bot, address, dialogue).await
}

pub fn schema() -> UpdateHandler<HanderError> {
    let commands = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Help].endpoint(help))
        .branch(dptree::case![Command::Cancel].endpoint(cancel));

    Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
        .branch(
            dptree::case![GlobalState::ManageValidatorSubscriptions(x)].branch(
                dptree::case![
                    manage_validator_subscriptions::State::UpdateMaxMessageFrequencyInBlocks {
                        address
                    }
                ]
                .branch(commands)
                .endpoint(update_max_message_frequency_in_blocks),
            ),
        )
}

use std::str::FromStr;

use subxt::utils::AccountId32;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use super::State as GlobalState;
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

pub async fn manage_validator_subscriptions_subscribe(
    bot: Bot,
    msg: Message,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "TODO: ManageValidatorSubscriptions subscribe, type your address or use command",
    )
    .await?;

    dialogue
        .update(GlobalState::ManageValidatorSubscriptions(
            manage_validator_subscriptions::State::Subscribe,
        ))
        .await?;

    Ok(())
}

/// Handle the get tokens address reception.
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
                bot.send_message(msg.chat.id, "type address").await?;
                return Ok(());
            }
        };
        match AccountId32::from_str(text) {
            Ok(val) => val,
            Err(error) => {
                bot.send_message(msg.chat.id, format!("invalid address {}", error))
                    .await?;
                return Ok(());
            }
        }
    };

    bot.send_message(msg.chat.id, "subscribing to the address")
        .await?;

    tx.send(SubscriptionUpdate::SubscribeToValidator {
        chat_id: chat_id.0,
        bioauth_public_key: address.0,
    })
    .await?;

    dialogue.exit().await?;

    set_local_commands(&msg, &bot, super::Command::bot_commands()).await
}

async fn subscribe_help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn subscribe_cancel(bot: Bot, msg: Message, dialogue: GlobalDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, "subscribe cancel").await?;

    dialogue
        .update(GlobalState::ManageValidatorSubscriptions(
            manage_validator_subscriptions::State::DisplayAllSubscriptions,
        ))
        .await?;

    set_local_commands(
        &msg,
        &bot,
        manage_validator_subscriptions::Command::bot_commands(),
    )
    .await
}

pub fn schema() -> UpdateHandler<HanderError> {
    let commands = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Help].endpoint(subscribe_help))
        .branch(dptree::case![Command::Cancel].endpoint(subscribe_cancel));

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

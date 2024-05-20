//! Telegram handlers.
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::str::FromStr;

use subxt::utils::AccountId32;
use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::{BotCommand, BotCommandScope},
    utils::command::BotCommands,
};

use crate::SubscriptionUpdate;

use super::{
    messages, teloxide_ext::efficient_dialogue_enter, ActivateFeaturesCommand, Command,
    SettingsCommand, State, StateDialogue,
};

type HanderError = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), HanderError>;

/// THe handlers schema.
pub fn schema() -> UpdateHandler<HanderError> {
    use dptree::case;

    let root_commands = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Start].endpoint(start))
        .branch(case![Command::ActivateFeatures].endpoint(activate_features))
        .branch(case![Command::Settings].endpoint(settings));

    let activate_features_commands = teloxide::filter_command::<ActivateFeaturesCommand, _>()
        .branch(case![ActivateFeaturesCommand::Help].endpoint(activate_features_help))
        .branch(
            case![ActivateFeaturesCommand::SetValidatorAddress { address }]
                .endpoint(set_validator_address),
        )
        .branch(
            case![ActivateFeaturesCommand::ClearValidatorAddress].endpoint(clear_validator_address),
        )
        // .branch(
        //     case![ActivateFeaturesCommand::SetBiomapperAddress { address }]
        //         .endpoint(set_biomapper_address),
        // )
        .branch(case![ActivateFeaturesCommand::Cancel].endpoint(activate_features_cancel));

    let settings_commands = teloxide::filter_command::<SettingsCommand, _>()
        .branch(case![SettingsCommand::Help].endpoint(activate_features_help))
        .branch(case![SettingsCommand::ResetAllFeatures].endpoint(reset_all_features))
        .branch(case![SettingsCommand::Cancel].endpoint(activate_features_cancel));

    let message_handler = Update::filter_message()
        .branch(case![State::ActivateFeatures].branch(activate_features_commands))
        .branch(case![State::Settings].branch(settings_commands))
        .branch(case![State::Start].branch(root_commands))
        .branch(dptree::endpoint(unknown_interaction));

    efficient_dialogue_enter::<Update, ErasedStorage<State>, State, _>().branch(message_handler)
}

/// Set new commands for a given local context deduced from the message.
async fn set_local_commands(msg: &Message, bot: &Bot, commands: Vec<BotCommand>) -> HandlerResult {
    let chat_id = msg.chat.id.into();
    bot.set_my_commands(commands)
        .scope(BotCommandScope::Chat { chat_id })
        .send()
        .await?;
    Ok(())
}

/// Set commands for activate features mode.
async fn commands_set_activate_features(msg: &Message, bot: &Bot) -> HandlerResult {
    set_local_commands(msg, bot, ActivateFeaturesCommand::bot_commands()).await
}

/// Set commands for settings mode.
async fn commands_set_settings(msg: &Message, bot: &Bot) -> HandlerResult {
    set_local_commands(msg, bot, SettingsCommand::bot_commands()).await
}

/// Handle help command.
async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

/// Handle start command.
async fn start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, messages::MESSAGE_WELCOME)
        .await?;

    Ok(())
}

/// Handle the activate features.
async fn activate_features(msg: Message, bot: Bot, dialogue: StateDialogue) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        messages::MESSAGE_ACTIVATE_FEATURES_DIALOGUE_START,
    )
    .await?;
    dialogue.update(State::ActivateFeatures).await?;
    commands_set_activate_features(&msg, &bot).await?;
    Ok(())
}

async fn activate_features_help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        ActivateFeaturesCommand::descriptions().to_string(),
    )
    .await?;
    Ok(())
}

/// Reset commands to usual mode.
async fn commands_reset(msg: &Message, bot: &Bot) -> HandlerResult {
    set_local_commands(msg, bot, Command::bot_commands()).await
}

/// Handle the set validator address.
async fn set_validator_address(
    bot: Bot,
    msg: Message,
    dialogue: StateDialogue,
    tx: tokio::sync::mpsc::Sender<SubscriptionUpdate>,
    address: String,
) -> HandlerResult {
    match AccountId32::from_str(&address) {
        Err(error) => {
            tracing::error!(message = "account32 construct error", ?error);

            Ok(())
        }
        Ok(account32) => {
            tracing::info!(message = "account32 parsed", ?account32);
            if let Err(error) = tx
                .send(SubscriptionUpdate {
                    chat_id: msg.chat.id.0,
                    bioauth_public_key: account32.0,
                })
                .await
            {
                tracing::error!(message = "send error", %error);
            }

            bot.send_message(msg.chat.id, messages::MESSAGE_SET_VALIDATOR_ADDRESS)
                .await?;
            dialogue.exit().await?;
            commands_reset(&msg, &bot).await?;

            Ok(())
        }
    }
}

/// Handle the clear validator address.
async fn clear_validator_address(
    bot: Bot,
    msg: Message,
    dialogue: StateDialogue,
    _tx: tokio::sync::mpsc::Sender<SubscriptionUpdate>,
) -> HandlerResult {
    // let chat_id: i64 = msg.chat.id.to_string().parse()?;

    // TODO: unsubscribe
    // tx.send(SubscriptionUpdate {
    //     chat_id,
    //     bioauth_public_key: None,
    // })
    // .await?;

    bot.send_message(msg.chat.id, messages::MESSAGE_SET_VALIDATOR_ADDRESS)
        .await?;
    dialogue.exit().await?;
    commands_reset(&msg, &bot).await?;

    Ok(())
}

/// Handle the set validator address.
async fn _set_biomapper_address(bot: Bot, msg: Message, dialogue: StateDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, messages::MESSAGE_SET_BIOMAPPER_ADDRESS)
        .await?;
    dialogue.exit().await?;

    // #TODO implement handler validator address
    commands_reset(&msg, &bot).await?;
    Ok(())
}

/// Handle the activate features operation cancellation.
async fn activate_features_cancel(
    bot: Bot,
    msg: Message,
    dialogue: StateDialogue,
) -> HandlerResult {
    bot.send_message(msg.chat.id, messages::MESSAGE_DIALOGUE_CANCEL)
        .await?;
    dialogue.exit().await?;
    commands_reset(&msg, &bot).await?;
    Ok(())
}

/// Handle the settings.
async fn settings(msg: Message, bot: Bot, dialogue: StateDialogue) -> HandlerResult {
    bot.send_message(msg.chat.id, messages::MESSAGE_SETTINGS_DIALOGUE_START)
        .await?;
    dialogue.update(State::Settings).await?;
    commands_set_settings(&msg, &bot).await?;
    Ok(())
}

/// Handle the set validator address.
async fn reset_all_features(bot: Bot, msg: Message, dialogue: StateDialogue) -> HandlerResult {
    tracing::info!(message = "chat_id", chat_id = ?msg.chat.id);
    bot.send_message(msg.chat.id, messages::MESSAGE_RESET_ALL_FEATURES)
        .await?;

    // #TODO implement handler validator address
    dialogue.exit().await?;
    commands_reset(&msg, &bot).await?;
    Ok(())
}

/// Handle unknown commands and messages.
async fn unknown_interaction(msg: Message, bot: Bot) -> HandlerResult {
    bot.send_message(msg.chat.id, messages::MESSAGE_OTHER)
        .await?;
    Ok(())
}

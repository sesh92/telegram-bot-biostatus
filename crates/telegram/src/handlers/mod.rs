use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};
use utils::{set_local_commands, HandlerResult};

use self::utils::HanderError;

pub mod admin;
pub mod common;
pub mod manage_dev_subscriptions;
pub mod manage_validator_subscriptions;
pub mod subscribe;
pub mod subscription_update;
pub mod unsubscribe;
pub mod update_alert_before_expiration_in_mins;
pub mod update_max_message_frequency_in_blocks;
pub mod utils;

pub type GlobalDialogue = Dialogue<State, ErasedStorage<State>>;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Welcome message")]
    Start,
    #[command(
        description = "manage validator subscriptions, you can add new ones, configure existing ones individually, update notification frequency, set warning time before validator status loss, or unsubscribe from one"
    )]
    ManageValidatorSubscriptions,
    #[command(
        description = "manage notifications from the developer. Stay informed about network updates that may affect your validator status"
    )]
    ManageDevSubscriptions,
    #[command(description = "#debug_command restart state.")]
    ResetState,
}

pub async fn transition_to_start(
    chat_id: ChatId,
    bot: &Bot,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    dialogue.update(State::Start).await?;
    set_local_commands(chat_id, bot, Command::bot_commands()).await
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    ManageValidatorSubscriptions(manage_validator_subscriptions::State),
    ManageNotificationFromDeveloper(manage_dev_subscriptions::State),
}

pub fn schema() -> UpdateHandler<HanderError> {
    dptree::entry()
        .branch(manage_validator_subscriptions::schema())
        .branch(manage_dev_subscriptions::schema())
        .branch(common::schema())
        .branch(admin::schema())
}

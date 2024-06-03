use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

use self::utils::HanderError;

pub mod common;
pub mod manage_validator_subscriptions;
pub mod subscribe;
pub mod utils;

pub type GlobalDialogue = Dialogue<State, ErasedStorage<State>>;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "check if I'm alive.")]
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start managing validator subscriptions.")]
    ManageValidatorSubscriptions,
    #[command(description = "debug call restart state.")]
    ResetState,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    ManageValidatorSubscriptions(manage_validator_subscriptions::State),
}

pub fn schema() -> UpdateHandler<HanderError> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .branch(manage_validator_subscriptions::schema())
                .branch(subscribe::schema()),
        )
        .branch(common::schema())
}

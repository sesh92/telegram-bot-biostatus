//! Telegram implementation.
#![allow(missing_docs)]

mod handlers;
mod messages;
mod teloxide_ext;

use chain::AccountSettings;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::future::Future;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::utils::command::BotCommands;
use teloxide::{dispatching::ShutdownToken, prelude::*};

/// Redis storage.
type MyStorage = std::sync::Arc<ErasedStorage<State>>;

#[derive(Derivative)]
#[derivative(Debug)]
/// Telegram encapsulates the interface to the telegram.
pub struct Telegram {
    /// The underlying Telegram Bot client.
    pub bot: Bot,
    pub bot_notifier: Bot,
    pub account_settings_tx: tokio::sync::mpsc::Sender<AccountSettings>,
    pub notification_rx: tokio::sync::mpsc::Receiver<AccountSettings>,
    #[derivative(Debug = "ignore")]
    pub storage: MyStorage,
}

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(description = "see the welcome message")]
    Start,
    #[command(description = "see the features list")]
    ActivateFeatures,
    #[command(description = "see the notification settings")]
    Settings,
}

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum ActivateFeaturesCommand {
    #[command(description = "display this text")]
    Help,
    #[command(description = "set public substrate address for watch validator status (hm...)")]
    SetValidatorAddress { address: String },
    #[command(description = "set public evm address for watch biomapper status (0x...)")]
    SetBiomapperAddress { address: String },
    #[command(description = "cancel the operation")]
    Cancel,
}

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum SettingsCommand {
    #[command(description = "display this text")]
    Help,
    #[command(description = "Reset all features")]
    ResetAllFeatures,
    #[command(description = "cancel the operation")]
    Cancel,
}

/// The state of set address.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    /// Dialogue start.
    #[default]
    Start,
    /// Actions to activate bot features.
    ActivateFeatures,
    /// Notification settings.
    Settings,
}

///  dialogue.
pub type StateDialogue = Dialogue<State, ErasedStorage<State>>;

impl Telegram {
    /// Set bot commands.
    pub async fn set_commands(&self) -> Result<(), anyhow::Error> {
        self.bot.set_my_commands(Command::bot_commands()).await?;
        Ok(())
    }

    /// Prepare the control future and a shutdown token.
    pub fn setup(self) -> (impl Future, ShutdownToken) {
        let Telegram {
            bot,
            bot_notifier,
            account_settings_tx,
            mut notification_rx,
            storage,
        } = self;

        tokio::spawn(async move {
            loop {
                let account_settings = notification_rx.recv().await;
                match account_settings {
                    Some(account_settings) => {
                        let res = bot_notifier
                            .send_message(
                                ChatId(account_settings.t_user.chat_id),
                                "you are not in a active validators list",
                            )
                            .await;

                        if let Err(error) = res {
                            tracing::error!(message = "notifier error", ?error);
                        }
                    }
                    None => continue,
                }
            }
        });

        let mut dispatcher = Dispatcher::builder(bot, handlers::schema())
            .dependencies(dptree::deps![account_settings_tx, storage])
            .build();

        let shutdown_token = dispatcher.shutdown_token();

        let fut = async move {
            dispatcher.dispatch().await;
        };

        (fut, shutdown_token)
    }
}

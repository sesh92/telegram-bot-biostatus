//! Telegram implementation.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

mod handlers;
mod messages;
mod teloxide_ext;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::future::Future;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::utils::command::BotCommands;
use teloxide::{dispatching::ShutdownToken, prelude::*};

/// Redis storage.
type MyStorage = std::sync::Arc<ErasedStorage<State>>;

/// Telegram encapsulates the interface to the telegram.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Telegram {
    /// The underlying Telegram Bot client.
    pub bot: Bot,
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
    #[command(description = "clear public substrate address to unwatch validator status")]
    ClearValidatorAddress,
    // #[command(description = "set public evm address for watch biomapper status (0x...)")]
    // SetBiomapperAddress { address: String },
    // #[command(description = "clear public evm address to unwatch biomapper status")]
    // ClearBiomapperAddress,
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

#[derive(Debug)]
pub struct SubscriptionUpdate {
    pub chat_id: i64,
    pub bioauth_public_key: [u8; 32],
}

#[derive(Debug)]
pub struct SubscriptionUpdateHandle {
    rx: tokio::sync::mpsc::Receiver<SubscriptionUpdate>,
}

impl SubscriptionUpdateHandle {
    pub async fn next(&mut self) -> Option<SubscriptionUpdate> {
        self.rx.recv().await
    }
}

#[derive(Debug)]
pub struct NotificationHandle {
    tx: tokio::sync::mpsc::Sender<channel_messages::Notification>,
}

impl NotificationHandle {
    pub async fn send_notification(
        &self,
        notification: channel_messages::Notification,
    ) -> Result<(), anyhow::Error> {
        self.tx
            .send(notification)
            .await
            .map_err(|_| anyhow::format_err!("NotificationHandle error"))
    }
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
    pub fn setup(
        self,
    ) -> (
        impl Future,
        ShutdownToken,
        SubscriptionUpdateHandle,
        NotificationHandle,
    ) {
        let Telegram { bot, storage } = self;

        let (subscription_update_tx, subscription_update_rx) =
            tokio::sync::mpsc::channel::<SubscriptionUpdate>(1000);
        let (notification_handle_tx, mut notification_handle_rx) =
            tokio::sync::mpsc::channel::<channel_messages::Notification>(1000);
        let subscription_update_handle = SubscriptionUpdateHandle {
            rx: subscription_update_rx,
        };
        let notification_handle = NotificationHandle {
            tx: notification_handle_tx,
        };
        {
            let bot = bot.clone();

            tokio::spawn(async move {
                loop {
                    let notification = notification_handle_rx.recv().await;
                    if let Some(notification) = notification {
                        let res = match notification {
                            channel_messages::Notification::BioauthLostNotification { chat_id } => {
                                bot.send_message(
                                    ChatId(chat_id),
                                    "You have lost bio-authentication to be an active validator.",
                                )
                                .await
                            }
                            channel_messages::Notification::BioauthSoonExpiredAlert { chat_id } => {
                                bot.send_message(
                                    ChatId(chat_id),
                                    "You will lost bio-authentication soon",
                                )
                                .await
                            }
                        };

                        if let Err(error) = res {
                            tracing::error!(message = "notifier error", ?error);
                        };
                    };
                }
            });
        }

        let mut dispatcher = Dispatcher::builder(bot, handlers::schema())
            .dependencies(dptree::deps![subscription_update_tx, storage])
            .build();

        let shutdown_token = dispatcher.shutdown_token();

        let fut = async move {
            dispatcher.dispatch().await;
        };

        (
            fut,
            shutdown_token,
            subscription_update_handle,
            notification_handle,
        )
    }
}

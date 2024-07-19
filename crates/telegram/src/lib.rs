//! Telegram implementation.
#![allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::multiple_crate_versions
)]

pub mod bioauth_handlers;
mod handlers;

use bioauth_handlers::SendNotificationError;
use derivative::Derivative;
use handlers::State as GlobalState;
use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use std::future::Future;
use std::sync::Arc;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::utils::command::BotCommands;
use teloxide::{dispatching::ShutdownToken, prelude::*};

/// Redis storage.
type MyStorage = Arc<ErasedStorage<GlobalState>>;

/// Telegram encapsulates the interface to the telegram.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Telegram {
    /// The underlying Telegram Bot client.
    pub bot: Bot,
    #[derivative(Debug = "ignore")]
    pub storage: MyStorage,
    pub rw_bioauth_settings_map:
        Arc<tokio::sync::RwLock<bioauth_settings::BioauthSettingsMap<[u8; 32]>>>,
    pub rw_dev_subscriptions_map: Arc<tokio::sync::RwLock<dev_subscriptions::DevSubscriptionMap>>,
    pub admin_chat_ids: Vec<i64>,
}

#[derive(Debug)]
pub enum SubscriptionUpdate {
    SubscribeToValidator {
        chat_id: i64,
        bioauth_public_key: [u8; 32],
    },
    UnsubscribeToValidator {
        chat_id: i64,
        bioauth_public_key: [u8; 32],
    },
    UpdateSubscriptionAlertBeforeExpirationInMins {
        chat_id: i64,
        bioauth_public_key: [u8; 32],
        in_mins: u64,
    },
    UpdateSubscriptionMaxMessageFrequencyInBlocks {
        chat_id: i64,
        bioauth_public_key: [u8; 32],
        in_blocks: u32,
    },
    RemoveAllValidatorSubscriptions {
        chat_id: i64,
    },
    AffectedValidatorEnable {
        chat_id: i64,
    },
    AffectedValidatorDisable {
        chat_id: i64,
    },
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
pub enum Notification {
    BioauthLostNotification {
        chat_id: i64,
        bioauth_public_key: [u8; 32],
    },
    BioauthSoonExpiredAlert {
        chat_id: i64,
        bioauth_public_key: [u8; 32],
    },
}

#[derive(Debug, Clone)]
pub struct NotificationHandle {
    tx: tokio::sync::mpsc::Sender<Notification>,
}

#[derive(Debug)]
pub struct BioauthSettings {
    pub rw_bioauth_settings_map:
        Arc<tokio::sync::RwLock<bioauth_settings::BioauthSettingsMap<[u8; 32]>>>,
}

impl BioauthSettings {
    async fn get(&self, key: &(i64, [u8; 32])) -> bioauth_settings::BioauthSettings {
        let bioauth_settings_map = self.rw_bioauth_settings_map.read().await;
        bioauth_settings_map.get(key).to_owned()
    }

    async fn get_all_subscriptions(&self, chat_id: i64) -> Vec<String> {
        let subscriptions = {
            let bioauth_settings_map = self.rw_bioauth_settings_map.read().await;
            bioauth_settings_map.get_all_subscriptions_by_id(chat_id)
        };

        tracing::info!(message = "get_all_subscriptions", ?subscriptions);

        subscriptions
            .iter()
            .map(|bytes| {
                sp_core::crypto::AccountId32::new(*bytes)
                    .to_ss58check_with_version(Ss58AddressFormatRegistry::HumanodeAccount.into())
            })
            .collect()
    }
}

impl NotificationHandle {
    #[cfg(any(test, feature = "test-utils"))]
    pub fn mock(tx: tokio::sync::mpsc::Sender<Notification>) -> Self {
        Self { tx }
    }

    pub async fn send_notification(
        &self,
        notification: Notification,
    ) -> Result<(), SendNotificationError> {
        self.tx.send(notification).await.unwrap();

        Ok(())
    }
}

impl Telegram {
    /// Set bot commands.
    pub async fn set_commands(&self) -> Result<(), anyhow::Error> {
        self.bot
            .set_my_commands(handlers::Command::bot_commands())
            .await?;

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
        let Telegram {
            bot,
            storage,
            rw_bioauth_settings_map,
            rw_dev_subscriptions_map,
            admin_chat_ids,
        } = self;

        let get_all_subscriptions = BioauthSettings {
            rw_bioauth_settings_map,
        };
        let get_all_subscriptions = Arc::new(get_all_subscriptions);

        let (subscription_update_tx, subscription_update_rx) =
            tokio::sync::mpsc::channel::<SubscriptionUpdate>(1000);
        let (notification_handle_tx, notification_handle_rx) =
            tokio::sync::mpsc::channel::<Notification>(1000);
        let subscription_update_handle = SubscriptionUpdateHandle {
            rx: subscription_update_rx,
        };
        let notification_handle = NotificationHandle {
            tx: notification_handle_tx,
        };
        {
            let bot = bot.clone();

            tokio::spawn(async move {
                if let Err(error) = bioauth_handlers::run_loop(bioauth_handlers::RunLoopParams {
                    bot,
                    notification_handle_rx,
                })
                .await
                {
                    tracing::error!(message = "bioauth_handlers::run_loop", ?error);
                }
            });
        }

        let mut dispatcher = Dispatcher::builder(bot, handlers::schema())
            .dependencies(dptree::deps![
                get_all_subscriptions,
                subscription_update_tx,
                rw_dev_subscriptions_map,
                admin_chat_ids,
                storage
            ])
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

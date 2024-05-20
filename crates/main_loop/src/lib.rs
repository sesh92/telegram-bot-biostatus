//! Main loop.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::sync::Arc;

use bioauth_logic::BioauthLogic;
use bioauth_notification_manager::BioauthNotificationManager;
use bioauth_settings::{BioauthSettings, BioauthSettingsMap};
use block_subscription::BlockSubscription;
use database::db::Db;
use tokio::{sync::Mutex, task::JoinSet};

#[derive(Debug)]
pub struct Params {
    pub db: Db,
    pub block_subscription: BlockSubscription,
    pub telegram_notification_handler: telegram::NotificationHandle,
    pub subscription_update_handle: telegram::SubscriptionUpdateHandle,
}

pub async fn run(params: Params) -> Result<JoinSet<()>, anyhow::Error> {
    let Params {
        db,
        mut block_subscription,
        telegram_notification_handler,
        mut subscription_update_handle,
    } = params;

    let all_loaded_data = db.load_for_initialization().await?;

    tracing::info!(message = "Got all load", ?all_loaded_data);
    let mut bioauths = vec![];
    let mut bioauth_settings = BioauthSettingsMap::new();

    for data in all_loaded_data {
        bioauths.push(bioauth_logic::InitParamBioauth {
            t_chat_id: data.t_chat_id,
            bioauth_public_key: data.validator_public_key,
        });

        bioauth_settings.update(
            data.t_chat_id,
            BioauthSettings {
                alert_before_expiration_in_mins: data.alert_before_expiration_in_mins,
                max_message_frequency_in_blocks: data.max_message_frequency_in_blocks,
            },
        );
    }

    let bioauth_logic = BioauthLogic::init(bioauth_logic::InitParams { bioauths });

    let mut bioauth_notification_manager =
        BioauthNotificationManager::new(telegram_notification_handler);

    let bioauth_logic = Arc::new(Mutex::new(bioauth_logic));
    let bioauth_settings_map = BioauthSettingsMap::new();

    let mut tasks = tokio::task::JoinSet::new();
    {
        let bioauth_logic = Arc::clone(&bioauth_logic);
        tasks.spawn(async move {
            loop {
                let block_subscription::BlockInfo {
                    block_number,
                    active_authentications_map,
                } = block_subscription.next_block().await.unwrap();

                let (chats_map, block_number) = {
                    let mut logic = bioauth_logic.lock().await;
                    logic.new_block(bioauth_logic::NewBlockParams {
                        active_authentications_map,
                        block_number,
                    })
                };

                bioauth_notification_manager.notify(block_number).await;
                bioauth_notification_manager.alert().await;

                chats_map.iter().for_each(|(chat_id, subscriptions)| {
                    let settings = bioauth_settings_map.get(chat_id);
                    tracing::info!(
                        message = "Propcess collected subscriptions from new block",
                        chat_id = ?chat_id,
                        subscription_len = subscriptions.keys().len()
                    );
                    for expired_at in subscriptions.values() {
                        bioauth_notification_manager.register(
                            chat_id,
                            settings.alert_before_expiration_in_mins,
                            settings.max_message_frequency_in_blocks,
                            block_number,
                            expired_at.as_ref(),
                        );
                    }
                });
            }
        });
    }

    {
        let bioauth_logic = Arc::clone(&bioauth_logic);
        tasks.spawn(async move {
            loop {
                let telegram::SubscriptionUpdate {
                    chat_id: t_chat_id,
                    bioauth_public_key,
                } = subscription_update_handle.next().await.unwrap();

                {
                    let mut bioauth_logic = bioauth_logic.lock().await;

                    bioauth_logic.update_subscription(bioauth_logic::UpdateSubscriptionParams {
                        t_chat_id,
                        bioauth_public_key,
                    });
                }
                tracing::info!(message = "New subscriptions", ?bioauth_public_key);

                db.bioauth_subscribe(t_chat_id, &bioauth_public_key)
                    .await
                    .unwrap();
            }
        });
    }

    Ok(tasks)
}

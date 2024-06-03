//! Main loop.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::sync::Arc;

use bioauth_logic::{BioauthLogic, FailedNotification};
use bioauth_settings::BioauthSettings;
use block_subscription::BlockSubscription;
use database::db::Db;
use tokio::{sync::Mutex, task::JoinSet};

#[derive(Debug)]
pub struct Params {
    pub db: Db,
    pub block_subscription: BlockSubscription,
    pub telegram_notification_handle: telegram::NotificationHandle,
    pub subscription_update_handle: telegram::SubscriptionUpdateHandle,
    pub rw_bioauth_settings_map:
        Arc<tokio::sync::RwLock<bioauth_settings::BioauthSettingsMap<[u8; 32]>>>,
}

pub async fn run(params: Params) -> Result<JoinSet<()>, anyhow::Error> {
    let Params {
        db,
        mut block_subscription,
        telegram_notification_handle,
        mut subscription_update_handle,
        rw_bioauth_settings_map,
    } = params;

    let all_loaded_data = db.load_for_initialization().await?;

    tracing::info!(message = "Got all load", ?all_loaded_data);
    let mut bioauths = vec![];

    {
        let mut bioauth_settings = rw_bioauth_settings_map.write().await;

        for data in all_loaded_data {
            tracing::info!(message = "main loop initializing data", ?data);

            bioauths.push(bioauth_logic::InitParamBioauth {
                t_chat_id: data.t_chat_id,
                bioauth_public_key: data.validator_public_key,
            });

            bioauth_settings.update(
                (data.t_chat_id, data.validator_public_key),
                BioauthSettings {
                    alert_before_expiration_in_mins: data.alert_before_expiration_in_mins,
                    max_message_frequency_in_blocks: data.max_message_frequency_in_blocks,
                },
            );
        }
    }

    let bioauth_logic = BioauthLogic::init(bioauth_logic::InitParams { bioauths });

    let (notification_failures_tx, mut notification_failures_rx) =
        tokio::sync::mpsc::channel(10_000);

    let bioauth_logic = Arc::new(Mutex::new(bioauth_logic));

    let mut tasks = tokio::task::JoinSet::new();
    {
        let bioauth_logic = Arc::clone(&bioauth_logic);
        let telegram_notification_handle = telegram_notification_handle.clone();
        let rw_bioauth_settings_map = Arc::clone(&rw_bioauth_settings_map);

        tasks.spawn(async move {
            let limit = 10_000;
            let mut notification_failures_buffer: Vec<FailedNotification> =
                Vec::with_capacity(limit);
            loop {
                let block_subscription::BlockInfo {
                    block_number,
                    active_authentications_map,
                } = block_subscription.next_block().await.unwrap();

                let notifications = {
                    let mut logic = bioauth_logic.lock().await;
                    let bioauth_settings_map = rw_bioauth_settings_map.read().await;
                    loop {
                        if notification_failures_rx.is_empty() {
                            break;
                        }
                        let size = notification_failures_rx
                            .recv_many(&mut notification_failures_buffer, limit)
                            .await;

                        if size == 0 {
                            break;
                        }

                        logic.communicate_notification_failures(&notification_failures_buffer);
                        notification_failures_buffer.clear();
                    }

                    logic.new_block(bioauth_logic::NewBlockParams {
                        active_authentications_map: &active_authentications_map,
                        block_number,
                        bioauth_settings_map: &bioauth_settings_map,
                    })
                };

                let telegram_notifications: Vec<telegram::Notification> = notifications
                    .iter()
                    .map(|notification| match notification {
                        bioauth_logic::Notification::BioauthLostNotification { chat_id } => {
                            telegram::Notification::BioauthLostNotification { chat_id: *chat_id }
                        }
                        bioauth_logic::Notification::BioauthSoonExpiredAlert { chat_id } => {
                            telegram::Notification::BioauthSoonExpiredAlert { chat_id: *chat_id }
                        }
                    })
                    .collect();

                for notification in telegram_notifications {
                    let notification_failures_tx = notification_failures_tx.clone();
                    let telegram_notification_handle = telegram_notification_handle.clone();

                    tokio::spawn(async move {
                        if let Err(telegram::bioauth_handlers::SendNotificationError {
                            notification,
                        }) = telegram_notification_handle
                            .send_notification(notification)
                            .await
                        {
                            let _ = notification_failures_tx.send(notification).await;
                        }
                    });
                }
            }
        });
    }

    {
        let bioauth_logic = Arc::clone(&bioauth_logic);
        let rw_bioauth_settings_map = Arc::clone(&rw_bioauth_settings_map);
        tasks.spawn(async move {
            loop {
                let subscription_update = subscription_update_handle.next().await.unwrap();
                tracing::info!(
                    message = "Got new subscription_update",
                    ?subscription_update
                );
                match subscription_update {
                    telegram::SubscriptionUpdate::SubscribeToValidator {
                        chat_id: t_chat_id,
                        bioauth_public_key,
                    } => {
                        {
                            let mut bioauth_logic = bioauth_logic.lock().await;

                            bioauth_logic.update_subscription(
                                bioauth_logic::UpdateSubscriptionParams {
                                    t_chat_id,
                                    bioauth_public_key,
                                },
                            );
                        };

                        tracing::info!(
                            message = "SubscribeToValidator",
                            ?t_chat_id,
                            ?bioauth_public_key
                        );

                        db.bioauth_subscribe(t_chat_id, &bioauth_public_key)
                            .await
                            .unwrap();
                    }
                    telegram::SubscriptionUpdate::UnsubscribeToValidator {
                        chat_id: t_chat_id,
                        bioauth_public_key,
                    } => {
                        {
                            let mut bioauth_logic = bioauth_logic.lock().await;

                            bioauth_logic.remove_subscription(
                                bioauth_logic::UpdateSubscriptionParams {
                                    t_chat_id,
                                    bioauth_public_key,
                                },
                            );
                        }
                        tracing::info!(
                            message = "UnsubscribeToValidator",
                            ?t_chat_id,
                            ?bioauth_public_key
                        );

                        db.bioauth_unsubscribe(t_chat_id, &bioauth_public_key)
                            .await
                            .unwrap();
                    }
                    telegram::SubscriptionUpdate::RemoveAllValidatorSubscriptions {
                        chat_id: t_chat_id,
                    } => {
                        {
                            let mut bioauth_logic = bioauth_logic.lock().await;

                            bioauth_logic.remove_all_subscription(t_chat_id);
                        }
                        tracing::info!(message = "RemoveAllValidatorSubscriptions", ?t_chat_id);

                        db.bioauth_unsubscribe_all(t_chat_id).await.unwrap();
                    }
                    telegram::SubscriptionUpdate::UpdateAlertBeforeExpirationInMins {
                        chat_id,
                        bioauth_public_key,
                        alert_before_expiration_in_mins,
                    } => {
                        let mut bioauth_settings_map = rw_bioauth_settings_map.write().await;
                        bioauth_settings_map.update_alert_before_expiration_in_mins(
                            (chat_id, bioauth_public_key),
                            alert_before_expiration_in_mins,
                        );
                    }
                    telegram::SubscriptionUpdate::UpdateMessageFrequencyInBlocks {
                        chat_id,
                        bioauth_public_key,
                        max_message_frequency_in_blocks,
                    } => {
                        let mut bioauth_settings_map = rw_bioauth_settings_map.write().await;

                        bioauth_settings_map.update_max_message_frequency_in_blocks(
                            (chat_id, bioauth_public_key),
                            max_message_frequency_in_blocks,
                        );
                    }
                }
            }
        });
    }

    Ok(tasks)
}

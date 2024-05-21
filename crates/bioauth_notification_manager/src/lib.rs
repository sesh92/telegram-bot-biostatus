#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Default)]
pub struct BioauthNotificationState {
    pub last_block_number_notified: u32,
    pub next_block_number_to_notify: u32,
    pub alert_at: Option<u64>,
}

#[derive(Debug)]
pub struct BioauthNotificationManager {
    map: HashMap<i64, BioauthNotificationState>,
    telegram_notification_handler: telegram::NotificationHandle,
}

impl BioauthNotificationManager {
    pub fn new(telegram_notification_handler: telegram::NotificationHandle) -> Self {
        BioauthNotificationManager {
            map: HashMap::new(),
            telegram_notification_handler,
        }
    }

    pub fn register(
        &mut self,
        chat_id: &i64,
        alert_before_expiration_in_mins: u64,
        max_message_frequency_in_blocks: u32,
        block_number: u32,
        expires_at: Option<&u64>,
    ) {
        let state = self.map.get_mut(chat_id);

        match (state, expires_at) {
            (None, None) => {
                self.map.insert(
                    *chat_id,
                    BioauthNotificationState {
                        ..Default::default()
                    },
                );
                tracing::info!(message = "Created default state for", ?chat_id);
            }
            (None, Some(expires_at)) => {
                let alert_at = Some(expires_at - alert_before_expiration_in_mins * 60000);
                self.map.insert(
                    *chat_id,
                    BioauthNotificationState {
                        alert_at,
                        ..Default::default()
                    },
                );
                tracing::info!(
                    message = "Created state with computed alert_at",
                    ?chat_id,
                    ?alert_at
                );
            }
            (Some(state), None) => {
                let last_block = {
                    match state.last_block_number_notified == 0 {
                        true => block_number,
                        false => state.last_block_number_notified,
                    }
                };
                tracing::info!(
                    message = "qwe",
                    ?last_block,
                    ?max_message_frequency_in_blocks,
                    ?block_number
                );

                if last_block + max_message_frequency_in_blocks < block_number {
                    return;
                }

                state.next_block_number_to_notify = last_block + max_message_frequency_in_blocks;
                tracing::info!(message = "Updated next_block_number_to_notify", ?chat_id, next_block_number_to_notify = ?state.next_block_number_to_notify);
            }
            (Some(state), Some(expires_at)) => {
                if state.alert_at.is_none() {
                    let alert_at = Some(expires_at - alert_before_expiration_in_mins * 60000);
                    self.map.insert(
                        *chat_id,
                        BioauthNotificationState {
                            alert_at,
                            ..Default::default()
                        },
                    );
                    tracing::info!(message = "Alerted, alert_at removed", ?chat_id);
                }
            }
        }
    }

    pub async fn notify(&mut self, block_number: u32) {
        for (chat_id, state) in self.map.iter_mut() {
            tracing::info!(message = "Notify process",
                ?chat_id,
                next_block_number_to_notify = ?state.next_block_number_to_notify,
                ?block_number
            );

            if state.next_block_number_to_notify > block_number {
                continue;
            }

            tracing::info!(message = "Notifing", ?chat_id);
            let res = self
                .telegram_notification_handler
                .send_notification(channel_messages::Notification::BioauthLostNotification {
                    chat_id: *chat_id,
                })
                .await;

            if let Err(error) = res {
                tracing::error!(message = "Send notification error", ?error);
            }

            state.last_block_number_notified = block_number;
            tracing::info!(message = "Updated last_block_number_notified", ?chat_id, last_block_number_notified = ?state.last_block_number_notified);
        }
    }

    pub async fn alert(&mut self) {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_the_epoch.as_secs();

        for (chat_id, state) in self.map.iter_mut() {
            match state.alert_at {
                None => continue,
                Some(alert_at) => {
                    if alert_at > timestamp {
                        continue;
                    }
                }
            };

            let res = self
                .telegram_notification_handler
                .send_notification(channel_messages::Notification::BioauthSoonExpiredAlert {
                    chat_id: *chat_id,
                })
                .await;

            if let Err(error) = res {
                tracing::error!(message = "Send notification error", ?error);
            }

            state.alert_at = None;
            tracing::info!(message = "Updated alert_at", ?chat_id, alert_at = ?state.alert_at);
        }
    }
}

#[cfg(test)]
mod tests;

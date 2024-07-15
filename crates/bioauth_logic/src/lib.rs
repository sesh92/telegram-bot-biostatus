#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    time::{SystemTime, UNIX_EPOCH},
};

use bioauth_settings::BioauthSettingsMap;
use bioauth_subscription_map::BioauthSubscriptionMap;

pub type ChatId = i64;

mod bioauth_subscription_map;

#[derive(Debug)]
pub enum FailedNotification {
    BioauthLostNotificationFailed { chat_id: i64 },
    BioauthSoonExpiredAlertFailed { chat_id: i64 },
}

#[derive(Debug)]
pub enum Notification<BioauthPublicKey> {
    BioauthLostNotification {
        chat_id: i64,
        bioauth_public_key: BioauthPublicKey,
    },
    BioauthSoonExpiredAlert {
        chat_id: i64,
        bioauth_public_key: BioauthPublicKey,
    },
}

#[derive(Debug)]
pub struct BioauthLogic<BioauthPublicKey> {
    pub bioauth_subscription_map: BioauthSubscriptionMap<BioauthPublicKey>,
}

#[derive(Debug)]
pub struct InitParamBioauth<BioauthPublicKey> {
    pub bioauth_public_key: BioauthPublicKey,
    pub t_chat_id: ChatId,
}

#[derive(Debug)]
pub struct InitParams<BioauthPublicKey> {
    pub bioauths: Vec<InitParamBioauth<BioauthPublicKey>>,
}

#[derive(Debug)]
pub struct NewBlockParams<'a, BioauthPublicKey> {
    pub block_number: u32,
    pub active_authentications_map: &'a HashMap<BioauthPublicKey, u64>,
    pub bioauth_settings_map: &'a BioauthSettingsMap<BioauthPublicKey>,
}

#[derive(Debug)]
pub struct UpdateSubscriptionParams<BioauthPublicKey> {
    pub t_chat_id: ChatId,
    pub bioauth_public_key: BioauthPublicKey,
}

#[derive(Debug, Clone)]
pub struct Subscrition {
    pub block_number: u32,
    pub expired_at: Option<u64>,
}

impl<BioauthPublicKey> BioauthLogic<BioauthPublicKey>
where
    BioauthPublicKey: Eq + Hash + Copy,
{
    pub fn init(params: InitParams<BioauthPublicKey>) -> Self {
        tracing::info!("BioauthLogic init");
        let mut bioauth_subscription_map = BioauthSubscriptionMap::new();

        for bioauth in params.bioauths {
            bioauth_subscription_map.subscribe(
                bioauth.bioauth_public_key,
                bioauth.t_chat_id,
                bioauth_subscription_map::BioauthNotificationState::default(),
            );
        }

        BioauthLogic {
            bioauth_subscription_map,
        }
    }

    pub fn communicate_notification_failures(&mut self, failures: &[FailedNotification]) {
        let mut failed_notifications_chat_ids = HashSet::new();
        let mut failed_alerts_chat_ids = HashSet::new();

        for failure in failures {
            match failure {
                FailedNotification::BioauthLostNotificationFailed { chat_id } => {
                    failed_notifications_chat_ids.insert(*chat_id);
                }
                FailedNotification::BioauthSoonExpiredAlertFailed { chat_id } => {
                    failed_alerts_chat_ids.insert(*chat_id);
                }
            }
        }

        for (_, chats) in self.bioauth_subscription_map.iter_mut() {
            for (chat_id, state) in chats {
                if failed_notifications_chat_ids.contains(chat_id) {
                    state.last_block_number_notified = 0;
                    state.next_block_number_to_notify = 0;
                }

                if failed_alerts_chat_ids.contains(chat_id) {
                    state.alerted_at = None;
                }
            }
        }
    }

    pub fn new_block(
        &mut self,
        params: NewBlockParams<BioauthPublicKey>,
    ) -> Vec<Notification<BioauthPublicKey>> {
        let NewBlockParams {
            block_number,
            active_authentications_map,
            bioauth_settings_map,
        } = params;

        let mut notifications = vec![];

        for (bioauth_public_key, chats) in self.bioauth_subscription_map.iter_mut() {
            let expires_at_opt = active_authentications_map.get(bioauth_public_key).copied();

            for (chat_id, state) in chats.iter_mut() {
                let settings = bioauth_settings_map.get(&(*chat_id, *bioauth_public_key));
                match expires_at_opt {
                    None => {
                        if block_number < state.next_block_number_to_notify
                            && state.next_block_number_to_notify != 0
                        {
                            continue;
                        }

                        notifications.push(Notification::BioauthLostNotification {
                            chat_id: *chat_id,
                            bioauth_public_key: *bioauth_public_key,
                        });

                        state.last_block_number_notified = block_number;
                        state.next_block_number_to_notify =
                            block_number + settings.max_message_frequency_in_blocks;
                    }
                    Some(expires_at) => {
                        let alert_at =
                            expires_at - settings.alert_before_expiration_in_mins * 60000;

                        let start = SystemTime::now();
                        let since_the_epoch = start
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards");
                        let timestamp = since_the_epoch.as_secs();

                        let alert_at = match state.alerted_at {
                            None => alert_at,
                            Some(alerted_at) => {
                                let mut res = alerted_at;
                                if alerted_at < alert_at {
                                    if alert_at <= timestamp {
                                        state.alerted_at = None;
                                    }
                                    res = alert_at
                                }
                                res
                            }
                        };

                        if alert_at <= timestamp {
                            notifications.push(Notification::BioauthSoonExpiredAlert {
                                chat_id: *chat_id,
                                bioauth_public_key: *bioauth_public_key,
                            });

                            state.alerted_at = Some(timestamp);
                        }
                    }
                }
            }
        }

        notifications
    }

    pub fn update_subscription(&mut self, params: UpdateSubscriptionParams<BioauthPublicKey>) {
        let UpdateSubscriptionParams {
            t_chat_id,
            bioauth_public_key,
        } = params;

        self.bioauth_subscription_map.subscribe(
            bioauth_public_key,
            t_chat_id,
            bioauth_subscription_map::BioauthNotificationState::default(),
        );
    }

    pub fn remove_subscription(&mut self, params: UpdateSubscriptionParams<BioauthPublicKey>) {
        let UpdateSubscriptionParams {
            t_chat_id,
            bioauth_public_key,
        } = params;

        self.bioauth_subscription_map
            .unsubscribe(bioauth_public_key, t_chat_id);
    }

    pub fn remove_all_subscription(&mut self, t_chat_id: ChatId) {
        self.bioauth_subscription_map.unsubscribe_all(t_chat_id);
    }
}

#[cfg(test)]
mod tests;

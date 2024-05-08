//! The chain interaction primitives and settings.
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

/// The generated runtime data.
mod gen {
    #![allow(missing_docs, clippy::too_many_arguments, clippy::enum_variant_names)]
    #[subxt::subxt(runtime_metadata_path = "../../generated/humanode_metadata.scale")]
    pub mod humanode {}
}
pub use gen::humanode;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum ActiveFeature {
    ActiveValidator,
    Biomapper,
}

#[derive(Debug, Clone)]
pub struct TUser {
    pub chat_id: i64,
}

#[derive(Debug)]
pub struct AccountSettings {
    pub address: AccountId32,
    pub t_user: TUser,
    pub active_features: HashSet<ActiveFeature>,
}

struct Sender {
    features: HashSet<ActiveFeature>,
    t_user: TUser,
}

#[derive(Debug)]
pub struct SubscribeActiveAuthenticationsParams {
    pub api: OnlineClient<PolkadotConfig>,
    pub account_settings_rx: tokio::sync::mpsc::Receiver<AccountSettings>,
    pub account_notification_tx: tokio::sync::mpsc::Sender<AccountSettings>,
}

/// Create a new API conneted to a given `url`.
pub async fn construct_api(url: String) -> Result<OnlineClient<PolkadotConfig>, subxt::Error> {
    let api = OnlineClient::<PolkadotConfig>::from_insecure_url(url).await?;
    Ok(api)
}

pub async fn subscribe_active_authentications(
    params: SubscribeActiveAuthenticationsParams,
) -> Result<(), anyhow::Error> {
    let mut validator_subscribers = HashMap::new();
    let mut biomapper_subscribers = HashMap::new();

    let limit = 10;
    let mut buffer: Vec<AccountSettings> = Vec::with_capacity(limit);

    let SubscribeActiveAuthenticationsParams {
        api,
        mut account_settings_rx,
        account_notification_tx,
    } = params;

    // Subscribe to all finalized blocks:
    let mut blocks_sub = api.blocks().subscribe_finalized().await?;

    while let Some(block) = blocks_sub.next().await {
        let block = block?;

        let events = match block.events().await {
            Err(error) => {
                tracing::error!(message = "block.events() error", ?error);
                continue;
            }
            Ok(val) => val,
        };

        for event in events.iter() {
            let event = event?;

            if let Ok(ev) = event.as_root_event::<humanode::bioauth::Event>() {
                match ev {
                    humanode::bioauth::Event::AuthenticationsExpired { expired } => {
                        // TODO: Handle expired authentications.
                        continue;
                    }
                    humanode::bioauth::Event::AuthenticationsRemoved { removed, reason } => {
                        // TODO: handler removed authentications
                        continue;
                    }
                    humanode::bioauth::Event::NewAuthentication {
                        validator_public_key,
                    } => {
                        // TODO: Handle new authentication
                        continue;
                    }
                }
            } else {
                continue;
            }
        }

        let account_settings_len = account_settings_rx.len();

        if account_settings_len > 0 {
            account_settings_rx.recv_many(&mut buffer, limit).await;

            for account_setting in buffer {
                tracing::info!(message = "got new account settings", ?account_setting);

                if account_setting
                    .active_features
                    .contains(&ActiveFeature::ActiveValidator)
                {
                    let address = account_setting.address.clone().to_string();
                    // if !validator_subscribers.contains_key(&address) {
                    //     validator_subscribers.insert(address, account_setting.t_user.clone());
                    // }
                    if let std::collections::hash_map::Entry::Vacant(map) =
                        validator_subscribers.entry(address)
                    {
                        map.insert(account_setting.t_user.clone());
                    }
                }
                if account_setting
                    .active_features
                    .contains(&ActiveFeature::Biomapper)
                {
                    let address = account_setting.address.clone().to_string();
                    // if !biomapper_subscribers.contains_key(&address) {
                    //     biomapper_subscribers.insert(address, account_setting.t_user.clone());
                    // }
                    if let std::collections::hash_map::Entry::Vacant(map) =
                        biomapper_subscribers.entry(address)
                    {
                        map.insert(account_setting.t_user.clone());
                    }
                }
            }
            buffer = Vec::with_capacity(limit);
        }

        let block_number = block.header().number;
        let block_hash = block.hash();

        tracing::debug!(?block_number, ?block_hash);

        let active_authentications = block
            .storage()
            .fetch(&gen::humanode::storage().bioauth().active_authentications())
            .await?;

        match active_authentications {
            Some(value) => {
                let active_authentications = value.0;
                let mut senders_map = HashMap::<String, Sender>::new();

                validator_subscribers.iter().for_each(|(address, t_user)| {
                    let found = active_authentications
                        .iter()
                        .find(|active_authentications| {
                            active_authentications.public_key.to_string() == address.clone()
                        });
                    if found.is_none() {
                        let key = address.to_string();
                        match senders_map.get(&key) {
                            Some(value) => {
                                let mut new_value = value.features.clone();
                                new_value.insert(ActiveFeature::ActiveValidator);
                                senders_map.insert(
                                    key,
                                    Sender {
                                        t_user: value.t_user.clone(),
                                        features: new_value,
                                    },
                                );
                            }
                            None => {
                                senders_map.insert(
                                    key,
                                    Sender {
                                        t_user: t_user.clone(),
                                        features: HashSet::from([ActiveFeature::ActiveValidator]),
                                    },
                                );
                            }
                        }
                    }
                });
                biomapper_subscribers.iter().for_each(|_address| {
                    // TODO: request to biomapper to get info about verification the address.
                });

                if senders_map.keys().len() > 0 {
                    for key in senders_map.keys() {
                        let sender = senders_map.get(key);
                        match sender {
                            Some(sender) => {
                                account_notification_tx
                                    .send(AccountSettings {
                                        address: AccountId32::from_str(key)?,
                                        t_user: sender.t_user.clone(),
                                        active_features: sender.features.clone(),
                                    })
                                    .await?;
                            }
                            None => {
                                tracing::error!(message = "senders_map.get(key) missing", key = ?key);
                            }
                        }
                    }
                }
            }
            None => continue,
        }
    }

    Ok(())
}

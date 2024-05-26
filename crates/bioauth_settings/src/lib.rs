#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone)]
pub struct BioauthSettings {
    pub max_message_frequency_in_blocks: u32,
    pub alert_before_expiration_in_mins: u64,
}

impl Default for BioauthSettings {
    fn default() -> Self {
        BioauthSettings {
            alert_before_expiration_in_mins: 60,
            max_message_frequency_in_blocks: 10,
        }
    }
}

const DEFAULT_SETTINGS: BioauthSettings = BioauthSettings {
    alert_before_expiration_in_mins: 60,
    max_message_frequency_in_blocks: 10,
};

#[derive(Debug, Clone)]
pub struct BioauthSettingsMap<Key>(HashMap<(i64, Key), BioauthSettings>);

impl<Key> Default for BioauthSettingsMap<Key> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<Key> BioauthSettingsMap<Key> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Key> BioauthSettingsMap<Key>
where
    Key: Clone + Hash + Eq,
{
    pub fn get(&self, key: &(i64, Key)) -> &BioauthSettings {
        let opt_value = self.0.get(key);
        match opt_value {
            None => &DEFAULT_SETTINGS,
            Some(val) => val,
        }
    }

    pub fn update(&mut self, key: (i64, Key), settings: BioauthSettings) {
        self.0.insert(key, settings);
    }

    pub fn update_alert_before_expiration_in_mins(
        &mut self,
        key: (i64, Key),
        alert_before_expiration_in_mins: u64,
    ) {
        let value = self.0.get_mut(&key);

        match value {
            None => {
                self.0.insert(
                    key.clone(),
                    BioauthSettings {
                        alert_before_expiration_in_mins,
                        ..BioauthSettings::default()
                    },
                );
            }
            Some(val) => {
                val.alert_before_expiration_in_mins = alert_before_expiration_in_mins;
            }
        }
    }

    pub fn update_max_message_frequency_in_blocks(
        &mut self,
        key: (i64, Key),
        max_message_frequency_in_blocks: u32,
    ) {
        let value = self.0.get_mut(&key);

        match value {
            None => {
                self.0.insert(
                    key,
                    BioauthSettings {
                        max_message_frequency_in_blocks,
                        ..BioauthSettings::default()
                    },
                );
            }
            Some(val) => {
                val.max_message_frequency_in_blocks = max_message_frequency_in_blocks;
            }
        }
    }
}

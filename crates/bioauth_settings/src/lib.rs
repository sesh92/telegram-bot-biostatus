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

#[derive(Debug)]
pub struct BioauthSettingsMap<Key>(HashMap<Key, BioauthSettings>);

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
    pub fn get(&self, key: &Key) -> &BioauthSettings {
        let opt_value = self.0.get(key);
        match opt_value {
            None => &DEFAULT_SETTINGS,
            Some(val) => val,
        }
    }

    pub fn update(&mut self, key: Key, settings: BioauthSettings) {
        self.0.insert(key, settings);
    }
}

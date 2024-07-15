#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct DevSubscriptions {
    pub affected_validator: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DevSubscriptionMap(HashMap<i64, DevSubscriptions>);

impl DevSubscriptionMap {
    pub fn new() -> Self {
        Self::default()
    }
}

const DEFAULT_DEV_SUBSCRIPTION: DevSubscriptions = DevSubscriptions {
    affected_validator: false,
};

impl DevSubscriptionMap {
    pub fn get(&self, key: &i64) -> &DevSubscriptions {
        let opt_value = self.0.get(key);
        match opt_value {
            None => &DEFAULT_DEV_SUBSCRIPTION,
            Some(val) => val,
        }
    }

    pub fn get_all_enabled_team_notification_subscribers(&self) -> HashSet<i64> {
        let mut subscribers = HashSet::new();

        for (id, data) in self.0.iter() {
            if !data.affected_validator {
                continue;
            }
            subscribers.insert(*id);
        }
        subscribers
    }

    pub fn update(&mut self, key: i64, settings: DevSubscriptions) {
        self.0.insert(key, settings);
    }

    pub fn remove(&mut self, key: &i64) {
        self.0.remove(key);
    }
}

use std::{collections::HashMap, hash::Hash};

use crate::ChatId;

#[derive(Debug, Clone, Default)]
pub struct BioauthNotificationState {
    pub last_block_number_notified: u32,
    pub next_block_number_to_notify: u32,
    pub alerted_at: Option<u64>,
}

#[derive(Debug)]
pub struct BioauthSubscriptionMap<Key>(HashMap<Key, HashMap<ChatId, BioauthNotificationState>>);

impl<Key> Default for BioauthSubscriptionMap<Key> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<Key> BioauthSubscriptionMap<Key> {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct BioauthSubscriptionMapIter<'a, Key> {
    inner: std::collections::hash_map::IterMut<'a, Key, HashMap<ChatId, BioauthNotificationState>>,
}

impl<'a, Key> Iterator for BioauthSubscriptionMapIter<'a, Key> {
    type Item = (&'a Key, &'a mut HashMap<ChatId, BioauthNotificationState>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<Key> BioauthSubscriptionMap<Key>
where
    Key: Hash + Eq,
{
    pub fn get(&self, key: &Key) -> Option<&HashMap<ChatId, BioauthNotificationState>> {
        self.0.get(key)
    }

    pub fn subscribe(&mut self, key: Key, chat_id: ChatId, state: BioauthNotificationState) {
        let entry = self.0.entry(key);

        match entry {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let chat_ids_set = entry.get_mut();
                chat_ids_set.insert(chat_id, state);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut states_map = HashMap::new();
                states_map.insert(chat_id, state);
                entry.insert(states_map);
            }
        };
    }

    pub fn unsubscribe(&mut self, key: Key, chat_id: ChatId) {
        let entry = self.0.entry(key);

        match entry {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let state_map = entry.get_mut();
                state_map.remove(&chat_id);
            }
            std::collections::hash_map::Entry::Vacant(_) => {}
        };
    }

    pub fn unsubscribe_all(&mut self, chat_id: ChatId) {
        for (_, states) in self.0.iter_mut() {
            states.remove(&chat_id);
        }
    }

    pub fn iter_mut(&mut self) -> BioauthSubscriptionMapIter<'_, Key> {
        BioauthSubscriptionMapIter {
            inner: self.0.iter_mut(),
        }
    }
}

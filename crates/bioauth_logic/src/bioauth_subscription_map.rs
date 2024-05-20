use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::ChatId;

#[derive(Debug)]
pub struct BioauthSubscriptionMap<Key>(HashMap<Key, HashSet<ChatId>>);

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

impl<Key> BioauthSubscriptionMap<Key>
where
    Key: Hash + Eq,
{
    pub fn get(&self, key: &Key) -> Option<&HashSet<ChatId>> {
        let chats = self.0.get(key);
        chats
    }

    pub fn subscribe(&mut self, key: Key, chat_id: ChatId) {
        let entry = self.0.entry(key);

        match entry {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let chat_ids_set = entry.get_mut();
                chat_ids_set.insert(chat_id);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut chat_ids_set = HashSet::new();
                chat_ids_set.insert(chat_id);
                entry.insert(chat_ids_set);
            }
        };
    }

    pub fn unsubscribe(&mut self, key: Key, chat_id: ChatId) {
        let entry = self.0.entry(key);

        match entry {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let chat_ids_set = entry.get_mut();
                chat_ids_set.remove(&chat_id);
            }
            std::collections::hash_map::Entry::Vacant(_) => {}
        };
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Key, HashSet<ChatId>> {
        self.0.iter()
    }
}

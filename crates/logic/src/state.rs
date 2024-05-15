use std::{collections::HashMap, hash::Hash};

pub type ChatId = i64;

#[derive(Debug)]
pub struct State<ValidatorPublicKey> {
    pub validators_subscription_map: ValidatorsSubscriptionMap<ValidatorPublicKey>,
}

#[derive(Debug)]
pub struct ValidatorsSubscriptionMap<ValidatorPublicKey>(HashMap<ValidatorPublicKey, Vec<ChatId>>);

impl<ValidatorPublicKey> Default for ValidatorsSubscriptionMap<ValidatorPublicKey> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<ValidatorPublicKey> ValidatorsSubscriptionMap<ValidatorPublicKey> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<ValidatorPublicKey> ValidatorsSubscriptionMap<ValidatorPublicKey>
where
    ValidatorPublicKey: Hash + Eq,
{
    pub fn get(&self, key: &ValidatorPublicKey) -> &[ChatId] {
        let chat_ids = self.0.get(key);
        match chat_ids {
            Some(chat_ids) => chat_ids,
            None => &[],
        }
    }

    pub fn subscribe(&mut self, key: ValidatorPublicKey, chat_id: ChatId) {
        let entry = self.0.entry(key);

        match entry {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let chat_ids = entry.get_mut();
                chat_ids.push(chat_id);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(vec![chat_id]);
            }
        };
    }

    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, ValidatorPublicKey, std::vec::Vec<ChatId>> {
        self.0.iter()
    }
}

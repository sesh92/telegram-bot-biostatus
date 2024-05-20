#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{collections::HashMap, hash::Hash};

use bioauth_subscription_map::BioauthSubscriptionMap;

pub type ChatId = i64;

mod bioauth_subscription_map;

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
pub struct NewBlockParams<BioauthPublicKey> {
    pub block_number: u32,
    pub active_authentications_map: HashMap<BioauthPublicKey, u64>,
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
            bioauth_subscription_map.subscribe(bioauth.bioauth_public_key, bioauth.t_chat_id);
        }

        BioauthLogic {
            bioauth_subscription_map,
        }
    }

    pub fn new_block(
        &mut self,
        params: NewBlockParams<BioauthPublicKey>,
    ) -> (HashMap<ChatId, HashMap<BioauthPublicKey, Option<u64>>>, u32) {
        let NewBlockParams {
            block_number,
            active_authentications_map,
        } = params;

        let mut chats_with_subscribtions_map = HashMap::new();

        for (bioauth_public_key, chat_ids) in self.bioauth_subscription_map.iter() {
            let active_authentication = active_authentications_map.get(bioauth_public_key).copied();

            for chat_id in chat_ids {
                let entry = chats_with_subscribtions_map.entry(*chat_id);

                entry
                    .and_modify(
                        |subscribtions: &mut HashMap<BioauthPublicKey, Option<u64>>| {
                            subscribtions.insert(*bioauth_public_key, active_authentication);
                        },
                    )
                    .or_insert({
                        let mut res = HashMap::new();
                        res.insert(*bioauth_public_key, active_authentication);
                        res
                    });
            }
        }

        (chats_with_subscribtions_map, block_number)
    }

    pub fn update_subscription(&mut self, params: UpdateSubscriptionParams<BioauthPublicKey>) {
        let UpdateSubscriptionParams {
            t_chat_id,
            bioauth_public_key,
        } = params;

        self.bioauth_subscription_map
            .subscribe(bioauth_public_key, t_chat_id);
    }
}

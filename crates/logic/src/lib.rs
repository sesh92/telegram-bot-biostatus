#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{collections::HashMap, hash::Hash};

use state::{ChatId, State};

use crate::state::ValidatorsSubscriptionMap;

mod state;

#[derive(Debug)]
pub struct Logic<ValidatorPublicKey> {
    pub state: State<ValidatorPublicKey>,
}

#[derive(Debug)]
pub struct InitParamValidator<ValidatorPublicKey> {
    pub validator_public_key: ValidatorPublicKey,
    pub t_chat_id: ChatId,
}

#[derive(Debug)]
pub struct InitParams<ValidatorPublicKey> {
    pub validators: Vec<InitParamValidator<ValidatorPublicKey>>,
}

#[derive(Debug)]
pub struct NewBlockParams<ValidatorPublicKey> {
    pub block_number: u32,
    pub active_authentications_map: HashMap<ValidatorPublicKey, u64>,
}

#[derive(Debug)]
pub struct UpdateSubscriptionParams<ValidatorPublicKey> {
    pub t_chat_id: ChatId,
    pub validator_public_key: Option<ValidatorPublicKey>,
}

impl<ValidatorPublicKey> Logic<ValidatorPublicKey>
where
    ValidatorPublicKey: Eq + Hash,
{
    pub fn init(params: InitParams<ValidatorPublicKey>) -> Self {
        tracing::info!("Logic init");
        let mut validators_subscription_map = ValidatorsSubscriptionMap::new();

        for validator in params.validators {
            validators_subscription_map
                .subscribe(validator.validator_public_key, validator.t_chat_id);
        }

        let state = State {
            validators_subscription_map,
        };

        Logic { state }
    }

    pub fn new_block(&mut self, params: NewBlockParams<ValidatorPublicKey>) -> Vec<ChatId> {
        let NewBlockParams {
            block_number: _,
            active_authentications_map,
        } = params;

        let mut chats_to_notify = vec![];

        for (validator_public_key, t_chat_ids) in self.state.validators_subscription_map.iter() {
            let active_authentication = active_authentications_map.get(validator_public_key);

            match active_authentication {
                None => {
                    for t_chat_id in t_chat_ids {
                        tracing::info!(
                            message = "Detect active authentication expired",
                            ?t_chat_id
                        );
                        chats_to_notify.push(*t_chat_id);
                    }
                }
                Some(_) => {}
            }
        }

        chats_to_notify
    }

    pub fn update_subscription(&mut self, params: UpdateSubscriptionParams<ValidatorPublicKey>) {
        let UpdateSubscriptionParams {
            t_chat_id,
            validator_public_key,
        } = params;

        match validator_public_key {
            Some(key) => self
                .state
                .validators_subscription_map
                .subscribe(key, t_chat_id),
            None => todo!(),
        };
    }
}

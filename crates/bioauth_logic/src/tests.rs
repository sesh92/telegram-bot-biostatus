use std::collections::HashMap;

use crate::{BioauthLogic, InitParamBioauth, InitParams, NewBlockParams, UpdateSubscriptionParams};

#[test]
fn process_block_with_empty_bioauths() {
    let bioauths_features = vec![];

    let mut logic = BioauthLogic::<usize>::init(InitParams {
        bioauths: bioauths_features,
    });

    let result = logic.new_block(NewBlockParams {
        active_authentications_map: HashMap::new(),
        block_number: 0,
    });

    assert_eq!(result.0.keys().len(), 0);
}

#[test]
fn process_block_with_empty_active_authentications() {
    let bioauths_features = vec![InitParamBioauth {
        bioauth_public_key: 123,
        t_chat_id: 1,
    }];

    let mut logic = BioauthLogic::init(InitParams {
        bioauths: bioauths_features,
    });

    let result = logic.new_block(NewBlockParams {
        active_authentications_map: HashMap::new(),
        block_number: 0,
    });

    assert_eq!(result.0.keys().len(), 1);
    let value = result.0.get(&1).unwrap();
    assert_eq!(value.keys().len(), 1);
    assert_eq!(value.get(&1), None);
}

#[test]
fn process_block_with_empty_active_authentications_and_pre_update_subscription() {
    let mut logic = BioauthLogic::<usize>::init(InitParams { bioauths: vec![] });

    logic.update_subscription(UpdateSubscriptionParams {
        bioauth_public_key: 123,
        t_chat_id: 1,
    });

    let result = logic.new_block(NewBlockParams {
        active_authentications_map: HashMap::new(),
        block_number: 0,
    });

    assert_eq!(result.0.keys().len(), 1);
    let value = result.0.get(&1).unwrap();
    assert_eq!(value.keys().len(), 1);
    assert_eq!(value.get(&1), None);
}

#[test]
fn process_block_with_active_authentication() {
    let mut logic = BioauthLogic::<usize>::init(InitParams { bioauths: vec![] });

    logic.update_subscription(UpdateSubscriptionParams {
        bioauth_public_key: 123,
        t_chat_id: 1,
    });

    let mut active_authentications_map = HashMap::new();

    active_authentications_map.insert(123, 456);

    let (map, block_number) = logic.new_block(NewBlockParams {
        active_authentications_map,
        block_number: 0,
    });

    assert_eq!(map.keys().len(), 1);
    assert_eq!(block_number, 0);
    let value = map.get(&1).unwrap();
    assert_eq!(value.keys().len(), 1);
    assert_eq!(value.get(&123), Some(Some(456)).as_ref());
}

use crate::{BioauthLogic, InitParams, NewBlockParams, Notification, UpdateSubscriptionParams};
use bioauth_settings::BioauthSettingsMap;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing_test::traced_test;

#[test]
#[traced_test]
fn process_block() {
    let mut logic = BioauthLogic::<usize>::init(InitParams { bioauths: vec![] });
    let bioauth_settings_map = BioauthSettingsMap::new();
    let mut active_authentications_map = HashMap::new();

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();

    let notifications = logic.new_block(NewBlockParams {
        block_number: 1,
        active_authentications_map: &active_authentications_map,
        bioauth_settings_map: &bioauth_settings_map,
    });

    assert_eq!(notifications.len(), 0);

    let bioauth_public_key_0 = 0;
    let t_chat_id_0 = 0;

    logic.update_subscription(UpdateSubscriptionParams {
        bioauth_public_key: bioauth_public_key_0,
        t_chat_id: t_chat_id_0,
    });

    let notifications = logic.new_block(NewBlockParams {
        block_number: 2,
        active_authentications_map: &active_authentications_map,
        bioauth_settings_map: &bioauth_settings_map,
    });

    assert_eq!(notifications.len(), 1);

    for notification in notifications {
        match notification {
            Notification::BioauthLostNotification { chat_id } => {
                assert_eq!(chat_id, t_chat_id_0);
            }
            _ => panic!(),
        }
    }

    active_authentications_map.insert(0, timestamp + 1000 * 60000);

    let notifications = logic.new_block(NewBlockParams {
        block_number: 3,
        active_authentications_map: &active_authentications_map,
        bioauth_settings_map: &bioauth_settings_map,
    });

    assert_eq!(notifications.len(), 0);

    active_authentications_map.insert(0, timestamp);

    let notifications = logic.new_block(NewBlockParams {
        block_number: 3,
        active_authentications_map: &active_authentications_map,
        bioauth_settings_map: &bioauth_settings_map,
    });

    assert_eq!(notifications.len(), 1);

    for notification in notifications {
        match notification {
            Notification::BioauthSoonExpiredAlert { chat_id } => {
                assert_eq!(chat_id, t_chat_id_0);
            }
            _ => panic!(),
        }
    }
}

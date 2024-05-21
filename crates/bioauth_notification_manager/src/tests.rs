use std::time::SystemTime;

use channel_messages::Notification;
use tracing_test::traced_test;

use crate::BioauthNotificationManager;

#[tokio::test]
async fn bioauth_notification_manager_initializing_chat() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
    let handle = telegram::NotificationHandle { tx };
    let mut bioauth_notification_manager = BioauthNotificationManager::new(handle);
    let chat_id = 1;
    bioauth_notification_manager.register(&chat_id, 60, 10, 0, None);

    bioauth_notification_manager.notify(0).await;

    let notification = rx.recv().await.unwrap();
    match notification {
        Notification::BioauthLostNotification {
            chat_id: notifying_chat_id,
        } => {
            assert_eq!(notifying_chat_id, chat_id);
        }
        _ => {
            panic!();
        }
    }
}

#[tokio::test]
#[traced_test]
async fn chat_notification() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
    let handle = telegram::NotificationHandle { tx };
    let mut bioauth_notification_manager = BioauthNotificationManager::new(handle);
    let chat_id = 1;
    let mut block_number = 1;

    bioauth_notification_manager.register(&chat_id, 60, 10, block_number, None);

    bioauth_notification_manager.notify(block_number).await;

    let notification = rx.recv().await.unwrap();
    match notification {
        Notification::BioauthLostNotification {
            chat_id: notifying_chat_id,
        } => {
            assert_eq!(notifying_chat_id, chat_id);
        }
        _ => {
            panic!();
        }
    }

    block_number += 1;
    bioauth_notification_manager.register(&chat_id, 60, 10, block_number, None);
    bioauth_notification_manager.notify(block_number).await;

    let notification_res = rx.try_recv();

    match notification_res {
        Err(error) => {
            assert!(tokio::sync::mpsc::error::TryRecvError::Empty.eq(&error))
        }
        Ok(_) => panic!(),
    };

    block_number += 8;
    bioauth_notification_manager.register(&chat_id, 60, 10, block_number, None);
    bioauth_notification_manager.notify(block_number).await;

    let notification_res = rx.try_recv();

    match notification_res {
        Err(error) => {
            assert!(tokio::sync::mpsc::error::TryRecvError::Empty.eq(&error))
        }
        Ok(_) => panic!(),
    };

    block_number += 1;
    bioauth_notification_manager.register(&chat_id, 60, 10, block_number, None);
    bioauth_notification_manager.notify(block_number).await;

    let notification_res = rx.try_recv();

    let notification = match notification_res {
        Err(error) => {
            tracing::info!(message = "tmp", ?error);
            panic!()
        }
        Ok(val) => val,
    };

    match notification {
        Notification::BioauthLostNotification {
            chat_id: notifying_chat_id,
        } => {
            assert_eq!(notifying_chat_id, chat_id);
        }
        _ => {
            panic!();
        }
    }
}

#[tokio::test]
#[traced_test]
async fn chat_alert() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
    let handle = telegram::NotificationHandle { tx };
    let mut bioauth_notification_manager = BioauthNotificationManager::new(handle);
    let chat_id = 1;

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let now = since_the_epoch.as_secs();

    bioauth_notification_manager.register(&chat_id, 60, 10, 1, Some(&now));
    bioauth_notification_manager.alert().await;
    tracing::info!(message = "1");
    let notification = match rx.try_recv() {
        Err(error) => {
            tracing::info!(message = "tmp", ?error);
            panic!()
        }
        Ok(val) => val,
    };

    match notification {
        Notification::BioauthSoonExpiredAlert {
            chat_id: notifying_chat_id,
        } => {
            assert_eq!(notifying_chat_id, chat_id);
        }
        _ => {
            panic!();
        }
    }

    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let in_hour = since_the_epoch.as_secs() + 60 * 60000;

    bioauth_notification_manager.register(&chat_id, 60, 10, 1, Some(&(in_hour)));
    bioauth_notification_manager.alert().await;
    tracing::info!(message = "2");

    let notification = match rx.try_recv() {
        Err(error) => {
            tracing::info!(message = "tmp", ?error);
            panic!()
        }
        Ok(val) => val,
    };

    match notification {
        Notification::BioauthSoonExpiredAlert {
            chat_id: notifying_chat_id,
        } => {
            assert_eq!(notifying_chat_id, chat_id);
        }
        _ => {
            panic!();
        }
    }

    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let more_then_in_hour = since_the_epoch.as_secs() + 1234560000;

    bioauth_notification_manager.register(&chat_id, 60, 10, 1, Some(&(more_then_in_hour)));
    bioauth_notification_manager.alert().await;
    tracing::info!(message = "3");

    match rx.try_recv() {
        Err(error) => {
            assert!(tokio::sync::mpsc::error::TryRecvError::Empty.eq(&error))
        }
        Ok(_) => panic!(),
    };
}

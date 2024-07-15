use crate::Notification;
use bioauth_logic::FailedNotification;
use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use teloxide::{prelude::*, types::ChatId, Bot};

#[derive(Debug)]
pub struct SendNotificationError {
    pub notification: FailedNotification,
}

#[derive(Debug)]
pub struct RunLoopParams {
    pub bot: Bot,
    pub notification_handle_rx: tokio::sync::mpsc::Receiver<Notification>,
}
pub async fn run_loop(params: RunLoopParams) -> Result<(), SendNotificationError> {
    let RunLoopParams {
        mut notification_handle_rx,
        bot,
    } = params;
    loop {
        let notification = notification_handle_rx.recv().await;
        if notification.is_some() {
            tracing::info!(message = "run_loop: Got new notification", ?notification);
        }
        if let Some(notification) = notification {
            let res = match notification {
                Notification::BioauthLostNotification {
                    chat_id,
                    bioauth_public_key,
                } => {
                    let bioauth_public_key_string =
                        sp_core::crypto::AccountId32::new(bioauth_public_key)
                            .to_ss58check_with_version(
                                Ss58AddressFormatRegistry::HumanodeAccount.into(),
                            );

                    bot.send_message(
                        ChatId(chat_id),
                        format!("{bioauth_public_key_string} have lost bio-authentication to be an active validator."),
                    )
                    .await
                }
                Notification::BioauthSoonExpiredAlert {
                    chat_id,
                    bioauth_public_key,
                } => {
                    let bioauth_public_key_string =
                        sp_core::crypto::AccountId32::new(bioauth_public_key)
                            .to_ss58check_with_version(
                                Ss58AddressFormatRegistry::HumanodeAccount.into(),
                            );

                    bot.send_message(
                        ChatId(chat_id),
                        format!("{bioauth_public_key_string} will lost bio-authentication soon"),
                    )
                    .await
                }
            };

            if let Err(error) = res {
                tracing::error!(message = "notifier error", ?error);
            };
        };
    }
}

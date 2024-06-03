use crate::Notification;
use bioauth_logic::FailedNotification;
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
        tracing::info!(message = "run_loop: Got new notification", ?notification);
        if let Some(notification) = notification {
            let res = match notification {
                Notification::BioauthLostNotification { chat_id } => {
                    bot.send_message(
                        ChatId(chat_id),
                        "You have lost bio-authentication to be an active validator.",
                    )
                    .await
                }
                Notification::BioauthSoonExpiredAlert { chat_id } => {
                    bot.send_message(ChatId(chat_id), "You will lost bio-authentication soon")
                        .await
                }
            };

            if let Err(error) = res {
                tracing::error!(message = "notifier error", ?error);
            };
        };
    }
}

use std::sync::Arc;

use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

use super::{
    utils::{HandlerError, HandlerResult},
    State as GlobalState,
};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "notify subscribers")]
    AdminNotify { text: String },
}

async fn start(
    bot: Bot,
    message: Message,
    admin_ids: Vec<i64>,
    text: String,
    rw_team_notification_subscription_map: Arc<
        tokio::sync::RwLock<dev_subscriptions::DevSubscriptionMap>,
    >,
) -> HandlerResult {
    let admin_chat_id = message.chat.id;

    if admin_ids.contains(&admin_chat_id.0) {
        bot.send_message(admin_chat_id, "Admin command got").await?;
        let team_notification_subscription_map = rw_team_notification_subscription_map.read().await;
        let subscriber_ids =
            team_notification_subscription_map.get_all_enabled_team_notification_subscribers();
        for chat_id in subscriber_ids.clone() {
            bot.send_message(ChatId(chat_id), text.clone()).await?;
        }

        bot.send_message(
            admin_chat_id,
            format!("Sent this text to {:?}", subscriber_ids),
        )
        .await?;
    }

    Ok(())
}

pub fn schema() -> UpdateHandler<HandlerError> {
    dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
            .filter_command::<Command>()
            .branch(dptree::case![Command::AdminNotify { text }].endpoint(start)),
    )
}

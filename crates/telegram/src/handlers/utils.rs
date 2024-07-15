use teloxide::{
    prelude::*,
    types::{BotCommand, BotCommandScope},
    Bot,
};

pub type HanderError = Box<dyn std::error::Error + Send + Sync>;
pub type HandlerResult = Result<(), HanderError>;

/// Set new commands for a given local context deduced from the message.
pub async fn set_local_commands(
    chat_id: ChatId,
    bot: &Bot,
    commands: Vec<BotCommand>,
) -> HandlerResult {
    let chat_id = chat_id.into();
    bot.set_my_commands(commands)
        .scope(BotCommandScope::Chat { chat_id })
        .send()
        .await?;
    Ok(())
}

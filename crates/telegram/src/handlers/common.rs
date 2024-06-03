use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

use super::{
    utils::{set_local_commands, HanderError, HandlerResult},
    Command, State,
};

/// Dialogue start.
const START_MESSAGE: &str = {
    "
    Welcome to the biostatus bot!

    Here you can subscribe to bio authentications to get notifications then it expires and also to get alert before expiration.

    Use the /managevalidatorsubscriptions command to manage your subscriptions

    Use /help to display bot usage instructions.
"
};

async fn start(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, START_MESSAGE).await?;
    Ok(())
}

async fn reset_state(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, "Reseting state").await?;

    set_local_commands(&message, &bot, Command::bot_commands()).await
}

async fn default_callback_handler(bot: Bot, callback: CallbackQuery) -> HandlerResult {
    bot.answer_callback_query(callback.id)
        .text("Button won't work at this state. Force exit from this state by using /exit command")
        .show_alert(true)
        .await?;
    Ok(())
}

pub fn schema() -> UpdateHandler<HanderError> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<State>, State>()
                .branch(
                    dptree::case![State::Start]
                        .filter_command::<Command>()
                        .branch(dptree::case![Command::Start].endpoint(start))
                        .branch(dptree::case![Command::ResetState].endpoint(reset_state))
                        .branch(dptree::case![Command::Help].endpoint(start)),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<State>, State>()
                .branch(
                    dptree::filter(|state| !matches!(state, State::Start))
                        .endpoint(default_callback_handler),
                ),
        )
}

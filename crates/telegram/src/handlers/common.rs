use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

use super::{
    utils::{HandlerError, HandlerResult},
    Command, GlobalDialogue, State as GlobalState,
};

const START_MESSAGE: &str = {
    "

!!! PAY ATTENTION !!!
!!! THIS IS EARLY ACCESS VERSION !!!
!!! SOME FUNCTIONALITY CAN BE UNSTABLE !!!
!!! SOMETIMES BOT CAN BE FULLY RESET, BUT I WILL NOTIFY YOU ABOUT THAT IF YOU ACTIVATE DEVELOPER NOTIFICATION !!!

Welcome to the biostatus bot!

By using this bot, you will receive timely notifications regarding the impending loss of your validator status according to your settings.

If your validator status is lost, or you are no longer a validator, you will be regularly notified based on your settings.

All bot messages are customizable to your liking, ensuring that you won't need to mute the bot and can stay informed about your status, allowing for immediate action.

It's also recommended to enable developer notifications to stay updated on any network changes that might affect your validator status, you can always unsubscribe from developer notifications if you find them irrelevant.

Use /help command to display bot usage instructions.
"
};

async fn start(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, START_MESSAGE).await?;
    Ok(())
}

async fn help(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn reset_state(bot: Bot, dialogue: GlobalDialogue, message: Message) -> HandlerResult {
    let chat_id = message.chat.id;
    bot.send_message(chat_id, "Resetting state").await?;

    super::transition_to_start(chat_id, &bot, dialogue).await
}

pub fn schema() -> UpdateHandler<HandlerError> {
    dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
            .branch(
                dptree::case![GlobalState::ManageValidatorSubscriptions(x)]
                    .filter_command::<Command>()
                    .branch(dptree::case![Command::ResetState].endpoint(reset_state)),
            )
            .branch(
                dptree::case![GlobalState::Start]
                    .filter_command::<Command>()
                    .branch(dptree::case![Command::Start].endpoint(start))
                    .branch(dptree::case![Command::Help].endpoint(help)),
            ),
    )
}

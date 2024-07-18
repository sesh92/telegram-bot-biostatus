use teloxide::{
    dispatching::{dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

use super::{
    unsubscribe, update_alert_before_expiration_in_mins, update_max_message_frequency_in_blocks,
    State as GlobalState,
};

use super::manage_validator_subscriptions;
use super::utils::{set_local_commands, HandlerError, HandlerResult};
use super::GlobalDialogue;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are available while sending number:"
)]
pub enum UpdateMaxMessageFrequencyInBlocksCommand {
    #[command(description = "display this text")]
    Help,
    #[command(description = "cancel the operation")]
    Cancel,
}

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text")]
    Help,
    #[command(
        description = "update the maximum message frequency (in block est. ~6sec per block) for this subscription"
    )]
    UpdateMaxMessageFrequency,
    #[command(description = "adjust the alert time (in minutes) before losing validator status")]
    UpdateAlertBefore,
    #[command(description = "unsubscribe from this subscription")]
    Unsubscribe,
    #[command(description = "cancel the operation")]
    Cancel,
}

pub async fn transition_to_update_subscription(
    chat_id: ChatId,
    bot: &Bot,
    address: String,
    dialogue: GlobalDialogue,
) -> HandlerResult {
    dialogue
        .update(GlobalState::ManageValidatorSubscriptions(
            manage_validator_subscriptions::State::UpdateSubscription {
                address: address.clone(),
            },
        ))
        .await?;
    set_local_commands(chat_id, bot, Command::bot_commands()).await
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

const CANCEL_MESSAGE: &str = {
    "
You have canceled the action.

use /help command to display bot usage instructions.
"
};

pub async fn cancel(bot: Bot, msg: Message, dialogue: GlobalDialogue) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, CANCEL_MESSAGE).await?;

    super::transition_to_start(chat_id, &bot, dialogue).await
}

pub fn schema() -> UpdateHandler<HandlerError> {
    let update_commands = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Help].endpoint(help))
        .branch(
            dptree::case![Command::UpdateAlertBefore]
                .endpoint(update_alert_before_expiration_in_mins::command),
        )
        .branch(
            dptree::case![Command::UpdateMaxMessageFrequency]
                .endpoint(update_max_message_frequency_in_blocks::command),
        )
        .branch(dptree::case![Command::Unsubscribe].endpoint(unsubscribe::command))
        .branch(dptree::case![Command::Cancel].endpoint(cancel));

    Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
        .branch(
            dptree::case![GlobalState::ManageValidatorSubscriptions(x)].branch(
                dptree::case![manage_validator_subscriptions::State::UpdateSubscription {
                    address
                }]
                .branch(update_commands),
            ),
        )
        .branch(unsubscribe::schema())
        .branch(update_alert_before_expiration_in_mins::schema())
        .branch(update_max_message_frequency_in_blocks::schema())
}

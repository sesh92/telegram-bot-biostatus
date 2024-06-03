//! Response message constants.

use indoc::indoc;

/// For the `/start` command.
pub(crate) const MESSAGE_WELCOME: &str = indoc! {"
    Welcome to the Verification checker!

    Here you can setup the bot to notifications when your bio verifications are expired.

    Use the /activatefeatures command to choose which features you want to
    Use /help to display bot usage instructions.
"};

/// For an unknown command.
pub(crate) const MESSAGE_OTHER: &str = indoc! {"
    Unknown command. Try /help for list of commands.
"};

/// For the `/setvalidatoraddress` command.
pub(crate) const MESSAGE_SET_VALIDATOR_ADDRESS: &str = indoc! {"
    Started watching the validator status.
"};

/// For activate features dialogue start.
pub(crate) const MESSAGE_MANAGE_VALIDATOR_SUBSCRIPTION_DIALOGUE_START: &str = indoc! {"
    Use the /subscribeToValidator command to subscribe to validator status
    format: /<cmd> <(hm...): address>

    Use the /updateMessageFrequencyInBlocks command to update subscription settings about max message frequency in blocks
    format: /<cmd> <(hm...): address> <number: blocks>

    Use the /updateAlertBeforeExpirationInMins command to update subscription settings about alert before expiration in mins
    format: /<cmd> <(hm...): address> <number: blocks>

    Use /help to display bot usage instructions.
"};

/// For a dialogue cancellation.
pub(crate) const MESSAGE_DIALOGUE_CANCEL: &str = indoc! {"
    Ok
"};

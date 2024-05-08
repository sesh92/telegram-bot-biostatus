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

/// For the `/setbiomapperaddress` command.
pub(crate) const MESSAGE_SET_BIOMAPPER_ADDRESS: &str = indoc! {"
    At this moment this feature is unsupported.
"};

/// For the `/resetallfeatures` command.
pub(crate) const MESSAGE_RESET_ALL_FEATURES: &str = indoc! {"
    Your features were reset.
"};

/// For settings dialogue start.
pub(crate) const MESSAGE_SETTINGS_DIALOGUE_START: &str = indoc! {"
    Let's update the settings.
"};

/// For activate features dialogue start.
pub(crate) const MESSAGE_ACTIVATE_FEATURES_DIALOGUE_START: &str = indoc! {"
    Let's choose the features you want to activate.
"};

/// For a dialogue cancellation.
pub(crate) const MESSAGE_DIALOGUE_CANCEL: &str = indoc! {"
    Ok
"};

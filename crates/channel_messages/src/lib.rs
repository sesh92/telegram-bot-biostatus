//! The chain interaction primitives and settings.
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

#[derive(Debug)]
pub enum Notification {
    BioauthLostNotification { chat_id: i64 },
    BioauthSoonExpiredAlert { chat_id: i64 },
}

#[derive(Debug)]
pub enum SystemMessage {
    HumanodeTeamMessage { message: String },
}

#[derive(Debug)]
pub enum TelegramMessage {
    Start {
        chat_id: i64,
    },
    SetValidatorPublicKey {
        chat_id: i64,
        public_key: Option<String>,
    },
    SetBiomapperPublicKey {
        chat_id: i64,
        public_key: Option<String>,
    },
    SetHumanodeTeamMessage {
        chat_id: i64,
        value: bool,
    },
    SetValidatorFrequencyInBlocks {
        chat_id: i64,
        validator_frequency_in_blocks: i32,
    },
    SetBiomapperFrequencyInBlocks {
        chat_id: i64,
        validator_frequency_in_blocks: i32,
    },
}

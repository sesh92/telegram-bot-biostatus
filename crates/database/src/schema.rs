// @generated automatically by Diesel CLI.

diesel::table! {
    bioauth_subscriptions (t_chat_id, validator_public_key) {
        t_chat_id -> Int8,
        validator_public_key -> Bytea,
        max_message_frequency_in_blocks -> Int4,
        alert_before_expiration_in_mins -> Int8,
    }
}

// @generated automatically by Diesel CLI.

diesel::table! {
    bioauth_subscriptions (t_chat_id, validator_public_key) {
        t_chat_id -> Int8,
        validator_public_key -> Bytea,
        max_message_frequency_in_blocks -> Int4,
        alert_before_expiration_in_mins -> Int8,
    }
}

diesel::table! {
    dev_subscriptions (t_chat_id) {
        t_chat_id -> Int8,
        affected_validator -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    bioauth_subscriptions,
    dev_subscriptions,
);

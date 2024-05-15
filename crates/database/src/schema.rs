// @generated automatically by Diesel CLI.

diesel::table! {
    account_settings (t_chat_id) {
        t_chat_id -> Int8,
        f_humanode_team_notifications -> Bool,
        s_active_validator_frequency_in_blocks -> Int4,
        s_active_validator_alert_before_in_mins -> Int4,
        s_biomapper_frequency_in_blocks -> Int4,
        s_biomapper_alert_before_in_mins -> Int4,
        validator_public_key -> Nullable<Bytea>,
        #[max_length = 40]
        biomapper_public_key -> Nullable<Bpchar>,
    }
}

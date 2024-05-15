-- Your SQL goes here
CREATE TABLE account_settings (
    t_chat_id BIGINT NOT NULL PRIMARY KEY,
    f_humanode_team_notifications BOOLEAN NOT NULL DEFAULT 'f',
    s_active_validator_frequency_in_blocks INTEGER NOT NULL DEFAULT 10,
    s_active_validator_alert_before_in_mins INTEGER NOT NULL DEFAULT 60,
    s_biomapper_frequency_in_blocks INTEGER NOT NULL DEFAULT 10,
    s_biomapper_alert_before_in_mins INTEGER NOT NULL DEFAULT 60,
    validator_public_key BYTEA,
    biomapper_public_key CHAR(40)
);


-- Your SQL goes here
CREATE TABLE bioauth_subscriptions (
    t_chat_id BIGINT NOT NULL,
    validator_public_key BYTEA NOT NULL,
    max_message_frequency_in_blocks INT NOT NULL DEFAULT 10,
    alert_before_expiration_in_mins BIGINT NOT NULL DEFAULT 60,
    PRIMARY KEY (t_chat_id, validator_public_key)
);

CREATE TABLE dev_subscriptions (
  t_chat_id BIGINT NOT NULL PRIMARY KEY,
  affected_validator BOOLEAN NOT NULL DEFAULT 'f'
);

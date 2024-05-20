//! Models for interaction with the database.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use crate::schema::bioauth_subscriptions;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    prelude::*,
    sql_types::Bytea,
};

#[derive(Debug)]
pub struct ByteArray<const N: usize>([u8; N]);

impl<DB, const N: usize> Queryable<Bytea, DB> for ByteArray<N>
where
    DB: Backend,
    Vec<u8>: FromSql<Bytea, DB>,
{
    type Row = Vec<u8>;

    fn build(data: Vec<u8>) -> deserialize::Result<Self> {
        if data.len() != N {
            return Err(
                anyhow::format_err!("data len {} is not equal to {}", data.len(), N).into(),
            );
        }
        let mut tmp = [0u8; N];
        tmp.copy_from_slice(&data[..N]);
        Ok(ByteArray(tmp))
    }
}

impl<const N: usize> From<ByteArray<N>> for [u8; N] {
    fn from(value: ByteArray<N>) -> Self {
        value.0
    }
}

/// Model for load init validator with settings values.
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = bioauth_subscriptions)]
pub struct LoadForInitialization {
    /// The telegram user's chat id.
    pub t_chat_id: i64,

    /// Validator public key
    #[diesel(deserialize_as = ByteArray<32>)]
    pub validator_public_key: [u8; 32],

    /// Frequency in blocks for notifying active validator.
    #[diesel(deserialize_as = i32)]
    pub max_message_frequency_in_blocks: u32,

    /// Notify a few minutes before expiration.
    #[diesel(deserialize_as = i64)]
    pub alert_before_expiration_in_mins: u64,
}

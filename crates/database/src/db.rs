//! Manager implementation.
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use crate::models::LoadForInitialization;

use diesel::prelude::*;
use diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection, RunQueryDsl};

#[derive(Debug)]
/// The bioauth_subscriptions manager.
pub struct Db {
    /// Pool.
    pub pool: Pool<AsyncPgConnection>,
}

/// Db implementation.
impl Db {
    pub async fn load_for_initialization(
        &self,
    ) -> Result<Vec<LoadForInitialization>, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::bioauth_subscriptions::dsl::*;

        let values = bioauth_subscriptions
            .select(LoadForInitialization::as_select())
            .get_results(&mut conn)
            .await?;

        Ok(values)
    }

    pub async fn bioauth_subscribe(
        &self,
        chat_id: i64,
        public_key: &[u8; 32],
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::bioauth_subscriptions::dsl::*;
        let public_key: &[u8] = &public_key[..];

        diesel::insert_into(bioauth_subscriptions)
            .values((t_chat_id.eq(chat_id), validator_public_key.eq(public_key)))
            .on_conflict((t_chat_id, validator_public_key))
            .do_update()
            .set(validator_public_key.eq(public_key))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn bioauth_unsubscribe(
        &self,
        chat_id: i64,
        public_key: &[u8; 32],
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::bioauth_subscriptions::dsl::*;
        let public_key: &[u8] = &public_key[..];

        diesel::delete(bioauth_subscriptions)
            .filter(
                t_chat_id
                    .eq(chat_id)
                    .and(validator_public_key.eq(public_key)),
            )
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn bioauth_unsubscribe_all(&self, chat_id: i64) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::bioauth_subscriptions::dsl::*;

        diesel::delete(bioauth_subscriptions)
            .filter(t_chat_id.eq(chat_id))
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}

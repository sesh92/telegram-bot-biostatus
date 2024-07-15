//! Manager implementation.
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use crate::models::{AllDevSubscriptions, LoadForInitialization};

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
    pub async fn load_all_team_subscriptions(
        &self,
    ) -> Result<Vec<AllDevSubscriptions>, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::dev_subscriptions::dsl::*;

        let values = dev_subscriptions
            .select(AllDevSubscriptions::as_select())
            .get_results(&mut conn)
            .await?;

        Ok(values)
    }

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

    pub async fn update_affected_validator_subscription(
        &self,
        chat_id: i64,
        enable: bool,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::dev_subscriptions::dsl::*;

        diesel::insert_into(dev_subscriptions)
            .values((t_chat_id.eq(chat_id), affected_validator.eq(enable)))
            .on_conflict(t_chat_id)
            .do_update()
            .set(affected_validator.eq(enable))
            .execute(&mut conn)
            .await?;

        Ok(())
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

    pub async fn update_bioauth_max_message_frequency_in_blocks(
        &self,
        chat_id: i64,
        public_key: &[u8; 32],
        max_message_frequency_in_blocks_value: i32,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::bioauth_subscriptions::dsl::*;
        let public_key: &[u8] = &public_key[..];

        diesel::update(bioauth_subscriptions)
            .filter(
                t_chat_id
                    .eq(chat_id)
                    .and(validator_public_key.eq(public_key)),
            )
            .set(max_message_frequency_in_blocks.eq(max_message_frequency_in_blocks_value))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn update_bioauth_alert_before_expiration_in_mins(
        &self,
        chat_id: i64,
        public_key: &[u8; 32],
        alert_before_expiration_in_mins_value: i64,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::bioauth_subscriptions::dsl::*;
        let public_key: &[u8] = &public_key[..];

        diesel::update(bioauth_subscriptions)
            .filter(
                t_chat_id
                    .eq(chat_id)
                    .and(validator_public_key.eq(public_key)),
            )
            .set(alert_before_expiration_in_mins.eq(alert_before_expiration_in_mins_value))
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

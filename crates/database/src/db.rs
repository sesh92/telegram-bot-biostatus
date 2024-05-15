//! Manager implementation.
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use crate::models::InitValidator;

use diesel::prelude::*;
use diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection, RunQueryDsl};

#[derive(Debug)]
/// The account_settings manager.
pub struct Db {
    /// Pool.
    pub pool: Pool<AsyncPgConnection>,
}

/// Db implementation.
impl Db {
    pub async fn load_init_validators(&self) -> Result<Vec<InitValidator>, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::account_settings::dsl::*;

        let values = account_settings
            .filter(validator_public_key.is_not_null())
            .select(InitValidator::as_select())
            .get_results(&mut conn)
            .await?;

        Ok(values)
    }

    /// Set `validator_public_key` for `t_chat_id`.
    pub async fn set_validator_public_key(
        &self,
        chat_id: i64,
        public_key: Option<&[u8; 32]>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::account_settings::dsl::*;
        let public_key = public_key.map(|x| &x[..]);

        diesel::insert_into(account_settings)
            .values((t_chat_id.eq(chat_id), validator_public_key.eq(public_key)))
            .on_conflict(t_chat_id)
            .do_update()
            .set(validator_public_key.eq(public_key))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    /// Start a new account with default settings.
    // pub async fn start(&self, chat_id: i64) -> Result<(), anyhow::Error> {
    //     let mut conn = self.pool.get().await?;
    //     use crate::schema::account_settings::dsl::*;

    //     let res = diesel::insert_into(account_settings)
    //         .values(NewAccountSettings {
    //             t_chat_id: chat_id,
    //             ..Default::default()
    //         })
    //         .execute(&mut conn)
    //         .await;

    //     match res {
    //         Ok(_) => Ok(()),
    //         Err(e) => Err(anyhow::Error::new(e)),
    //     }
    // }

    /// Set `validator_frequency_in_blocks` field.
    pub async fn set_validator_frequency_in_blocks(
        &self,
        chat_id: i64,
        validator_frequency_in_blocks: i32,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::account_settings::dsl::*;

        diesel::update(account_settings.filter(t_chat_id.eq(&chat_id)))
            .set(s_active_validator_frequency_in_blocks.eq(validator_frequency_in_blocks))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    /// Set `validator_frequency_in_blocks` field.
    pub async fn set_humanode_team_message(
        &self,
        chat_id: i64,
        value: bool,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        use crate::schema::account_settings::dsl::*;

        diesel::update(account_settings.filter(t_chat_id.eq(&chat_id)))
            .set(f_humanode_team_notifications.eq(value))
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}

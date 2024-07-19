//! The entrypoint to the biostatus.

#![allow(missing_docs)]

use std::sync::Arc;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;

use teloxide::{
    dispatching::dialogue::{serializer::Bincode, RedisStorage, Storage},
    dispatching::ShutdownToken,
    requests::Requester,
};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    let rpc_url: String = envfury::must("RPC_URL")?;
    let redis_url: String = envfury::must("REDIS_URL")?;
    let telegram_token: String = envfury::must("TELOXIDE_TOKEN")?;
    let database_url: String = envfury::must("DATABASE_URL")?;
    let admin_chat_ids_str: String = envfury::must("ADMIN_CHAT_IDS")?;
    let admin_chat_ids = admin_chat_ids_str
        .split(',')
        .map(|id| id.parse::<i64>().unwrap())
        .collect();

    let reqwest = teloxide::net::default_reqwest_settings().build()?;
    let storage = RedisStorage::open(redis_url, Bincode)
        .await
        .unwrap()
        .erase();
    let bot = teloxide::Bot::with_client(telegram_token, reqwest);
    let me = bot.get_me().await?;

    tracing::info!(message = "Bot info", ?me);

    let db_pool = {
        let manager =
            AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
        diesel_async::pooled_connection::bb8::Pool::builder()
            .build(manager)
            .await
            .unwrap()
    };
    let db = database::db::Db { pool: db_pool };
    let api = block_subscription::BlockSubscription::construct_api(rpc_url).await?;
    let block_subscription = block_subscription::BlockSubscription::subscribe(api).await?;
    let bioauth_settings_map = bioauth_settings::BioauthSettingsMap::new();
    let rw_bioauth_settings_map = Arc::new(RwLock::new(bioauth_settings_map));
    let dev_subscriptions_map = dev_subscriptions::DevSubscriptionMap::new();
    let rw_dev_subscriptions_map = Arc::new(RwLock::new(dev_subscriptions_map));
    let telegram = telegram::Telegram {
        bot,
        storage,
        rw_bioauth_settings_map: Arc::clone(&rw_bioauth_settings_map),
        rw_dev_subscriptions_map: Arc::clone(&rw_dev_subscriptions_map),
        admin_chat_ids,
    };

    telegram.set_commands().await?;

    let (fut, shutdown_token, subscription_update_handle, telegram_notification_handle) =
        telegram.setup();

    tracing::info!("Telegram commands successfully setup");

    let mut loops = main_loop::run(main_loop::Params {
        block_subscription,
        db,
        subscription_update_handle,
        telegram_notification_handle,
        rw_bioauth_settings_map: Arc::clone(&rw_bioauth_settings_map),
        rw_dev_subscriptions_map: Arc::clone(&rw_dev_subscriptions_map),
    })
    .await?;

    tracing::info!("Main loop successfully run");

    setup_shutdown_handler(shutdown_token);
    fut.await;
    loops.shutdown().await;

    tracing::info!(message = "Shutdown complete");
    Ok(())
}

/// Setup the the system signal callbacks to trigger the graceful shutdown.
fn setup_shutdown_handler(teloxide_shutdown_token: ShutdownToken) {
    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl+C");

            if let Ok(fut) = teloxide_shutdown_token.shutdown() {
                fut.await
            }
        }
    });
}

//! The entrypoint to the biostatus.
#![allow(missing_docs)]

use teloxide::{
    dispatching::dialogue::{serializer::Bincode, RedisStorage, Storage},
    dispatching::ShutdownToken,
    requests::Requester,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    let rpc_url: String = envfury::must("RPC_URL")?;
    let redis_url: String = envfury::must("REDIS_URL")?;
    let telegram_token: String = envfury::must("TELOXIDE_TOKEN")?;

    let (chain_tx, chain_rx) = tokio::sync::mpsc::channel(100);
    let (telegram_tx, telegram_rx) = tokio::sync::mpsc::channel(100);

    let reqwest = teloxide::net::default_reqwest_settings().build()?;

    let storage = RedisStorage::open(redis_url, Bincode)
        .await
        .unwrap()
        .erase();

    let bot = teloxide::Bot::with_client(telegram_token.clone(), reqwest.clone());
    let bot_notifier = teloxide::Bot::with_client(telegram_token, reqwest);

    let me = bot.get_me().await?;

    tracing::info!(message = "Bot info", ?me);

    tokio::spawn(async move {
        let api = chain::construct_api(rpc_url).await;
        match api {
            Err(error) => tracing::error!(message = "construct_api error", ?error),
            Ok(api) => {
                if let Err(error) = chain::subscribe_active_authentications(
                    chain::SubscribeActiveAuthenticationsParams {
                        api,
                        account_settings_rx: telegram_rx,
                        account_notification_tx: chain_tx,
                    },
                )
                .await
                {
                    tracing::error!(message = "subscribe_active_authentications exited", ?error);
                }
            }
        }
    });

    let telegram = telegram::Telegram {
        bot,
        bot_notifier,
        account_settings_tx: telegram_tx,
        notification_rx: chain_rx,
        storage,
    };

    telegram.set_commands().await?;

    let (fut, shutdown_token) = telegram.setup();
    tracing::info!("Telegram commands successfully setup");

    setup_shutdown_handler(shutdown_token);
    fut.await;

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

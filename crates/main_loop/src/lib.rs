//! Main loop.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::sync::Arc;

use block_subscription::BlockSubscription;
use database::db::Db;
use logic::Logic;
use tokio::{sync::Mutex, task::JoinSet};

#[derive(Debug)]
pub struct Params {
    pub db: Db,
    pub block_subscription: BlockSubscription,
    pub telegram_notification_handler: telegram::NotificationHandle,
    pub subscription_update_handle: telegram::SubscriptionUpdateHandle,
}

pub async fn run(params: Params) -> Result<JoinSet<()>, anyhow::Error> {
    let Params {
        db,
        mut block_subscription,
        telegram_notification_handler,
        mut subscription_update_handle,
    } = params;

    let all_validator_info = db.load_init_validators().await?;

    tracing::info!(message = "Got validator", ?all_validator_info);

    let validators = all_validator_info
        .into_iter()
        .map(|validator_info| logic::InitParamValidator {
            t_chat_id: validator_info.t_chat_id,
            validator_public_key: validator_info.validator_public_key,
        })
        .collect();

    let logic = Logic::init(logic::InitParams { validators });

    let logic = Arc::new(Mutex::new(logic));

    let mut tasks = tokio::task::JoinSet::new();
    {
        let logic = Arc::clone(&logic);
        tasks.spawn(async move {
            loop {
                let block_subscription::BlockInfo {
                    block_number,
                    active_authentications_map,
                } = block_subscription.next_block().await.unwrap();

                let chat_ids = {
                    let mut logic = logic.lock().await;
                    logic.new_block(logic::NewBlockParams {
                        active_authentications_map,
                        block_number,
                    })
                };

                for chat_id in chat_ids {
                    tracing::info!(message = "Notify to", ?chat_id);

                    telegram_notification_handler
                        .send_bioauth_lost_notification(chat_id)
                        .await
                        .unwrap();
                }
            }
        });
    }

    {
        let logic = Arc::clone(&logic);
        tasks.spawn(async move {
            loop {
                let telegram::SubscriptionUpdate {
                    chat_id: t_chat_id,
                    validator_public_key,
                } = subscription_update_handle.next().await.unwrap();

                {
                    let mut logic = logic.lock().await;
                    logic.update_subscription(logic::UpdateSubscriptionParams {
                        t_chat_id,
                        validator_public_key,
                    });
                }
                tracing::info!(message = "New subscriptions", ?validator_public_key);

                db.set_validator_public_key(t_chat_id, validator_public_key.as_ref())
                    .await
                    .unwrap();
            }
        });
    }

    Ok(tasks)
}

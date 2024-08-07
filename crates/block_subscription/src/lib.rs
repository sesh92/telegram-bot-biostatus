//! The chain interaction primitives and settings.

#![allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::multiple_crate_versions
)]

use std::collections::HashMap;

use subxt::{backend::StreamOfResults, blocks::Block, OnlineClient, PolkadotConfig};

/// The generated runtime data.
mod r#gen {
    #![allow(missing_docs, clippy::too_many_arguments, clippy::enum_variant_names)]
    #[subxt::subxt(runtime_metadata_path = "../../generated/humanode_metadata.scale")]
    pub mod humanode {}
}
pub use r#gen::humanode;

#[derive(Debug)]
pub struct BlockSubscription {
    pub api: OnlineClient<PolkadotConfig>,
    pub subscription: StreamOfResults<Block<PolkadotConfig, OnlineClient<PolkadotConfig>>>,
}

#[derive(Debug)]
pub struct BlockInfo {
    pub active_authentications_map: HashMap<ValidatorPublicKey, u64>,
    pub block_number: u32,
}

#[derive(Debug)]
pub enum NewBlockError {
    BlockNotReceived,
    SubscriptionBlocksError(subxt::Error),
    ActiveAuthenticationNotReceived(subxt::Error),
}

type ValidatorPublicKey = [u8; 32];

impl BlockSubscription {
    pub async fn construct_api(url: String) -> Result<OnlineClient<PolkadotConfig>, subxt::Error> {
        let api = OnlineClient::<PolkadotConfig>::from_insecure_url(url).await?;
        Ok(api)
    }
    pub async fn subscribe(api: OnlineClient<PolkadotConfig>) -> Result<Self, subxt::Error> {
        let subscription = api.blocks().subscribe_finalized().await?;

        Ok(Self { api, subscription })
    }

    pub async fn next_block(&mut self) -> Result<BlockInfo, NewBlockError> {
        let res_opt = self.subscription.next().await;
        let mut active_authentications_map = HashMap::new();

        let res = match res_opt {
            None => return Err(NewBlockError::BlockNotReceived),
            Some(res) => res,
        };

        let block = res.map_err(NewBlockError::SubscriptionBlocksError)?;
        let block_number = block.number();

        let query = &r#gen::humanode::storage()
            .bioauth()
            .active_authentications();

        let active_authentications = block
            .storage()
            .fetch(query)
            .await
            .map_err(NewBlockError::ActiveAuthenticationNotReceived)?;

        if let Some(value) = active_authentications {
            let active_authentications = value.0;

            for active_authentication in active_authentications {
                active_authentications_map.insert(
                    active_authentication.public_key.0,
                    active_authentication.expires_at,
                );
            }
        }

        tracing::info!(message = "new block", ?block_number);

        Ok(BlockInfo {
            block_number,
            active_authentications_map,
        })
    }
}

use crate::misc::{url_to_value, Blockchain};
use crate::ApiClient;
use crate::RpcTExt;
use crate::Run;
use crate::RunResult;
use crate::StorageKind;
use async_trait::async_trait;
use clap::Args;
use color_eyre::Result;
use parity_scale_codec::Decode;

#[derive(Debug, Clone, Args)]
pub struct GetArgs {
    /// The name of the blockchain to get the RPC URL for.
    blockchain: Blockchain,
}

#[derive(Debug, Clone, Args)]
pub struct SetArgs {
    /// The blockchain to set the RPC URL for.
    blockchain: Blockchain,
    /// The RPC URL.
    rpc_url: String,
}

#[async_trait]
impl Run for GetArgs {
    async fn run(self, api: &ApiClient) -> RunResult {
        let Self { blockchain } = self;

        let value = get(api, blockchain).await?;

        println!("{value:?}");
        Ok(())
    }
}

#[async_trait]
impl Run for SetArgs {
    async fn run(self, api: &ApiClient) -> RunResult {
        let Self {
            blockchain,
            rpc_url,
        } = self;
        let key = blockchain.to_key();

        api.rpc()
            .set_offchain_storage(StorageKind::PERSISTENT, &key, &url_to_value(&rpc_url))
            .await?;

        assert_eq!(
            api.rpc()
                .offchain_storage(StorageKind::PERSISTENT, &key)
                .await?,
            Some(url_to_value(&rpc_url))
        );

        println!("{:?} -> {}", blockchain, rpc_url);
        Ok(())
    }
}

pub async fn get(api: &ApiClient, blockchain: Blockchain) -> Result<Option<String>> {
    let key = blockchain.to_key();

    let value = api
        .rpc()
        .offchain_storage(StorageKind::PERSISTENT, &key)
        .await?;

    let value = match value {
        Some(value) => Some(String::decode(&mut value.0.as_slice())?),
        None => None,
    };

    Ok(value)
}

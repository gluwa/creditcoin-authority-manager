use crate::RpcTExt;
use crate::Run;
use crate::RunResult;
use crate::RuntimeApi;
use crate::StorageKind;
use async_trait::async_trait;
use clap::{Args, Subcommand};
use parity_scale_codec::Decode;
use sp_core::storage::StorageData;
use sp_core::storage::StorageKey;
use sp_tracing::warn;

#[derive(Debug, Clone, Subcommand)]
pub enum NonceCommand {
    Reset(ResetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ResetArgs {
    ///hex-string or SS58 address
    public_hex: String,
}

#[async_trait]
impl Run for NonceCommand {
    async fn run(self, api: &RuntimeApi) -> RunResult {
        match self {
            Self::Reset(args) => args.run(api).await,
        }
    }
}

#[async_trait]
impl Run for ResetArgs {
    async fn run(self, api: &RuntimeApi) -> RunResult {
        let acc = self.public_hex;
        let rpc = api.client.rpc();
        let nonce_key = rpc.task_get_offchain_nonce_key(&acc).await?;
        let nonce = rpc
            .offchain_storage(StorageKind::PERSISTENT, &StorageKey(nonce_key.clone()))
            .await?
            .map(|StorageData(nonce)| u32::decode(&mut nonce.as_slice()));

        warn!(target: "task", "Resetting prev nonce {nonce:?} for Account {acc}");

        //reset by emptying, the OCW backend will fail to decode the value on the next read, i.e. essentially a None.
        rpc.set_offchain_storage(
            StorageKind::PERSISTENT,
            &StorageKey(nonce_key.clone()),
            &StorageData(vec![]),
        )
        .await?;
        Ok(())
    }
}

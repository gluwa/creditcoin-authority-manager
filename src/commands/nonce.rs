use crate::DefaultConfig;
use crate::Result;
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
    /// Reset the offchain nonce.
    Reset(ResetArgs),
    /// Show the current offchain nonce (best-effort)
    Show(ShowArgs),
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
            Self::Show(args) => args.run(api).await,
        }
    }
}

#[async_trait]
impl Run for ResetArgs
where
    ResetArgs: NonceCommons,
{
    async fn run(self, api: &RuntimeApi) -> RunResult {
        let rpc = api.client.rpc();
        let nonce_key = self.offchain_nonce_key(rpc).await?;
        warn!(target: "task", "Resetting prev nonce for Account {}", self.public_hex_or_ss58());

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

#[derive(Debug, Clone, Args)]
pub struct ShowArgs {
    ///hex-string or SS58 address
    public_hex: String,
}

#[async_trait]
impl Run for ShowArgs
where
    ResetArgs: NonceCommons,
{
    async fn run(self, api: &RuntimeApi) -> RunResult {
        let rpc = api.client.rpc();
        let nonce = self.offchain_nonce(rpc).await?;
        println!(
            "offchain nonce {nonce:?} for Account {}",
            self.public_hex_or_ss58()
        );
        Ok(())
    }
}

#[async_trait]
trait NonceCommons {
    fn public_hex_or_ss58(&self) -> &String;

    async fn offchain_nonce_key(&self, rpc: &subxt::rpc::Rpc<DefaultConfig>) -> Result<Vec<u8>> {
        let acc = self.public_hex_or_ss58();
        rpc.task_get_offchain_nonce_key(acc)
            .await
            .map_err(|e| e.into())
    }

    async fn offchain_nonce(
        &self,
        rpc: &subxt::rpc::Rpc<DefaultConfig>,
    ) -> Result<Option<Result<u32, parity_scale_codec::Error>>> {
        let nonce_key = self.offchain_nonce_key(rpc).await?;
        let nonce = rpc
            .offchain_storage(StorageKind::PERSISTENT, &StorageKey(nonce_key.clone()))
            .await?
            .map(|StorageData(nonce)| u32::decode(&mut nonce.as_slice()));
        Ok(nonce)
    }
}

impl NonceCommons for ResetArgs {
    fn public_hex_or_ss58(&self) -> &String {
        &self.public_hex
    }
}
impl NonceCommons for ShowArgs {
    fn public_hex_or_ss58(&self) -> &String {
        &self.public_hex
    }
}

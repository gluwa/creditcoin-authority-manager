#![allow(clippy::too_many_arguments)]
use async_trait::async_trait;
use color_eyre::Result;
use extend::ext;
use subxt::{
    extrinsic::{BaseExtrinsicParams, PlainTip},
    DefaultConfig,
};
#[subxt::subxt(runtime_metadata_path = "./creditcoin-metadata.scale")]
pub mod creditcoin {}
use misc::{StorageData, StorageKey};
use sp_core::offchain::StorageKind;
use subxt::{
    rpc::{rpc_params, ClientT, Rpc},
    BasicError, Config,
};
pub mod commands;
pub mod misc;

pub type CreditcoinExtrinsicParams = BaseExtrinsicParams<DefaultConfig, PlainTip>;
pub type RuntimeApi = creditcoin::RuntimeApi<DefaultConfig, CreditcoinExtrinsicParams>;
pub type RunResult = Result<()>;

#[ext]
#[async_trait]
pub impl<T: Config> Rpc<T> {
    async fn offchain_storage(
        &self,
        storage_kind: StorageKind,
        key: &StorageKey,
    ) -> Result<Option<StorageData>, BasicError> {
        let params = rpc_params![storage_kind.to_string(), key];
        let data = self
            .client
            .request("offchain_localStorageGet", params)
            .await?;
        Ok(data)
    }

    async fn set_offchain_storage(
        &self,
        storage_kind: StorageKind,
        key: &StorageKey,
        value: &StorageData,
    ) -> Result<(), BasicError> {
        let params = rpc_params![storage_kind.to_string(), key, value];
        self.client
            .request("offchain_localStorageSet", params)
            .await?;
        Ok(())
    }

    async fn add_log_filter(&self, filter: String) -> Result<(), BasicError> {
        let params = rpc_params![filter];
        self.client.request("system_addLogFilter", params).await?;
        Ok(())
    }

    async fn reset_log_filter(&self) -> Result<(), BasicError> {
        self.client.request("system_resetLogFilter", None).await?;
        Ok(())
    }
}

#[async_trait]
pub trait Run {
    async fn run(self, api: &RuntimeApi) -> RunResult;
}

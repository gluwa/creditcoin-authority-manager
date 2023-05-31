#![allow(clippy::too_many_arguments)]
use async_trait::async_trait;
use color_eyre::Result;
use extend::ext;

#[subxt::subxt(runtime_metadata_path = "./creditcoin-metadata.scale")]
pub mod creditcoin {}
use misc::{StorageData, StorageKey};
use sp_core::offchain::StorageKind;
use subxt::{
    config::{extrinsic_params::BaseExtrinsicParams, polkadot::PlainTip, WithExtrinsicParams},
    rpc::{rpc_params, Rpc},
    Config, Error, OnlineClient, SubstrateConfig,
};
pub mod commands;
pub mod misc;

pub type CreditcoinExtrinsicParams = BaseExtrinsicParams<SubstrateConfig, PlainTip>;
pub type CreditcoinConfig = WithExtrinsicParams<SubstrateConfig, CreditcoinExtrinsicParams>;
pub type RunResult = Result<()>;
pub type ApiClient = OnlineClient<CreditcoinConfig>;

#[ext]
#[async_trait]
pub impl<T: Config> Rpc<T> {
    async fn offchain_storage(
        &self,
        storage_kind: StorageKind,
        key: &StorageKey,
    ) -> Result<Option<StorageData>, Error> {
        let data = self
            .request("offchain_localStorageGet", rpc_params![storage_kind, key])
            .await?;
        Ok(data)
    }

    async fn set_offchain_storage(
        &self,
        storage_kind: StorageKind,
        key: &StorageKey,
        value: &StorageData,
    ) -> Result<(), Error> {
        self.request(
            "offchain_localStorageSet",
            rpc_params![storage_kind, key, value],
        )
        .await?;
        Ok(())
    }

    async fn add_log_filter(&self, filter: String) -> Result<(), Error> {
        self.request("system_addLogFilter", rpc_params![filter])
            .await?;
        Ok(())
    }

    async fn reset_log_filter(&self) -> Result<(), Error> {
        self.request("system_resetLogFilter", rpc_params![]).await?;
        Ok(())
    }

    async fn task_get_offchain_nonce_key(&self, hex_key: &str) -> Result<Vec<u8>, Error> {
        self.request("task_getOffchainNonceKey", rpc_params![hex_key])
            .await
    }
}

#[async_trait]
pub trait Run {
    async fn run(self, api: &ApiClient) -> RunResult;
}

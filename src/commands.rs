use crate::misc::Blockchain;
use crate::Run;
use crate::RunResult;
use crate::RuntimeApi;
use async_trait::async_trait;
use clap::{Args, Subcommand};
use color_eyre::Result;
use futures::FutureExt;
use parity_scale_codec::Decode;
use sp_core::{
    crypto::{AccountId32, Ss58Codec},
    Bytes,
};
use strum::IntoEnumIterator;
use tabled::{TableIteratorExt, Tabled};
pub mod get_set;
use get_set::{get, GetArgs, SetArgs};
pub mod log_filters;
use log_filters::LogFilterCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Get the currently configured RPC URL for a given blockchain.
    Get(GetArgs),
    /// Set the RPC URL to use for a given blockchain.
    Set(SetArgs),
    /// Insert an (authority) key into the keystore.
    Insert(InsertArgs),
    #[clap(name = "log-filter", subcommand)]
    /// Operations on log filters.
    LogFilter(LogFilterCommand),
    /// Retrieves the account ID of the authority, if one exists
    Account,
    /// Lists the configured RPC URLs for all blockchains.
    List,
}

#[derive(Debug, Clone, Args)]
pub struct InsertArgs {
    /// The mnemonic phrase of the key to insert.
    suri: String,
    /// The hex encoded public key of the key to insert.
    public_hex: String,
}

async fn authority_account_command(api: &RuntimeApi) -> RunResult {
    println!(
        "{}",
        match authority_account(api).await? {
            Some(acct) => acct.to_ss58check(),
            None => "No authority account found".into(),
        }
    );
    Ok(())
}

#[async_trait]
impl Run for Commands {
    async fn run(self, api: &RuntimeApi) -> RunResult {
        use Commands::*;
        match self {
            Get(get) => get.run(api).await,
            Set(set) => set.run(api).await,
            Insert(insert) => insert.run(api).await,
            List => list(api).await,
            Account => authority_account_command(api).await,
            LogFilter(log_filter) => log_filter.run(api).await,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Tabled)]
struct RpcConfig {
    #[tabled(rename = "Blockchain")]
    blockchain: Blockchain,
    #[tabled(rename = "Configured URL")]
    url: String,
}

async fn list(api: &RuntimeApi) -> RunResult {
    let url_requests = Blockchain::iter().map(|blockchain| {
        get(api, blockchain).map(move |url| url.map(|url| RpcConfig { blockchain, url }))
    });
    let configs = futures::future::try_join_all(url_requests).await?.table();
    println!("{configs}");
    Ok(())
}

async fn authority_account(api: &RuntimeApi) -> Result<Option<AccountId32>> {
    let mut authorities = api.storage().creditcoin().authorities_iter(None).await?;

    while let Some((key, ())) = authorities.next().await? {
        let len = key.0.len();
        let account_id = AccountId32::decode(&mut &key.0[len - 32..])?;

        let bytes: &[u8; 32] = account_id.as_ref();
        let has_key = api
            .client
            .rpc()
            .has_key(Bytes(bytes.to_vec()), "ctcs".into())
            .await?;
        if has_key {
            return Ok(Some(account_id));
        }
    }

    Ok(None)
}

#[async_trait]
impl Run for InsertArgs {
    async fn run(self, api: &RuntimeApi) -> RunResult {
        let Self {
            suri, public_hex, ..
        } = self;
        let client = &api.client;

        let public_bytes = Bytes(hex::decode(&public_hex.trim_start_matches("0x"))?);

        client
            .rpc()
            .insert_key("ctcs".into(), suri, public_bytes.clone())
            .await?;

        assert!(client.rpc().has_key(public_bytes, "ctcs".into()).await?);

        println!("Inserted {}", public_hex);
        Ok(())
    }
}

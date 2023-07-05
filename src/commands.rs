use crate::{misc::Blockchain, ApiClient, Run, RunResult};
use async_trait::async_trait;
use clap::{Args, Subcommand};
use color_eyre::Result;
use futures::FutureExt;
use parity_scale_codec::Decode;
use sp_core::crypto::{AccountId32, Ss58Codec};
use strum::IntoEnumIterator;
use tabled::{Table, Tabled};
pub mod get_set;
use get_set::{get, GetArgs, SetArgs};
pub mod log_filters;
use log_filters::LogFilterCommand;
pub mod nonce;
use nonce::NonceCommand;
use subxt::rpc::types::Bytes;

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
    /// Offchain Nonce directives.
    #[clap(subcommand)]
    Nonce(NonceCommand),
}

#[derive(Debug, Clone, Args)]
pub struct InsertArgs {
    /// The mnemonic phrase of the key to insert.
    suri: String,
    /// The hex encoded public key of the key to insert.
    public_hex: String,
}

async fn authority_account_command(api: &ApiClient) -> RunResult {
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
    async fn run(self, api: &ApiClient) -> RunResult {
        use Commands::*;
        match self {
            Get(get) => get.run(api).await,
            Set(set) => set.run(api).await,
            Insert(insert) => insert.run(api).await,
            List => list(api).await,
            Account => authority_account_command(api).await,
            LogFilter(log_filter) => log_filter.run(api).await,
            Nonce(subcommand) => subcommand.run(api).await,
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

async fn list(api: &ApiClient) -> RunResult {
    let url_requests = Blockchain::iter().map(|blockchain| {
        get(api, blockchain).map(move |item| {
            item.map(|o| RpcConfig {
                blockchain,
                url: o.unwrap_or_else(|| "None".into()),
            })
        })
    });

    let configs = Table::new(futures::future::try_join_all(url_requests).await?);
    println!("{configs}");
    Ok(())
}

const AUTH_KEY_ID: &str = "gots";

async fn authority_account(api: &ApiClient) -> Result<Option<AccountId32>> {
    let mut authorities = api
        .storage()
        .at_latest()
        .await?
        .iter(
            crate::creditcoin::storage()
                .task_scheduler()
                .authorities_root(),
            26,
        )
        .await?;

    while let Some((key, ())) = authorities.next().await? {
        let len = key.0.len();
        let account_id = AccountId32::decode(&mut &key.0[len - 32..])?;

        let bytes: &[u8; 32] = account_id.as_ref();
        let has_key = api
            .rpc()
            .has_key(Bytes(bytes.to_vec()), AUTH_KEY_ID.into())
            .await?;
        if has_key {
            return Ok(Some(account_id));
        }
    }

    Ok(None)
}

#[async_trait]
impl Run for InsertArgs {
    async fn run(self, api: &ApiClient) -> RunResult {
        let Self {
            suri, public_hex, ..
        } = self;

        let public_bytes = Bytes(hex::decode(public_hex.trim_start_matches("0x"))?);

        api.rpc()
            .insert_key(AUTH_KEY_ID.into(), suri, public_bytes.clone())
            .await?;

        assert!(api.rpc().has_key(public_bytes, AUTH_KEY_ID.into()).await?);

        println!("Inserted {}", public_hex);
        Ok(())
    }
}

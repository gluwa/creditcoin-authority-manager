use core::fmt;
use std::str::FromStr;

use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use color_eyre::Result;

use creditcoin_authority_manager::RuntimeApi;
use extend::ext;
use futures::FutureExt;
use parity_scale_codec::{Decode, Encode};
use sp_core::{
    crypto::{AccountId32, Ss58Codec},
    storage::{StorageData, StorageKey},
    Bytes,
};
use strum::{EnumIter, IntoEnumIterator};
use subxt::{
    rpc::{rpc_params, ClientT, Rpc},
    BasicError, ClientBuilder, Config, DefaultConfig,
};
use tabled::{TableIteratorExt, Tabled};

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[clap(long, default_value = "ws://127.0.0.1:9944")]
    url: String,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    Get(GetArgs),
    Set(SetArgs),
    Insert(InsertArgs),
    Account,
    List,
    Setup,
}

#[derive(Debug, Clone, Args)]
struct GetArgs {
    blockchain: Blockchain,
}

#[derive(Debug, Clone, Args)]
struct SetArgs {
    blockchain: Blockchain,
    rpc_url: String,
}

#[derive(Debug, Clone, Args)]
struct InsertArgs {
    suri: String,
    public_hex: String,
}

#[async_trait]
trait Run {
    async fn run(self, client: &RuntimeApi) -> Result<()>;
}

#[async_trait]
impl Run for Commands {
    async fn run(self, client: &RuntimeApi) -> Result<()> {
        match self {
            Commands::Get(get) => get.run(client).await,
            Commands::Set(set) => set.run(client).await,
            Commands::Insert(insert) => insert.run(client).await,
            Commands::List => list(client).await,
            Commands::Account => {
                println!(
                    "{}",
                    match authority_account(client).await? {
                        Some(acct) => acct.to_ss58check(),
                        None => "No authority account found".into(),
                    }
                );
                Ok(())
            }
            Commands::Setup => todo!(),
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

async fn list(client: &RuntimeApi) -> Result<()> {
    let foo = Blockchain::iter().map(|blockchain| {
        get(client, blockchain).map(move |url| url.map(|url| RpcConfig { blockchain, url }))
    });
    let configs = futures::future::try_join_all(foo).await?.table();
    println!("{configs}");
    Ok(())
}

async fn get(api: &RuntimeApi, blockchain: Blockchain) -> Result<String> {
    let key = blockchain.to_key();

    let value = api
        .client
        .rpc()
        .offchain_storage(StorageKind::Persistent, &key)
        .await?;

    let value = match value {
        Some(value) => String::decode(&mut value.0.as_slice())?,
        None => "None".to_string(),
    };

    Ok(value)
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
impl Run for GetArgs {
    async fn run(self, client: &RuntimeApi) -> Result<()> {
        let Self { blockchain } = self;

        let value = get(client, blockchain).await?;

        println!("{value}");
        Ok(())
    }
}

#[async_trait]
impl Run for SetArgs {
    async fn run(self, api: &RuntimeApi) -> Result<()> {
        let Self {
            blockchain,
            rpc_url,
        } = self;
        let key = blockchain.to_key();
        let client = &api.client;

        client
            .rpc()
            .set_offchain_storage(StorageKind::Persistent, &key, &url_to_value(&rpc_url))
            .await?;

        assert_eq!(
            client
                .rpc()
                .offchain_storage(StorageKind::Persistent, &key)
                .await?,
            Some(url_to_value(&rpc_url))
        );

        println!("{:?} -> {}", blockchain, rpc_url);
        Ok(())
    }
}

#[async_trait]
impl Run for InsertArgs {
    async fn run(self, api: &RuntimeApi) -> Result<()> {
        let Self {
            suri, public_hex, ..
        } = self;
        let client = &api.client;

        let public_bytes = Bytes(hex::decode(&public_hex)?);

        client
            .rpc()
            .insert_key("ctcs".into(), suri, public_bytes.clone())
            .await?;

        assert!(client.rpc().has_key(public_bytes, "ctcs".into()).await?);

        println!("Inserted {}", public_hex);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
enum Blockchain {
    Ethereum,
    Rinkeby,
    Luniverse,
    Bitcoin,
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Blockchain::Ethereum => write!(f, "Ethereum"),
            Blockchain::Rinkeby => write!(f, "Rinkeby"),
            Blockchain::Luniverse => write!(f, "Luniverse"),
            Blockchain::Bitcoin => write!(f, "Bitcoin"),
        }
    }
}

impl FromStr for Blockchain {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" => Ok(Blockchain::Ethereum),
            "rinkeby" => Ok(Blockchain::Rinkeby),
            "luniverse" => Ok(Blockchain::Luniverse),
            "bitcoin" => Ok(Blockchain::Bitcoin),
            _ => Err(color_eyre::eyre::eyre!("unknown blockchain: {}", s)),
        }
    }
}

impl Blockchain {
    fn to_key(self) -> StorageKey {
        StorageKey(
            format!("{}-rpc-uri", self.to_string().to_lowercase())
                .as_bytes()
                .to_vec(),
        )
    }
}

fn url_to_value(url: &str) -> StorageData {
    StorageData(url.encode())
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum StorageKind {
    #[allow(dead_code)]
    Local,
    Persistent,
}

impl fmt::Display for StorageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StorageKind::Local => "LOCAL",
                StorageKind::Persistent => "PERSISTENT",
            }
        )
    }
}

#[ext]
#[async_trait]
impl<T: Config> Rpc<T> {
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
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let client = ClientBuilder::new()
        .set_url(&cli.url)
        .build::<DefaultConfig>()
        .await?
        .to_runtime_api();
    cli.command.run(&client).await?;

    Ok(())
}

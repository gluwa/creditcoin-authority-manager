use color_eyre::Result;
use core::fmt;
use once_cell::sync::Lazy;
use parity_scale_codec::Encode;
use regex::{Regex, RegexBuilder};
pub use sp_core::storage::{StorageData, StorageKey};
use std::str::FromStr;
use strum::EnumIter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StorageKind {
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
impl Blockchain {
    pub fn to_key(self) -> StorageKey {
        StorageKey(
            format!("{}-rpc-uri", self.to_string().to_lowercase())
                .as_bytes()
                .to_vec(),
        )
    }
}

pub fn url_to_value(url: &str) -> StorageData {
    StorageData(url.encode())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum Blockchain {
    Ethereum,
    Rinkeby,
    Luniverse,
    Bitcoin,
    Evm(u64),
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Blockchain::Ethereum => write!(f, "Ethereum"),
            Blockchain::Rinkeby => write!(f, "Rinkeby"),
            Blockchain::Luniverse => write!(f, "Luniverse"),
            Blockchain::Bitcoin => write!(f, "Bitcoin"),
            Blockchain::Evm(chain_id) => write!(f, "EVM-{chain_id}"),
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
            other => {
                static EVM_REGEX: Lazy<Regex> = Lazy::new(|| {
                    RegexBuilder::new(r"evm\-(\d+)")
                        .case_insensitive(true)
                        .build()
                        .unwrap()
                });

                if let Some(captures) = EVM_REGEX.captures(other) {
                    Ok(Blockchain::Evm(captures.get(1).unwrap().as_str().parse()?))
                } else {
                    Err(color_eyre::eyre::eyre!("unknown blockchain: {other}"))
                }
            }
        }
    }
}

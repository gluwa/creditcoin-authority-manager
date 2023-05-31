use color_eyre::Result;
use core::fmt;
use parity_scale_codec::Encode;
pub use sp_core::storage::{StorageData, StorageKey};
use std::str::FromStr;
use strum::EnumIter;

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
            other => Err(color_eyre::eyre::eyre!("unknown blockchain: {other}")),
        }
    }
}

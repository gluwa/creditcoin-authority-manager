#![allow(clippy::too_many_arguments)]

use subxt::{
    extrinsic::{BaseExtrinsicParams, PlainTip},
    DefaultConfig,
};

#[subxt::subxt(runtime_metadata_path = "./creditcoin-metadata.scale")]
pub mod creditcoin {}

pub type CreditcoinExtrinsicParams = BaseExtrinsicParams<DefaultConfig, PlainTip>;

pub type RuntimeApi = creditcoin::RuntimeApi<DefaultConfig, CreditcoinExtrinsicParams>;

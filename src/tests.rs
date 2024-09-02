use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    providers::{ProviderBuilder, ReqwestProvider},
    transports::http::reqwest::Url,
};
use dotenv::dotenv;
use once_cell::sync::Lazy;

pub(crate) const BLOCK_NUMBER: BlockId = BlockId::Number(BlockNumberOrTag::Number(17000000));
pub(crate) static RPC_URL: Lazy<Url> = Lazy::new(|| {
    dotenv().ok();
    std::env::var("MAINNET_RPC_URL").unwrap().parse().unwrap()
});
pub(crate) static PROVIDER: Lazy<ReqwestProvider> =
    Lazy::new(|| ProviderBuilder::new().on_http(RPC_URL.clone()));

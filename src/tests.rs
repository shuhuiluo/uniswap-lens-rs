use alloy::{
    eips::BlockId,
    providers::{ProviderBuilder, ReqwestProvider},
    transports::http::reqwest::Url,
};
use dotenv::dotenv;
use once_cell::sync::Lazy;

pub(crate) static BLOCK_NUMBER: Lazy<BlockId> = Lazy::new(|| BlockId::from(17000000));
pub(crate) static RPC_URL: Lazy<Url> = Lazy::new(|| {
    dotenv().ok();
    std::env::var("MAINNET_RPC_URL").unwrap().parse().unwrap()
});
pub(crate) static PROVIDER: Lazy<ReqwestProvider> =
    Lazy::new(|| ProviderBuilder::new().on_http(RPC_URL.clone()));

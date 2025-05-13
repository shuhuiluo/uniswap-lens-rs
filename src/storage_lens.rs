//! ## Storage Lens
//!
//! The storage lens module provides a function to batch `eth_getStorageAt` RPC calls in a single
//! `eth_call` by overriding the target contract's deployed bytecode with `EphemeralStorageLens`.

use crate::{bindings::ephemeralstoragelens::EphemeralStorageLens, error::Error};
use alloc::vec::Vec;
use alloy::{
    eips::BlockId,
    network::Network,
    primitives::{Address, B256},
    providers::Provider,
    rpc::types::state::{AccountOverride, StateOverride},
};

/// Batch `eth_getStorageAt` RPC calls in a single `eth_call` by overriding the target contract's
/// deployed bytecode with `EphemeralStorageLens`
///
/// ## Arguments
///
/// * `address`: The contract address to fetch storage from
/// * `slots`: The storage slots to query
/// * `provider`: The alloy provider
/// * `block_id`: Optional block id to query
///
/// ## Returns
///
/// The storage values at the given slots
#[inline]
pub async fn get_storage_at<N, P>(
    address: Address,
    slots: Vec<B256>,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<B256>, Error>
where
    N: Network,
    P: Provider<N>,
{
    // override the deployed bytecode at `address`
    let state = StateOverride::from_iter([(
        address,
        AccountOverride {
            code: Some(EphemeralStorageLens::DEPLOYED_BYTECODE.clone()),
            ..Default::default()
        },
    )]);
    let lens = EphemeralStorageLens::new(address, provider);
    let call_builder = lens.extsload(slots).state(state);
    let call_builder = match block_id {
        Some(block_id) => call_builder.block(block_id),
        None => call_builder,
    };
    Ok(call_builder.call().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use alloy::primitives::{address, U256};
    use futures::future::join_all;

    const POOL_ADDRESS: Address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");

    #[tokio::test]
    async fn test_get_storage_at() {
        let provider = PROVIDER.clone();
        let slots = get_storage_at(
            POOL_ADDRESS,
            (0..10)
                .map(|i| B256::from(U256::from_limbs([i, 0, 0, 0])))
                .collect(),
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        let slots_ref = slots.as_slice();
        let provider = provider.root();
        let futures = (0..10).map(|i| async move {
            let slot = provider
                .get_storage_at(POOL_ADDRESS, U256::from_limbs([i, 0, 0, 0]))
                .block_id(BLOCK_NUMBER)
                .await
                .unwrap();
            assert_eq!(slot, U256::from_be_bytes(slots_ref[i as usize].0));
        });
        join_all(futures).await;
    }
}

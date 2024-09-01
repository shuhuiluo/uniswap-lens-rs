//! ## Storage Lens
//!
//! The storage lens module provides a function to batch `eth_getStorageAt` RPC calls in a single
//! `eth_call` by overriding the target contract's deployed bytecode with `EphemeralStorageLens`.

use crate::bindings::ephemeralstoragelens::{
    EphemeralStorageLens, EphemeralStorageLens::EphemeralStorageLensInstance,
};
use alloc::vec::Vec;
use alloy::{
    eips::BlockId,
    primitives::{Address, FixedBytes},
    providers::Provider,
    rpc::types::state::{AccountOverride, StateOverride},
    transports::Transport,
};
use anyhow::Result;

/// Batch `eth_getStorageAt` RPC calls in a single `eth_call` by overriding the target contract's
/// deployed bytecode with `EphemeralStorageLens`
///
/// # Arguments
///
/// * `address`: The contract address to fetch storage from
/// * `slots`: The storage slots to query
/// * `provider`: The alloy provider
/// * `block_id`: Optional block id to query
///
/// returns: Result<Vec<[u8; 32], Global>, ContractError<M>>
pub async fn get_storage_at<T, P>(
    address: Address,
    slots: Vec<FixedBytes<32>>,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<FixedBytes<32>>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    // override the deployed bytecode at `address`
    let mut state = StateOverride::default();
    state.insert(
        address,
        AccountOverride {
            code: Some(EphemeralStorageLens::DEPLOYED_BYTECODE.clone()),
            ..Default::default()
        },
    );
    let lens = EphemeralStorageLensInstance::new(address, provider);
    let call_builder = lens.extsload(slots).state(state);
    let call_builder = match block_id {
        Some(block_id) => call_builder.block(block_id),
        None => call_builder,
    };
    Ok(call_builder.call().await?._0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;
    use alloy::primitives::{address, U256};
    use anyhow::Result;
    use futures::future::join_all;

    const POOL_ADDRESS: Address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");

    #[tokio::test]
    async fn test_get_storage_at() -> Result<()> {
        let provider = PROVIDER.clone();
        let slots = get_storage_at(
            POOL_ADDRESS,
            (0..10)
                .map(|i| FixedBytes::from(U256::from_limbs([i, 0, 0, 0])))
                .collect(),
            provider.clone(),
            Some(*BLOCK_NUMBER),
        )
        .await?;
        let slots_ref = slots.as_slice();
        let provider = provider.root();
        let futures = (0..10).map(|i| async move {
            let slot = provider
                .get_storage_at(POOL_ADDRESS, U256::from_limbs([i, 0, 0, 0]))
                .block_id(*BLOCK_NUMBER)
                .await
                .unwrap();
            assert_eq!(slot, U256::from_be_bytes(slots_ref[i as usize].0));
        });
        join_all(futures).await;
        Ok(())
    }
}

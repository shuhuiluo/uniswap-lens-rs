//! ## Pool Lens
//!
//! The pool lens module provides functions to fetch pool details using ephemeral contracts.

use crate::{
    bindings::{
        ephemeralgetpopulatedticksinrange::{
            EphemeralGetPopulatedTicksInRange,
            EphemeralGetPopulatedTicksInRange::{
                getPopulatedTicksInRangeCall, getPopulatedTicksInRangeReturn,
            },
            PoolUtils::PopulatedTick,
        },
        ephemeralpoolpositions::{EphemeralPoolPositions, PoolUtils::PositionKey},
        ephemeralpoolslots::{
            EphemeralPoolSlots, EphemeralPoolSlots::getSlotsCall, PoolUtils::Slot,
        },
        ephemeralpooltickbitmap::EphemeralPoolTickBitmap,
        ephemeralpoolticks::EphemeralPoolTicks,
    },
    call_ephemeral_contract,
    error::Error,
};
use alloc::vec::Vec;
use alloy::{
    contract::Error as ContractError,
    eips::BlockId,
    network::Network,
    primitives::{aliases::I24, Address},
    providers::Provider,
    sol_types::SolCall,
    transports::TransportError,
};

/// Get the populated ticks in a tick range.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `tick_lower`: The lower tick boundary
/// * `tick_upper`: The upper tick boundary
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of populated ticks within the range
#[inline]
pub async fn get_populated_ticks_in_range<N, P>(
    pool: Address,
    tick_lower: I24,
    tick_upper: I24,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<(Vec<PopulatedTick>, I24), Error>
where
    N: Network,
    P: Provider<N>,
{
    let deploy_builder =
        EphemeralGetPopulatedTicksInRange::deploy_builder(provider, pool, tick_lower, tick_upper);
    match call_ephemeral_contract!(deploy_builder, getPopulatedTicksInRangeCall, block_id) {
        Ok(getPopulatedTicksInRangeReturn {
            populatedTicks,
            tickSpacing,
        }) => Ok((
            populatedTicks
                .into_iter()
                .filter(|PopulatedTick { tick, .. }| *tick >= tick_lower && *tick <= tick_upper)
                .collect(),
            tickSpacing,
        )),
        Err(err) => Err(err),
    }
}

/// Call an ephemeral contract and return the decoded storage slots
macro_rules! get_pool_storage {
    ($deploy_builder:expr, $block_id:expr) => {
        call_ephemeral_contract!($deploy_builder, getSlotsCall, $block_id)
    };
}

/// Get the static storage slots of a pool.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_static_slots<N, P>(
    pool: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    get_pool_storage!(EphemeralPoolSlots::deploy_builder(provider, pool), block_id)
}

/// Get the storage slots in the `ticks` mapping between `tick_lower` and `tick_upper`.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `tick_lower`: The lower tick boundary
/// * `tick_upper`: The upper tick boundary
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_ticks_slots<N, P>(
    pool: Address,
    tick_lower: I24,
    tick_upper: I24,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    get_pool_storage!(
        EphemeralPoolTicks::deploy_builder(provider, pool, tick_lower, tick_upper),
        block_id
    )
}

/// Get the storage slots in the `tickBitmap` mapping.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_tick_bitmap_slots<N, P>(
    pool: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    get_pool_storage!(
        EphemeralPoolTickBitmap::deploy_builder(provider, pool),
        block_id
    )
}

/// Get the storage slots in the `positions` mapping.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `positions`: A vector of position keys
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_positions_slots<N, P>(
    pool: Address,
    positions: Vec<PositionKey>,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    get_pool_storage!(
        EphemeralPoolPositions::deploy_builder(provider, pool, positions),
        block_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bindings::iuniswapv3pool::{IUniswapV3Pool, IUniswapV3Pool::Mint},
        tests::*,
    };
    use alloy::{
        primitives::address, providers::MulticallBuilder, rpc::types::Filter, sol_types::SolEvent,
    };
    use futures::future::join_all;

    const POOL_ADDRESS: Address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");

    #[tokio::test]
    async fn test_get_populated_ticks_in_range() {
        let provider = PROVIDER.clone();
        let pool = IUniswapV3Pool::new(POOL_ADDRESS, provider.clone());
        let tick_current = pool.slot0().block(BLOCK_NUMBER).call().await.unwrap().tick;
        let tick_spacing = pool.tickSpacing().block(BLOCK_NUMBER).call().await.unwrap();
        let (ticks, _) = get_populated_ticks_in_range(
            POOL_ADDRESS,
            tick_current,
            tick_current + (tick_spacing << 8),
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        assert!(!ticks.is_empty());

        let mut multicall = MulticallBuilder::new_dynamic(provider.clone());
        for PopulatedTick { tick, .. } in ticks.iter() {
            multicall = multicall.add_dynamic(pool.ticks(*tick));
        }
        let alt_ticks = multicall.block(BLOCK_NUMBER).aggregate().await.unwrap();

        for (
            i,
            PopulatedTick {
                liquidityGross,
                liquidityNet,
                ..
            },
        ) in ticks.into_iter().enumerate()
        {
            let tick_info = &alt_ticks[i];
            assert_eq!(liquidityGross, tick_info.liquidityGross);
            assert_eq!(liquidityNet, tick_info.liquidityNet);
        }
    }

    async fn verify_slots<N, P>(slots: Vec<Slot>, provider: P)
    where
        N: Network,
        P: Provider<N>,
    {
        assert!(!slots.is_empty());
        let provider = provider.root();
        let futures = slots[0..4].iter().map(|slot| async move {
            let data = provider
                .get_storage_at(POOL_ADDRESS, slot.slot)
                .block_id(BLOCK_NUMBER)
                .await
                .unwrap();
            assert!(slot.data.eq(&data));
        });
        join_all(futures).await;
    }

    #[tokio::test]
    async fn test_get_static_slots() {
        let provider = PROVIDER.clone();
        let slots = get_static_slots(POOL_ADDRESS, provider.clone(), Some(BLOCK_NUMBER))
            .await
            .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_ticks_slots() {
        let provider = PROVIDER.clone();
        let pool = IUniswapV3Pool::new(POOL_ADDRESS, provider.clone());
        let tick_current = pool.slot0().block(BLOCK_NUMBER).call().await.unwrap().tick;
        let slots = get_ticks_slots(
            POOL_ADDRESS,
            tick_current,
            tick_current,
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_tick_bitmap_slots() {
        let provider = PROVIDER.clone();
        let slots = get_tick_bitmap_slots(POOL_ADDRESS, provider.clone(), Some(BLOCK_NUMBER))
            .await
            .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_positions_slots() {
        let provider = PROVIDER.clone();
        // create a filter to get the mint events
        let filter = Filter::new()
            .from_block(BLOCK_NUMBER.as_u64().unwrap() - 499)
            .to_block(BLOCK_NUMBER.as_u64().unwrap())
            .event_signature(<Mint as SolEvent>::SIGNATURE_HASH);
        let logs = provider.get_logs(&filter).await.unwrap();
        // decode the logs into position keys
        let positions: Vec<_> = logs
            .iter()
            .map(|log| <Mint as SolEvent>::decode_log_data(log.data()).unwrap())
            .map(
                |Mint {
                     owner,
                     tickLower,
                     tickUpper,
                     ..
                 }| PositionKey {
                    owner,
                    tickLower,
                    tickUpper,
                },
            )
            .collect();
        assert!(!positions.is_empty());
        let slots = get_positions_slots(
            POOL_ADDRESS,
            positions,
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        verify_slots(slots, provider).await;
    }
}

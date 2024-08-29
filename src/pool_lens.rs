use crate::{
    bindings::{
        ephemeralgetpopulatedticksinrange::EphemeralGetPopulatedTicksInRange::{
            getPopulatedTicksInRangeCall, getPopulatedTicksInRangeReturn,
            EphemeralGetPopulatedTicksInRangeInstance, PopulatedTick,
        },
        ephemeralpoolpositions::EphemeralPoolPositions::{
            EphemeralPoolPositionsInstance, PositionKey,
        },
        ephemeralpoolslots::EphemeralPoolSlots::{
            getSlotsCall, getSlotsReturn, EphemeralPoolSlotsInstance, Slot,
        },
        ephemeralpooltickbitmap::EphemeralPoolTickBitmap::EphemeralPoolTickBitmapInstance,
        ephemeralpoolticks::EphemeralPoolTicks::EphemeralPoolTicksInstance,
    },
    call_ephemeral_contract,
};
use alloy::{
    contract::Error,
    eips::BlockId,
    primitives::{aliases::I24, Address, Bytes},
    providers::Provider,
    sol_types::SolCall,
    transports::{Transport, TransportError},
};
use anyhow::Result;

pub async fn get_populated_ticks_in_range<T, P>(
    pool: Address,
    tick_lower: I24,
    tick_upper: I24,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<PopulatedTick>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    let deploy_builder = EphemeralGetPopulatedTicksInRangeInstance::deploy_builder(
        provider, pool, tick_lower, tick_upper,
    );
    match call_ephemeral_contract!(deploy_builder, getPopulatedTicksInRangeCall, block_id) {
        Ok(getPopulatedTicksInRangeReturn { populatedTicks }) => Ok(populatedTicks),
        Err(err) => Err(err.into()),
    }
}

/// Call an ephemeral contract and return the decoded storage slots
macro_rules! get_pool_storage {
    ($deploy_builder:expr, $block_id:expr) => {
        match call_ephemeral_contract!($deploy_builder, getSlotsCall, $block_id) {
            Ok(getSlotsReturn { slots }) => Ok(slots),
            Err(err) => Err(err.into()),
        }
    };
}

pub async fn get_static_slots<T, P>(
    pool: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    get_pool_storage!(
        EphemeralPoolSlotsInstance::deploy_builder(provider, pool),
        block_id
    )
}

pub async fn get_ticks_slots<T, P>(
    pool: Address,
    tick_lower: I24,
    tick_upper: I24,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    get_pool_storage!(
        EphemeralPoolTicksInstance::deploy_builder(provider, pool, tick_lower, tick_upper),
        block_id
    )
}
pub async fn get_tick_bitmap_slots<T, P>(
    pool: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    get_pool_storage!(
        EphemeralPoolTickBitmapInstance::deploy_builder(provider, pool),
        block_id
    )
}
pub async fn get_positions_slots<T, P>(
    pool: Address,
    positions: Vec<PositionKey>,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    get_pool_storage!(
        EphemeralPoolPositionsInstance::deploy_builder(provider, pool, positions),
        block_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bindings::iuniswapv3pool::IUniswapV3Pool::{IUniswapV3PoolInstance, Mint},
        tests::*,
    };
    use alloy::{primitives::address, rpc::types::Filter, sol_types::SolEvent};
    use anyhow::Result;
    use futures::future::join_all;

    const POOL_ADDRESS: Address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    #[tokio::test]
    async fn test_get_populated_ticks_in_range() -> Result<()> {
        let provider = PROVIDER.clone();
        let pool = IUniswapV3PoolInstance::new(POOL_ADDRESS, provider.clone());
        let tick_current = pool.slot0().block(*BLOCK_NUMBER).call().await?.tick;
        let ticks = get_populated_ticks_in_range(
            POOL_ADDRESS,
            tick_current,
            tick_current,
            provider,
            Some(*BLOCK_NUMBER),
        )
        .await?;
        assert!(!ticks.is_empty());
        // let mut multicall = Multicall::new(client.clone(), None).await?;
        // multicall.add_calls(
        //     false,
        //     ticks
        //         .iter()
        //         .map(|&PopulatedTick { tick, .. }| pool.ticks(tick)),
        // );
        // #[allow(clippy::type_complexity)]
        // let alt_ticks: Vec<(u128, i128, U256, U256, i64, U256, u32, bool)> = multicall
        //     .block(match *BLOCK_NUMBER {
        //         BlockId::Number(n) => n,
        //         _ => panic!("block id must be a number"),
        //     })
        //     .call_array()
        //     .await?;
        // for (
        //     i,
        //     &PopulatedTick {
        //         liquidity_gross,
        //         liquidity_net,
        //         ..
        //     },
        // ) in ticks.iter().enumerate()
        // {
        //     let (_liquidity_gross, _liquidity_net, _, _, _, _, _, _) = alt_ticks[i];
        //     assert_eq!(liquidity_gross, _liquidity_gross);
        //     assert_eq!(liquidity_net, _liquidity_net);
        // }
        Ok(())
    }

    async fn verify_slots<T, P>(slots: Vec<Slot>, provider: P)
    where
        T: Transport + Clone,
        P: Provider<T>,
    {
        assert!(!slots.is_empty());
        let provider = provider.root();
        let futures = slots[0..4].iter().map(|slot| async move {
            let data = provider
                .get_storage_at(POOL_ADDRESS, slot.slot)
                .block_id(*BLOCK_NUMBER)
                .await
                .unwrap();
            assert!(slot.data.eq(&data));
        });
        join_all(futures).await;
    }

    #[tokio::test]
    async fn test_get_static_slots() {
        let provider = PROVIDER.clone();
        let slots = get_static_slots(POOL_ADDRESS, provider.clone(), Some(*BLOCK_NUMBER))
            .await
            .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_ticks_slots() {
        let provider = PROVIDER.clone();
        let pool = IUniswapV3PoolInstance::new(POOL_ADDRESS, provider.clone());
        let tick_current = pool.slot0().block(*BLOCK_NUMBER).call().await.unwrap().tick;
        let slots = get_ticks_slots(
            POOL_ADDRESS,
            tick_current,
            tick_current,
            provider.clone(),
            Some(*BLOCK_NUMBER),
        )
        .await
        .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_tick_bitmap_slots() {
        let provider = PROVIDER.clone();
        let slots = get_tick_bitmap_slots(POOL_ADDRESS, provider.clone(), Some(*BLOCK_NUMBER))
            .await
            .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_positions_slots() -> Result<()> {
        let provider = PROVIDER.clone();
        // create a filter to get the mint events
        let filter = Filter::new()
            .from_block(17000000 - 10000)
            .to_block(17000000)
            .event_signature(<Mint as SolEvent>::SIGNATURE_HASH);
        let logs = provider.get_logs(&filter).await?;
        // decode the logs into position keys
        let positions = logs
            .iter()
            .map(|log| <Mint as SolEvent>::decode_log_data(log.data(), true).unwrap())
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
        let slots = get_positions_slots(
            POOL_ADDRESS,
            positions,
            provider.clone(),
            Some(*BLOCK_NUMBER),
        )
        .await?;
        verify_slots(slots, provider).await;
        Ok(())
    }
}

use crate::{
    bindings::{
        ephemeralallpositionsbyowner::{
            EphemeralAllPositionsByOwner,
            EphemeralAllPositionsByOwner::{
                allPositionsCall, allPositionsReturn, EphemeralAllPositionsByOwnerInstance,
            },
        },
        ephemeralgetposition::{
            EphemeralGetPosition,
            EphemeralGetPosition::{
                getPositionCall, getPositionReturn, EphemeralGetPositionInstance,
            },
        },
        ephemeralgetpositions::{
            EphemeralGetPositions,
            EphemeralGetPositions::{
                getPositionsCall, getPositionsReturn, EphemeralGetPositionsInstance,
            },
        },
    },
    call_ephemeral_contract,
};
use alloy::{
    contract::Error,
    eips::BlockId,
    primitives::{Address, Bytes, U256},
    providers::Provider,
    sol_types::SolCall,
    transports::{Transport, TransportError},
};
use anyhow::Result;

pub async fn get_position_details<T, P>(
    npm: Address,
    token_id: U256,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<EphemeralGetPosition::PositionState>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    let deploy_builder = EphemeralGetPositionInstance::deploy_builder(provider, npm, token_id);
    match call_ephemeral_contract!(deploy_builder, getPositionCall, block_id) {
        Ok(getPositionReturn { state }) => Ok(state),
        Err(err) => Err(err.into()),
    }
}

pub async fn get_positions<T, P>(
    npm: Address,
    token_ids: Vec<U256>,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<EphemeralGetPositions::PositionState>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    let deploy_builder = EphemeralGetPositionsInstance::deploy_builder(provider, npm, token_ids);
    match call_ephemeral_contract!(deploy_builder, getPositionsCall, block_id) {
        Ok(getPositionsReturn { positions }) => Ok(positions),
        Err(err) => Err(err.into()),
    }
}

pub async fn get_all_positions_by_owner<T, P>(
    npm: Address,
    owner: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<EphemeralAllPositionsByOwner::PositionState>>
where
    T: Transport + Clone,
    P: Provider<T>,
{
    let deploy_builder = EphemeralAllPositionsByOwnerInstance::deploy_builder(provider, npm, owner);
    match call_ephemeral_contract!(deploy_builder, allPositionsCall, block_id) {
        Ok(allPositionsReturn { positions }) => Ok(positions),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bindings::{
            ephemeralgetposition::EphemeralGetPosition::{PositionFull, Slot0},
            inonfungiblepositionmanager::INonfungiblePositionManager::INonfungiblePositionManagerInstance,
            iuniswapv3pool::IUniswapV3Pool::IUniswapV3PoolInstance,
        },
        tests::*,
    };
    use alloy::{
        primitives::{address, b256, keccak256, uint, B256},
        sol_types::SolValue,
    };

    const FACTORY_ADDRESS: Address = address!("1F98431c8aD98523631AE4a59f267346ea31F984");
    const NPM_ADDRESS: Address = address!("C36442b4a4522E871399CD717aBDD847Ab11FE88");
    static POOL_INIT_CODE_HASH: B256 =
        b256!("e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54");

    /// Computes the address of a Uniswap V3 pool given the factory address, the two tokens, and the
    /// fee.
    ///
    /// # Arguments
    ///
    /// * `factory`: The Uniswap V3 factory address
    /// * `token_0`: The first token address
    /// * `token_1`: The second token address
    /// * `fee`: The fee tier
    /// * `init_code_hash`: The init code hash of the pool
    ///
    /// returns: Address
    fn compute_pool_address(
        factory: Address,
        token_a: Address,
        token_b: Address,
        fee: u32,
        init_code_hash: B256,
    ) -> Address {
        let (token_0, token_1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        let pool_key = (token_0, token_1, fee as i32);
        factory.create2(keccak256(pool_key.abi_encode()), init_code_hash)
    }

    #[tokio::test]
    async fn test_get_position_details() -> Result<()> {
        let provider = PROVIDER.clone();
        let EphemeralGetPosition::PositionState {
            tokenId,
            position:
                PositionFull {
                    token0,
                    token1,
                    fee,
                    ..
                },
            slot0: Slot0 {
                sqrtPriceX96, tick, ..
            },
            ..
        } = get_position_details(
            NPM_ADDRESS,
            uint!(4_U256),
            provider.clone(),
            Some(*BLOCK_NUMBER),
        )
        .await?;
        let pool = IUniswapV3PoolInstance::new(
            compute_pool_address(FACTORY_ADDRESS, token0, token1, fee, POOL_INIT_CODE_HASH),
            provider,
        );
        let slot0 = pool.slot0().block(*BLOCK_NUMBER).call().await?;
        assert_eq!(tokenId, uint!(4_U256));
        assert_eq!(sqrtPriceX96, slot0.sqrtPriceX96);
        assert_eq!(tick, slot0.tick);
        Ok(())
    }

    // async fn verify_position_details(
    //     positions: Vec<PositionState>,
    //     npm: INonfungiblePositionManager<Provider<Http>>,
    // ) -> Result<()> {
    //     assert!(!positions.is_empty());
    //     let client = npm.client();
    //     let mut multicall = Multicall::new(client.clone(), None).await.unwrap();
    //     multicall.add_calls(
    //         false,
    //         positions
    //             .iter()
    //             .map(|PositionState { token_id, .. }| npm.positions(*token_id)),
    //     );
    //     #[allow(clippy::type_complexity)]
    //     let alt_positions: Vec<(
    //         u128,
    //         Address,
    //         Address,
    //         Address,
    //         u32,
    //         i32,
    //         i32,
    //         u128,
    //         U256,
    //         U256,
    //         u128,
    //         u128,
    //     )> = multicall
    //         .block(match *BLOCK_NUMBER {
    //             BlockId::Number(n) => n,
    //             _ => panic!("block id must be a number"),
    //         })
    //         .call_array()
    //         .await?;
    //     for (
    //         i,
    //         &PositionState {
    //             position:
    //                 PositionFull {
    //                     token_0,
    //                     token_1,
    //                     fee,
    //                     tick_lower,
    //                     tick_upper,
    //                     liquidity,
    //                     ..
    //                 },
    //             ..
    //         },
    //     ) in positions.iter().enumerate()
    //     {
    //         let (_, _, _token_0, _token_1, _fee, _tick_lower, _tick_upper, _liquidity, _, _, _,
    // _) =             alt_positions[i];
    //         assert_eq!(token_0, _token_0);
    //         assert_eq!(token_1, _token_1);
    //         assert_eq!(fee, _fee);
    //         assert_eq!(tick_lower, _tick_lower);
    //         assert_eq!(tick_upper, _tick_upper);
    //         assert_eq!(liquidity, _liquidity);
    //     }
    //     Ok(())
    // }

    #[tokio::test]
    async fn test_get_positions() -> Result<()> {
        let provider = PROVIDER.clone();
        let _positions = get_positions(
            NPM_ADDRESS,
            (1u64..100)
                .map(|i| U256::from_limbs([i, 0, 0, 0]))
                .collect(),
            provider.clone(),
            Some(*BLOCK_NUMBER),
        )
        .await?;
        let _npm = INonfungiblePositionManagerInstance::new(NPM_ADDRESS, provider);
        Ok(())
        // verify_position_details(positions, npm).await
    }

    #[tokio::test]
    async fn test_get_all_positions_by_owner() -> Result<()> {
        let provider = PROVIDER.clone();
        let npm = INonfungiblePositionManagerInstance::new(NPM_ADDRESS, provider.clone());
        let total_supply: U256 = npm.totalSupply().block(*BLOCK_NUMBER).call().await?._0;
        let owner = npm
            .ownerOf(total_supply - uint!(1_U256))
            .block(*BLOCK_NUMBER)
            .call()
            .await?
            .owner;
        let _positions =
            get_all_positions_by_owner(NPM_ADDRESS, owner, provider, Some(*BLOCK_NUMBER)).await?;
        Ok(())
        // verify_position_details(positions, npm).await
    }
}

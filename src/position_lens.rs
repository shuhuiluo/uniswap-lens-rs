//! ## Position Lens
//!
//! The position lens module provides functions to fetch position details using ephemeral contracts.

use crate::{
    bindings::{
        ephemeralallpositionsbyowner::{
            EphemeralAllPositionsByOwner, EphemeralAllPositionsByOwner::allPositionsCall,
        },
        ephemeralgetposition::{EphemeralGetPosition, EphemeralGetPosition::getPositionCall},
        ephemeralgetpositions::{EphemeralGetPositions, EphemeralGetPositions::getPositionsCall},
    },
    call_ephemeral_contract,
    error::Error,
};
use alloc::vec::Vec;
use alloy::{
    contract::Error as ContractError,
    eips::BlockId,
    network::Network,
    primitives::{Address, U256},
    providers::Provider,
    sol_types::SolCall,
    transports::TransportError,
};

/// Get the details of a position given the token ID.
///
/// ## Arguments
///
/// * `npm`: The address of the non-fungible position manager
/// * `token_id`: The token ID of the position
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// The position details
#[inline]
pub async fn get_position_details<N, P>(
    npm: Address,
    token_id: U256,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<EphemeralGetPosition::PositionState, Error>
where
    N: Network,
    P: Provider<N>,
{
    let deploy_builder = EphemeralGetPosition::deploy_builder(provider, npm, token_id);
    call_ephemeral_contract!(deploy_builder, getPositionCall, block_id)
}

/// Get the details of multiple positions given the token IDs.
///
/// ## Arguments
///
/// * `npm`: The address of the non-fungible position manager
/// * `token_ids`: The token IDs of the positions
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// The array of position details
#[inline]
pub async fn get_positions<N, P>(
    npm: Address,
    token_ids: Vec<U256>,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<EphemeralGetPositions::PositionState>, Error>
where
    N: Network,
    P: Provider<N>,
{
    let deploy_builder = EphemeralGetPositions::deploy_builder(provider, npm, token_ids);
    call_ephemeral_contract!(deploy_builder, getPositionsCall, block_id)
}

/// Get all positions owned by an address.
///
/// ## Arguments
///
/// * `npm`: The address of the non-fungible position manager
/// * `owner`: The address of the owner
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// The array of position details
#[inline]
pub async fn get_all_positions_by_owner<N, P>(
    npm: Address,
    owner: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<EphemeralAllPositionsByOwner::PositionState>, Error>
where
    N: Network,
    P: Provider<N>,
{
    let deploy_builder = EphemeralAllPositionsByOwner::deploy_builder(provider, npm, owner);
    call_ephemeral_contract!(deploy_builder, allPositionsCall, block_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bindings::{
            ephemeralgetposition::EphemeralGetPosition::{PositionFull, Slot0},
            iuniswapv3nonfungiblepositionmanager::IUniswapV3NonfungiblePositionManager,
            iuniswapv3pool::IUniswapV3Pool,
        },
        tests::*,
    };
    use alloy::{
        primitives::{address, aliases::U24, b256, keccak256, uint, B256},
        providers::{MulticallBuilder, RootProvider},
        sol_types::SolValue,
    };

    const FACTORY_ADDRESS: Address = address!("1F98431c8aD98523631AE4a59f267346ea31F984");
    const NPM_ADDRESS: Address = address!("C36442b4a4522E871399CD717aBDD847Ab11FE88");
    static POOL_INIT_CODE_HASH: B256 =
        b256!("e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54");

    fn compute_pool_address(
        factory: Address,
        token_a: Address,
        token_b: Address,
        fee: U24,
        init_code_hash: B256,
    ) -> Address {
        let (token_0, token_1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        let pool_key = (token_0, token_1, fee);
        factory.create2(keccak256(pool_key.abi_encode()), init_code_hash)
    }

    #[tokio::test]
    async fn test_get_position_details() {
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
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        let pool = IUniswapV3Pool::new(
            compute_pool_address(FACTORY_ADDRESS, token0, token1, fee, POOL_INIT_CODE_HASH),
            provider,
        );
        let slot0 = pool.slot0().block(BLOCK_NUMBER).call().await.unwrap();
        assert_eq!(tokenId, uint!(4_U256));
        assert_eq!(sqrtPriceX96, slot0.sqrtPriceX96);
        assert_eq!(tick, slot0.tick);
    }

    async fn verify_position_details(
        positions: Vec<EphemeralGetPositions::PositionState>,
        npm: IUniswapV3NonfungiblePositionManager::IUniswapV3NonfungiblePositionManagerInstance<
            RootProvider,
        >,
    ) {
        assert!(!positions.is_empty());
        let mut multicall = MulticallBuilder::new_dynamic(npm.provider());
        for EphemeralGetPositions::PositionState { tokenId, .. } in positions.iter() {
            multicall = multicall.add_dynamic(npm.positions(*tokenId));
        }
        let alt_positions: Vec<IUniswapV3NonfungiblePositionManager::positionsReturn> =
            multicall.block(BLOCK_NUMBER).aggregate().await.unwrap();
        for (i, EphemeralGetPositions::PositionState { position, .. }) in
            positions.into_iter().enumerate()
        {
            assert_eq!(position.token0, alt_positions[i].token0);
            assert_eq!(position.token1, alt_positions[i].token1);
            assert_eq!(position.fee, alt_positions[i].fee);
            assert_eq!(position.tickLower, alt_positions[i].tickLower);
            assert_eq!(position.tickUpper, alt_positions[i].tickUpper);
            assert_eq!(position.liquidity, alt_positions[i].liquidity);
        }
    }

    #[tokio::test]
    async fn test_get_positions() {
        let provider = PROVIDER.clone();
        let positions = get_positions(
            NPM_ADDRESS,
            (1_u64..100)
                .map(|i| U256::from_limbs([i, 0, 0, 0]))
                .collect(),
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        let npm = IUniswapV3NonfungiblePositionManager::new(NPM_ADDRESS, provider);
        verify_position_details(positions, npm).await;
    }

    #[tokio::test]
    async fn test_get_all_positions_by_owner() {
        let provider = PROVIDER.clone();
        let npm = IUniswapV3NonfungiblePositionManager::new(NPM_ADDRESS, provider.clone());
        let total_supply: U256 = npm.totalSupply().block(BLOCK_NUMBER).call().await.unwrap();
        let owner = npm
            .ownerOf(total_supply - uint!(1_U256))
            .block(BLOCK_NUMBER)
            .call()
            .await
            .unwrap();
        let positions =
            get_all_positions_by_owner(NPM_ADDRESS, owner, provider, Some(BLOCK_NUMBER))
                .await
                .unwrap();
        // Convert from Vec<EphemeralAllPositionsByOwner::PositionState> to
        // Vec<EphemeralGetPositions::PositionState>
        let positions: Vec<EphemeralGetPositions::PositionState> =
            unsafe { core::mem::transmute(positions) };
        verify_position_details(positions, npm).await
    }
}

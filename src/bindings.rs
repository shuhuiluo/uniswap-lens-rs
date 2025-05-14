/// Creates a module with Solidity bindings for the specified contract.
///
/// ## Arguments
///
/// * `module_name` - The name of the module to create
/// * `contract_name` - The name of the contract to bind
macro_rules! create_sol_binding {
    ($module_name:ident, $contract_name:ident) => {
        pub mod $module_name {
            alloy::sol!(
                #[sol(rpc)]
                $contract_name,
                concat!(
                    "out/",
                    stringify!($contract_name),
                    ".sol/",
                    stringify!($contract_name),
                    ".json"
                )
            );
        }
    };
}

// Use the macro to create all the bindings
create_sol_binding!(ephemeralallpositionsbyowner, EphemeralAllPositionsByOwner);
create_sol_binding!(
    ephemeralgetpopulatedticksinrange,
    EphemeralGetPopulatedTicksInRange
);
create_sol_binding!(ephemeralgetposition, EphemeralGetPosition);
create_sol_binding!(ephemeralgetpositions, EphemeralGetPositions);
create_sol_binding!(ephemeralpoolpositions, EphemeralPoolPositions);
create_sol_binding!(ephemeralpoolslots, EphemeralPoolSlots);
create_sol_binding!(ephemeralpooltickbitmap, EphemeralPoolTickBitmap);
create_sol_binding!(ephemeralpoolticks, EphemeralPoolTicks);
create_sol_binding!(ephemeralstoragelens, EphemeralStorageLens);

create_sol_binding!(iuniswapv3pool, IUniswapV3Pool);
create_sol_binding!(
    iuniswapv3nonfungiblepositionmanager,
    IUniswapV3NonfungiblePositionManager
);

create_sol_binding!(ierc20, IERC20);
create_sol_binding!(ierc20metadata, IERC20Metadata);
create_sol_binding!(ierc721enumerable, IERC721Enumerable);

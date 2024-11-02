//! Solidity types for IBC Store

/// The storage slot for the `mapping(bytes32 => bytes32) public commitments` mapping in the
/// `IBCStore` contract.
pub const IBC_STORE_COMMITMENTS_SLOT: u64 = 1;

#[cfg(feature = "rpc")]
alloy_sol_types::sol!(
    #[sol(rpc)]
    #[derive(serde::Deserialize, serde::Serialize)]
    store,
    "../../node_modules/solidity-ibc/abi/IBCStore.json"
);

// NOTE: The riscv programs won't compile with the `rpc` features.
#[cfg(not(feature = "rpc"))]
alloy_sol_types::sol!(
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(missing_docs, clippy::pedantic)]
    store,
    "../../node_modules/solidity-ibc/abi/IBCStore.json"
);

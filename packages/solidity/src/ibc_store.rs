//! Solidity types for IBC Store

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

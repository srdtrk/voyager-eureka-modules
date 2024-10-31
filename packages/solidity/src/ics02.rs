//! Solidity types for ICS02Client.sol

#[cfg(feature = "rpc")]
alloy_sol_types::sol!(
    #[sol(rpc)]
    #[derive(serde::Deserialize, serde::Serialize)]
    client,
    "../../node_modules/solidity-ibc/abi/ICS02Client.json"
);

// NOTE: The riscv programs won't compile with the `rpc` features.
#[cfg(not(feature = "rpc"))]
alloy_sol_types::sol!(
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(missing_docs, clippy::pedantic)]
    client,
    "../../node_modules/solidity-ibc/abi/ICS02Client.json"
);

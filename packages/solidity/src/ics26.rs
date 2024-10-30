//! Solidity types for ICS26Router.sol

#[cfg(feature = "rpc")]
alloy_sol_types::sol!(
    #[sol(rpc)]
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(missing_docs, clippy::pedantic, warnings)]
    router,
    "../../node_modules/solidity-ibc/abi/ICS26Router.json"
);

// NOTE: The riscv programs won't compile with the `rpc` features.
#[cfg(not(feature = "rpc"))]
alloy_sol_types::sol!(
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(missing_docs, clippy::pedantic)]
    router,
    "../../node_modules/solidity-ibc/abi/ICS26Router.json"
);

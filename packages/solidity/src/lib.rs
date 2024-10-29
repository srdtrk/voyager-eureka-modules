#![doc = include_str!("../README.md")]
#![deny(clippy::nursery, clippy::pedantic, warnings)]

#[cfg(feature = "rpc")]
alloy_sol_types::sol!(
    #[sol(rpc)]
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(missing_docs, clippy::pedantic, warnings)]
    ics26_router,
    "../../node_modules/solidity-ibc/abi/ICS26Router.json"
);

// NOTE: The riscv program won't compile with the `rpc` features.
#[cfg(not(feature = "rpc"))]
alloy_sol_types::sol!(
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(missing_docs, clippy::pedantic)]
    ics26_router,
    "../../node_modules/solidity-ibc/abi/ICS26Router.json"
);

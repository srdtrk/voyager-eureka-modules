//! # Shared Types for IBC Eureka Voyager Modules

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

/// The name of the IBC Eureka Interface (required by voyager)
pub const IBC_EUREKA_INTERFACE: &str = "solidity-ibc-eureka";

/// The name of the sp1-ics07-tendermint client (required by voyager)
pub const SP1_ICS07_CLIENT_TYPE: &str = "sp1-ics07-tendermint";

//! SP1 ICS07 Light Client Module for Voyager

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

/// The supported IBC interfaces for SP1 ICS07 family of light clients
#[derive(Debug, Clone)]
pub enum SupportedIbcInterfaces {
    /// The Solidity IBC Eureka interface
    SolidityIbcEureka,
}

/// The supported zero-knowledge proof algorithms
#[derive(Debug, Clone)]
pub enum SupportedZkAlgorithms {
    /// SP1's Groth16
    Groth16,
    /// SP1's Plonk
    Plonk,
}

/// The configuration for the SP1 ICS07 Light Client Module
/// No configuration is required for this module
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {}

/// The SP1 ICS07 Light Client Module
#[derive(Debug, Clone)]
pub struct Module {
    /// The ibc interface used in this instance
    pub ibc_interface: SupportedIbcInterfaces,
    /// The zero-knowledge algorithm used in this instance
    pub zk_algorithm: SupportedZkAlgorithms,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    todo!()
    //run_client_module_server::<Module>().await
}

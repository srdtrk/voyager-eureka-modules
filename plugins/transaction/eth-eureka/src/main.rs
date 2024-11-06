//! # Ethereum IBC Eureka transaction module

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

mod call;
mod callback;
mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    //run_plugin_server::<Module>().await
    todo!()
}

/// The configuration for the Ethereum IBC Eureka transaction module
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {}

/// The Ethereum IBC Eureka transaction module
#[derive(Debug, Clone)]
pub struct Module {}

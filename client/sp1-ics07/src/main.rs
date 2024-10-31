//! SP1 ICS07 Light Client Module for Voyager

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

use ibc_eureka_types::SOL_IBC_EUREKA_INTERFACE;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde_json::Value;
use serde_utils::Hex;
use voyager_message::{
    core::{ClientStateMeta, ClientType, ConsensusStateMeta},
    module::{ClientModuleInfo, ClientModuleServer},
    run_client_module_server, ClientModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_client_module_server::<Module>().await;
}

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

impl ClientModule for Module {
    type Config = Config;

    async fn new(Config {}: Self::Config, _info: ClientModuleInfo) -> Result<Self, BoxDynError> {
        todo!()
    }
}

#[async_trait]
impl ClientModuleServer for Module {
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        _client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta> {
        todo!()
    }

    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        _consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<ConsensusStateMeta> {
        todo!()
    }

    async fn decode_client_state(
        &self,
        _: &Extensions,
        _client_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        todo!()
    }

    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        _consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        todo!()
    }

    async fn encode_client_state(
        &self,
        _: &Extensions,
        _client_state: Value,
        _metadata: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
        todo!()
    }

    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        _consensus_state: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
        todo!()
    }

    async fn reencode_counterparty_client_state(
        &self,
        _: &Extensions,
        _client_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        todo!()
    }

    async fn reencode_counterparty_consensus_state(
        &self,
        _: &Extensions,
        _consensus_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        todo!()
    }

    async fn encode_header(&self, _: &Extensions, _header: Value) -> RpcResult<Hex<Vec<u8>>> {
        todo!()
    }

    async fn encode_proof(&self, _: &Extensions, _proof: Value) -> RpcResult<Hex<Vec<u8>>> {
        todo!()
    }
}

impl TryFrom<String> for SupportedIbcInterfaces {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            SOL_IBC_EUREKA_INTERFACE => Ok(Self::SolidityIbcEureka),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl From<SupportedIbcInterfaces> for String {
    fn from(value: SupportedIbcInterfaces) -> Self {
        match value {
            SupportedIbcInterfaces::SolidityIbcEureka => SOL_IBC_EUREKA_INTERFACE.to_string(),
        }
    }
}

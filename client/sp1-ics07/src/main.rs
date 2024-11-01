//! SP1 ICS07 Light Client Module for Voyager

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

use alloy::sol_types::SolValue;
use ibc_client_tendermint_types::Header;
use ibc_eureka_types::SOL_IBC_EUREKA_INTERFACE;
use ibc_eureka_union_ext::height::IntoUnionHeight;
use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde_json::{json, Value};
use serde_utils::Hex;
use sp1_ics07_tendermint_solidity::{
    IICS07TendermintMsgs::{ClientState, ConsensusState},
    IMembershipMsgs::MembershipProof,
};
use tendermint_proto::Protobuf;
use unionlabs::ErrorReporter;
use voyager_message::{
    core::{ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType},
    module::{ClientModuleInfo, ClientModuleServer},
    run_client_module_server, ClientModule, FATAL_JSONRPC_ERROR_CODE,
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

    async fn new(Config {}: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError> {
        info.ensure_client_type(ibc_eureka_types::SP1_ICS07_CLIENT_TYPE)?;
        info.ensure_consensus_type(ConsensusType::TENDERMINT)?;

        Ok(Self {
            ibc_interface: SupportedIbcInterfaces::try_from(info.ibc_interface.to_string())?,
            zk_algorithm: SupportedZkAlgorithms::Plonk,
        })
    }
}

#[async_trait]
impl ClientModuleServer for Module {
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta> {
        let cs = self.decode_client_state(&client_state.0)?;

        Ok(ClientStateMeta {
            chain_id: ChainId::new(cs.chainId.as_str().to_owned()),
            height: cs.latestHeight.into_unionlabs_height(),
        })
    }

    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = self.decode_consensus_state(&consensus_state.0)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.timestamp * 1_000_000_000, // convert to nanoseconds
        })
    }

    async fn decode_client_state(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_client_state(&client_state.0)?).unwrap())
    }

    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_consensus_state(&consensus_state.0)?).unwrap())
    }

    async fn encode_client_state(
        &self,
        _: &Extensions,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<ClientState>(client_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .and_then(|cs| match self.ibc_interface {
                SupportedIbcInterfaces::SolidityIbcEureka => {
                    if !metadata.is_null() {
                        return Err(ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            "metadata was provided, but this client type does not require \
                            metadata for client state encoding",
                            Some(json!({
                                "provided_metadata": metadata,
                            })),
                        ));
                    }

                    Ok(cs.abi_encode())
                }
            })
            .map(Hex)
    }

    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!(
                        "unable to deserialize consensus state: {}",
                        ErrorReporter(err)
                    ),
                    None::<()>,
                )
            })
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterfaces::SolidityIbcEureka => cs.abi_encode(),
            })
            .map(Hex)
    }

    async fn reencode_counterparty_client_state(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(client_state)
    }

    async fn reencode_counterparty_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(consensus_state)
    }

    // NOTE: We always serialize the header using protobuf
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<Header>(header)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize header: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(<Header as Protobuf<RawHeader>>::encode_vec)
            .map(Hex)
    }

    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<MembershipProof>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|proof| match self.ibc_interface {
                SupportedIbcInterfaces::SolidityIbcEureka => proof.abi_encode(),
            })
            .map(Hex)
    }
}

impl Module {
    /// Decode a consensus state from bytes
    /// # Errors
    /// Fails if the consensus state cannot be decoded
    pub fn decode_consensus_state(&self, consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        match self.ibc_interface {
            SupportedIbcInterfaces::SolidityIbcEureka => {
                ConsensusState::abi_decode(consensus_state, false).map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode consensus state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
            }
        }
    }

    /// Decode a client state from bytes
    /// # Errors
    /// Fails if the client state cannot be decoded
    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterfaces::SolidityIbcEureka => {
                ClientState::abi_decode(client_state, false).map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
            }
        }
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

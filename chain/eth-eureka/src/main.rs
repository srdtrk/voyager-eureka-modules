//! Ethereum Chain Module for IBC Eureka

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

use std::num::NonZeroU64;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    sol_types::SolValue,
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use ethereum_light_client_types::StorageProof;
use ibc_eureka_solidity::{
    ibc_store::{store as ibc_store, store::storeInstance, IBC_STORE_COMMITMENTS_SLOT},
    ics02::client as ics02_client,
    ics26::router::{self as ics26_router, routerInstance},
};
use ibc_eureka_union_ext::path::IbcEurekaPathExt;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sp1_ics07_tendermint_solidity::sp1_ics07_tendermint;
use unionlabs::{
    bytes::Bytes,
    ethereum::ibc_commitment_key,
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    ics24::{AcknowledgementPath, ClientStatePath, CommitmentPath, Path, ReceiptPath},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    uint::U256,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    run_chain_module_server, ChainModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_chain_module_server::<Module>().await;
}

/// The Ethereum Eureka Chain Module
#[derive(Debug, Clone)]
pub struct Module {
    /// The chain ID of the Ethereum chain
    pub chain_id: ChainId<'static>,

    /// The ics26 router contract instance
    pub ics26_router: routerInstance<BoxTransport, RootProvider<BoxTransport>>,

    /// The ethereum provider
    pub eth_provider: RootProvider<BoxTransport>,
    /// The RPC endpoint for the beacon api.
    pub beacon_api_client: BeaconApiClient,
}

/// The configuration for the Ethereum Eureka Chain Module
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ics26_router_address: String,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl ChainModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ChainModuleInfo) -> Result<Self, BoxDynError> {
        let eth_provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let chain_id = eth_provider.get_chain_id().await?;

        info.ensure_chain_id(U256::from(chain_id).to_string())?;

        let ics26_router =
            ics26_router::new(config.ics26_router_address.parse()?, eth_provider.clone());

        Ok(Self {
            chain_id: ChainId::new(U256::from(chain_id).to_string()),
            ics26_router,
            eth_provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await?,
        })
    }
}

impl Module {
    /// Create a new height with the revision number set to the Ethereum revision number.
    #[must_use]
    pub const fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }

    /// Get the execution height of a beacon slot.
    /// # Panics
    /// Panics if the beacon api call fails.
    pub async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        self.beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap()
    }

    /// Get the IBC store contract instance.
    /// # Panics
    /// Panics if the contract call fails.
    pub async fn ibc_store_contract(
        &self,
    ) -> storeInstance<BoxTransport, RootProvider<BoxTransport>> {
        ibc_store::new(
            self.ics26_router.IBC_STORE().call().await.unwrap()._0,
            self.eth_provider.clone(),
        )
    }

    /// Get the IBC client contract instance.
    /// # Panics
    /// Panics if the contract call fails.
    // TODO: Use a generic light client interface
    pub async fn ibc_client_contract(
        &self,
        client_id: ClientId,
    ) -> sp1_ics07_tendermint::sp1_ics07_tendermintInstance<BoxTransport, RootProvider<BoxTransport>>
    {
        let ics02_address = self.ics26_router.ICS02_CLIENT().call().await.unwrap()._0;
        let ics02_contract = ics02_client::new(ics02_address, self.eth_provider.clone());
        let sp1_ics07_address = ics02_contract
            .getClient(client_id.to_string())
            .call()
            .await
            .unwrap()
            ._0;
        sp1_ics07_tendermint::new(sp1_ics07_address, self.eth_provider.clone())
    }

    /// Fetch the IBC state at a given height and path.
    /// # Errors
    /// Returns an error if the contract calls fail.
    /// # Panics
    /// Panics if the requested path is not implemented in IBC Eureka.
    pub async fn fetch_ibc_state(
        &self,
        path: Path,
        height: Height,
    ) -> Result<Option<Bytes>, BoxDynError> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        Ok(match path {
            Path::ClientState(path) => {
                let client_state = self
                    .ibc_client_contract(path.client_id)
                    .await
                    .getClientState()
                    .block(execution_height.into())
                    .call()
                    .await
                    .unwrap()
                    ._0;

                Some(Bytes::from(client_state.abi_encode()))
            }
            Path::Commitment(_) | Path::Acknowledgement(_) | Path::Receipt(_) => {
                let commitment = self
                    .ibc_store_contract()
                    .await
                    .getCommitment(path.to_storage_key().into())
                    .block(execution_height.into())
                    .call()
                    .await
                    .unwrap()
                    ._0;

                if commitment.is_zero() {
                    return Ok(None);
                }

                Some(Bytes::from(commitment.abi_encode()))
            }
            Path::ClientConsensusState(_)
            | Path::Connection(_)
            | Path::ChannelEnd(_)
            | Path::NextSequenceSend(_)
            | Path::NextSequenceRecv(_)
            | Path::NextSequenceAck(_)
            | Path::NextConnectionSequence(_)
            | Path::NextClientSequence(_) => {
                unimplemented!()
            }
        })
    }
}

#[async_trait]
impl ChainModuleServer for Module {
    /// Query the latest finalized height of this chain.
    async fn query_latest_height(&self, _: &Extensions) -> RpcResult<Height> {
        self.beacon_api_client
            .finality_update()
            .await
            .map(|response| self.make_height(response.data.attested_header.beacon.slot))
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    async fn query_latest_timestamp(&self, _: &Extensions) -> RpcResult<i64> {
        Ok(self
            .beacon_api_client
            .finality_update()
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .data
            .attested_header
            .execution
            .timestamp
            .try_into()
            .unwrap())
    }

    async fn query_client_prefix(&self, _: &Extensions, _raw_client_id: u32) -> RpcResult<String> {
        // NOTE: We only support one client type for now
        Ok("07-tendermint".to_string())
    }

    async fn client_info(&self, _: &Extensions, _client_id: ClientId) -> RpcResult<ClientInfo> {
        // NOTE: We only support one client type for now
        Ok(ClientInfo {
            client_type: ClientType::new(ibc_eureka_types::SP1_ICS07_CLIENT_TYPE),
            ibc_interface: IbcInterface::new(ibc_eureka_types::SOL_IBC_EUREKA_INTERFACE),
            metadata: Value::default(),
        })
    }

    async fn query_client_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Bytes> {
        let path = Path::ClientState(ClientStatePath { client_id });

        self.fetch_ibc_state(path, height)
            .await
            .map(Option::unwrap_or_default)
            .map_err(|err| ErrorObject::owned(-1, err.to_string(), None::<()>))
    }

    async fn query_commitment(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let path = Path::Commitment(CommitmentPath {
            port_id,
            channel_id,
            sequence,
        });

        self.fetch_ibc_state(path, height)
            .await
            .map(|commitment| {
                commitment.map(|commitment| {
                    let fixed_length_commitment: [u8; 32] = commitment
                        .into_vec()
                        .try_into()
                        .expect("commitment should be 32 bytes long");

                    fixed_length_commitment.into()
                })
            })
            .map_err(|err| ErrorObject::owned(-1, err.to_string(), None::<()>))
    }

    async fn query_acknowledgement(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let path = Path::Acknowledgement(AcknowledgementPath {
            port_id,
            channel_id,
            sequence,
        });

        self.fetch_ibc_state(path, height)
            .await
            .map(|commitment| {
                commitment.map(|commitment| {
                    let fixed_length_commitment: [u8; 32] = commitment
                        .into_vec()
                        .try_into()
                        .expect("ack should be 32 bytes long");

                    fixed_length_commitment.into()
                })
            })
            .map_err(|err| ErrorObject::owned(-1, err.to_string(), None::<()>))
    }

    async fn query_receipt(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<bool> {
        let path = Path::Receipt(ReceiptPath {
            port_id,
            channel_id,
            sequence,
        });

        self.fetch_ibc_state(path, height)
            .await
            .map(|commitment| commitment.is_some())
            .map_err(|err| ErrorObject::owned(-1, err.to_string(), None::<()>))
    }

    async fn query_ibc_proof(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        let location = ibc_commitment_key(
            path.to_storage_key().into(),
            IBC_STORE_COMMITMENTS_SLOT.into(),
        );

        let execution_height = self.execution_height_of_beacon_slot(at.height()).await;

        let proof = self
            .eth_provider
            .get_proof(
                *self.ics26_router.address(),
                vec![location.to_be_bytes().into()],
            )
            .block_id(execution_height.into())
            .await
            .unwrap();

        let proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
            }
        };
        let proof = StorageProof {
            key: U256::from_be_bytes(proof.key.0 .0),
            value: U256::from_be_bytes(proof.value.to_be_bytes()),
            proof: proof
                .proof
                .into_iter()
                .map(|bytes| bytes.to_vec())
                .collect(),
        };

        Ok(serde_json::to_value(proof).expect("serialization is infallible; qed;"))
    }

    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        e: &Extensions,
        client_id: ClientId,
    ) -> RpcResult<RawClientState> {
        let latest_execution_height = self.eth_provider.get_block_number().await.unwrap();

        let client_state = self
            .fetch_ibc_state(
                ClientStatePath {
                    client_id: client_id.clone(),
                }
                .into(),
                self.make_height(latest_execution_height),
            )
            .await
            .unwrap();

        let client_state_bytes = serde_json::to_vec(&client_state).unwrap();

        let ClientInfo {
            client_type,
            ibc_interface,
            metadata: _,
        } = self.client_info(e, client_id).await?;

        Ok(RawClientState {
            client_type,
            ibc_interface,
            bytes: client_state_bytes.into(),
        })
    }

    async fn query_client_consensus_state(
        &self,
        _: &Extensions,
        _height: Height,
        _client_id: ClientId,
        _trusted_height: Height,
    ) -> RpcResult<Bytes> {
        unimplemented!("solidity_ibc_eureka does not store client consensus states")
    }

    async fn query_connection(
        &self,
        _: &Extensions,
        _height: Height,
        _connection_id: ConnectionId,
    ) -> RpcResult<Option<ConnectionEnd>> {
        unimplemented!("ibc_eureka does not support connections")
    }

    async fn query_channel(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
        unimplemented!("ibc_eureka does not support channels")
    }

    async fn query_next_sequence_send(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
    ) -> RpcResult<u64> {
        unimplemented!("ibc_eureka does not support provable sequences")
    }

    async fn query_next_sequence_recv(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
    ) -> RpcResult<u64> {
        unimplemented!("ibc_eureka does not support provable sequences")
    }

    async fn query_next_sequence_ack(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
    ) -> RpcResult<u64> {
        unimplemented!("ibc_eureka does not support provable sequences")
    }

    async fn query_next_connection_sequence(
        &self,
        _: &Extensions,
        _height: Height,
    ) -> RpcResult<u64> {
        unimplemented!("ibc_eureka does not support provable sequences")
    }

    async fn query_next_client_sequence(&self, _: &Extensions, _height: Height) -> RpcResult<u64> {
        unimplemented!("ibc_eureka does not support provable sequences")
    }
}

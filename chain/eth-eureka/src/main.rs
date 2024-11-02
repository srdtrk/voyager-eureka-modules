//! Ethereum Chain Module for IBC Eureka

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    sol_types::SolValue,
};
use beacon_api::client::BeaconApiClient;
use ibc_eureka_solidity::{
    ibc_store::{store as ibc_store, IBC_STORE_COMMITMENTS_SLOT},
    ics02::client as ics02_client,
    ics26::router as ics26_router,
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
    ethereum::ibc_commitment_key,
    ibc::{core::client::height::Height, lightclients::ethereum::storage_proof::StorageProof},
    ics24::{ClientStatePath, Path},
    id::ClientId,
    uint::U256,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    run_chain_module_server, ChainModule,
};
use voyager_vm::BoxDynError;

const ETHEREUM_REVISION_NUMBER: u64 = 0;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_chain_module_server::<Module>().await;
}

/// The Ethereum Eureka Chain Module
#[derive(Debug, Clone)]
pub struct Module {
    /// The chain ID of the Ethereum chain
    pub chain_id: ChainId<'static>,

    /// The address of the `ICS26Router.sol` smart contract.
    pub ics26_router_address: Address,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: reqwest::Url,
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
        let eth_rpc_api = reqwest::Url::parse(&config.eth_rpc_api)?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(eth_rpc_api.clone());

        let chain_id = provider.get_chain_id().await?;

        info.ensure_chain_id(U256::from(chain_id).to_string())?;

        Ok(Self {
            chain_id: ChainId::new(U256::from(chain_id).to_string()),
            ics26_router_address: config.ics26_router_address.parse()?,
            eth_rpc_api,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await?,
        })
    }
}

impl Module {
    /// Create a new Ethereum Eureka Chain Module
    /// # Errors
    /// Returns an error if the chain ID does not match the expected chain ID.
    /// Returns an error if the api calls fail.
    pub async fn new(config: Config) -> Result<Self, BoxDynError> {
        let eth_rpc_api = reqwest::Url::parse(&config.eth_rpc_api)?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(eth_rpc_api.clone());

        let chain_id = provider.get_chain_id().await?;

        Ok(Self {
            chain_id: ChainId::new(U256::from(chain_id).to_string()),
            ics26_router_address: config.ics26_router_address.parse()?,
            eth_rpc_api,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await?,
        })
    }

    /// Create a new height with the revision number set to the Ethereum revision number.
    #[must_use]
    pub const fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: ETHEREUM_REVISION_NUMBER,
            revision_height: height,
        }
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

    /// Fetch the IBC state at a given height and path.
    /// # Errors
    /// Returns an error if the contract calls fail.
    /// # Panics
    /// Panics if the requested path is not implemented in IBC Eureka.
    pub async fn fetch_ibc_state(&self, path: Path, height: Height) -> Result<Value, BoxDynError> {
        let execution_height = self
            .execution_height_of_beacon_slot(height.revision_height)
            .await;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(self.eth_rpc_api.clone());
        let ics26_contract = ics26_router::new(self.ics26_router_address, provider.clone());

        Ok(match path {
            Path::ClientState(path) => {
                let ics02_address = ics26_contract.ICS02_CLIENT().call().await.unwrap()._0;
                let ics02_contract = ics02_client::new(ics02_address, provider.clone());
                let sp1_ics07_address = ics02_contract
                    .getClient(path.client_id.to_string())
                    .call()
                    .await
                    .unwrap()
                    ._0;
                let sp1_ics07_contract = sp1_ics07_tendermint::new(sp1_ics07_address, provider);
                let client_state = sp1_ics07_contract
                    .getClientState()
                    .block(execution_height.into())
                    .call()
                    .await
                    .unwrap()
                    ._0;
                serde_json::to_value(client_state.abi_encode()).unwrap()
            }
            Path::Commitment(_) | Path::Acknowledgement(_) | Path::Receipt(_) => {
                let ibc_store_address = ics26_contract.IBC_STORE().call().await.unwrap()._0;
                let ibc_store_contract = ibc_store::new(ibc_store_address, provider);

                let commitment = ibc_store_contract
                    .getCommitment(path.to_storage_key().into())
                    .block(execution_height.into())
                    .call()
                    .await
                    .unwrap()
                    ._0;

                serde_json::to_value(commitment.abi_encode()).unwrap()
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

    /// Query the latest (non-finalized) height of this chain.
    async fn query_latest_height_as_destination(&self, _: &Extensions) -> RpcResult<Height> {
        let height = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Head)
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .data
            .message
            .slot;

        Ok(self.make_height(height))
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

    async fn client_info(&self, _: &Extensions, _client_id: ClientId) -> RpcResult<ClientInfo> {
        // NOTE: We only support one client type for now
        Ok(ClientInfo {
            client_type: ClientType::new(ibc_eureka_types::SP1_ICS07_CLIENT_TYPE),
            ibc_interface: IbcInterface::new(ibc_eureka_types::SOL_IBC_EUREKA_INTERFACE),
            metadata: Value::default(),
        })
    }

    async fn query_ibc_state(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        self.fetch_ibc_state(path, at).await.map_err(|err| {
            ErrorObject::owned(
                -1,
                format!("error fetching ibc state: {}", ErrorReporter(&*err)),
                None::<()>,
            )
        })
    }

    async fn query_ibc_proof(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        let location = ibc_commitment_key(&path.to_string(), IBC_STORE_COMMITMENTS_SLOT.into());

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(self.eth_rpc_api.clone());

        let execution_height = self
            .execution_height_of_beacon_slot(at.revision_height)
            .await;

        let proof = provider
            .get_proof(
                self.ics26_router_address,
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
    ) -> RpcResult<RawClientState<'static>> {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(self.eth_rpc_api.clone());

        let latest_execution_height = provider.get_block_number().await.unwrap();

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
}

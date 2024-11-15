//! # ICS07 Light Client Update Plugin

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

mod call;
mod callback;
mod data;

use std::{collections::VecDeque, env, str::FromStr};

use alloy_sol_types::SolValue;
use call::{FetchSP1Proof, FetchUpdate, ModuleCall};
use callback::ModuleCallback;
use data::{ModuleData, ProveResponse};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use sp1_ics07_tendermint_prover::{
    programs::UpdateClientProgram,
    prover::{SP1ICS07TendermintProver, SupportedProofType},
};
use sp1_ics07_tendermint_solidity::{
    IICS02ClientMsgs::Height,
    IICS07TendermintMsgs::{ClientState, TrustThreshold as SolTrustThreshold},
    ISP1Msgs::{SP1Proof, SupportedZkAlgorithm},
};
use sp1_ics07_tendermint_utils::{light_block::LightBlockExt, rpc::TendermintRpcExt};
use sp1_sdk::HashableKey;
use tendermint_rpc::{HttpClient, Url};
use voyager_message::{
    call::{Call, WaitForHeight},
    core::ChainId,
    data::Data,
    hook::UpdateHook,
    module::{PluginInfo, PluginServer},
    run_plugin_server, DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{call, data, pass::PassResult, seq, void, BoxDynError, Op, Visit};

/// The configuration for the SP1 ICS07 Light Client Update Plugin
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Chain ID
    pub chain_id: ChainId<'static>,

    /// Tendermint RPC URL
    pub tm_rpc_url: String,

    /// SP1 prover for the prover network
    /// Should be one of "network", "local", "mock"
    pub sp1_prover: String,

    /// SP1 key for the prover network if `sp1_prover` is "network"
    pub sp1_private_key: String,

    /// Trust threshold
    pub trust_threshold: SolTrustThreshold,

    /// Trusting period
    pub trusting_period: u32,

    /// Proof type
    /// Should be one of "groth16", "plonk"
    pub proof_type: String,
}

/// The SP1 ICS07 Light Client Update Plugin
pub struct Module {
    /// Chain ID
    pub chain_id: ChainId<'static>,

    /// Tendermint RPC client
    pub tm_client: HttpClient,
    /// SP1 ICS07 Tendermint Prover for client update
    pub client_update_prover: SP1ICS07TendermintProver<UpdateClientProgram>,
    /// Trust threshold
    pub trust_threshold: SolTrustThreshold,
    /// Trusting period
    pub trusting_period: u32,
    /// Proof type
    pub proof_type: SupportedProofType,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_plugin_server::<Module>().await;
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let tm_client = HttpClient::new(Url::from_str(&config.tm_rpc_url)?)?;

        let tm_chain_id = tm_client.get_light_block(None).await?.chain_id()?;
        if tm_chain_id.as_str() != config.chain_id.as_str() {
            return Err(format!(
                "Chain ID mismatch: expected {}, got {}",
                config.chain_id, tm_chain_id
            )
            .into());
        }

        // NOTE: SP1 SDK only supports initializing through environment variables
        env::set_var("SP1_PROVER", &config.sp1_prover);
        env::set_var("SP1_PRIVATE_KEY", &config.sp1_private_key);

        let proof_type = match config.proof_type.as_str() {
            "groth16" => SupportedProofType::Groth16,
            "plonk" => SupportedProofType::Plonk,
            _ => return Err(format!("Unsupported proof type: {}", config.proof_type).into()),
        };

        let client_update_prover = SP1ICS07TendermintProver::<UpdateClientProgram>::new(proof_type);

        Ok(Self {
            chain_id: config.chain_id,
            tm_client,
            client_update_prover,
            trust_threshold: config.trust_threshold,
            trusting_period: config.trusting_period,
            proof_type,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(&config.chain_id),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                update_to,
                update_from,
            }) => {
                let trusted_light_block = self
                    .tm_client
                    .get_light_block(Some(update_from.height().try_into().unwrap()))
                    .await
                    .unwrap();

                // Get trusted consensus state from the trusted light block.
                let trusted_consensus_state = trusted_light_block.to_consensus_state();

                let target_light_block = self
                    .tm_client
                    .get_light_block(Some(update_to.height().try_into().unwrap()))
                    .await
                    .unwrap();

                // Get the proposed header from the target light block.
                let proposed_header = target_light_block.into_header(&trusted_light_block);

                Ok(seq([
                    void(call(WaitForHeight {
                        chain_id: self.chain_id.clone(),
                        height: update_to,
                    })),
                    call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(FetchSP1Proof {
                            trusted_consensus_state,
                            proposed_header,
                        }),
                    )),
                ]))
            }
            ModuleCall::FetchSP1Proof(FetchSP1Proof {
                trusted_consensus_state,
                proposed_header,
            }) => {
                let trusted_consensus_state = trusted_consensus_state.into();
                let proof = self.client_update_prover.generate_proof(
                    &self.to_client_state(),
                    &trusted_consensus_state,
                    &proposed_header,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );

                let sp1_proof = SP1Proof::new(
                    &self.client_update_prover.vkey.bytes32(),
                    proof.bytes(),
                    proof.public_values.to_vec(),
                );

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(ProveResponse {
                        trusted_consensus_state: trusted_consensus_state.into(),
                        proposed_header,
                        sp1_proof: sp1_proof.abi_encode(),
                    }),
                )))
            }
        }
    }

    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    UpdateHook::new(&self.chain_id, |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                update_from: fetch.update_from,
                                update_to: fetch.update_to,
                            }),
                        ))
                    })
                    .visit_op(&mut op);

                    op
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    async fn callback(
        &self,
        _: &Extensions,
        _aggregate: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        unimplemented!()
    }
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    // TODO: Call into the eth-eureka chain module to get the client state
    fn to_client_state(&self) -> ClientState {
        ClientState {
            chainId: self.chain_id.to_string(),
            trustLevel: self.trust_threshold.clone(),
            trustingPeriod: self.trusting_period,
            zkAlgorithm: Into::<SupportedZkAlgorithm>::into(self.proof_type).into(),
            unbondingPeriod: self.trusting_period,
            latestHeight: Height {
                // irrelevant
                revisionNumber: 0,
                revisionHeight: 0,
            },
            isFrozen: false,
        }
    }
}

fn plugin_name(chain_id: &ChainId<'_>) -> String {
    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    format!("{PKG_NAME}/{chain_id}")
}

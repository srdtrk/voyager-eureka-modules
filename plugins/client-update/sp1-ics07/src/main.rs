//! # ICS07 Light Client Update Plugin

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

mod call;
mod callback;

use std::{collections::VecDeque, env, str::FromStr};

use call::ModuleCall;
use callback::ModuleCallback;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use sp1_ics07_tendermint_prover::{
    programs::UpdateClientProgram, prover::SP1ICS07TendermintProver,
};
use sp1_ics07_tendermint_utils::{light_block::LightBlockExt, rpc::TendermintRpcExt};
use tendermint_rpc::{HttpClient, Url};
use voyager_message::{
    core::ChainId,
    data::Data,
    module::{PluginInfo, PluginServer},
    run_plugin_server, DefaultCmd, Plugin, VoyagerMessage,
};
use voyager_vm::{pass::PassResult, BoxDynError, Op};

/// The configuration for the SP1 ICS07 Light Client Update Plugin
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
}

/// The SP1 ICS07 Light Client Update Plugin
pub struct Module {
    /// Chain ID
    pub chain_id: ChainId<'static>,

    /// Tendermint RPC client
    pub tm_client: HttpClient,
    /// SP1 ICS07 Tendermint Prover for client update
    pub client_update_prover: SP1ICS07TendermintProver<UpdateClientProgram>,
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

        let client_update_prover = SP1ICS07TendermintProver::<UpdateClientProgram>::default();

        Ok(Self {
            chain_id: config.chain_id,
            tm_client,
            client_update_prover,
        })
    }

    fn info(_config: Self::Config) -> PluginInfo {
        todo!()
    }

    async fn cmd(_config: Self::Config, _cmd: Self::Cmd) {
        todo!()
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    async fn call(&self, _: &Extensions, _call: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        todo!()
    }

    async fn run_pass(
        &self,
        _: &Extensions,
        _msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        todo!()
    }

    async fn callback(
        &self,
        _: &Extensions,
        _aggregate: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        todo!()
    }
}

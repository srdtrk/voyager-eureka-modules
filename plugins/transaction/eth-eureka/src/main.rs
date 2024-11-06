//! # Ethereum IBC Eureka transaction module

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

use std::collections::VecDeque;

use call::ModuleCall;
use callback::ModuleCallback;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use voyager_message::{
    data::Data,
    module::{PluginInfo, PluginServer},
    run_plugin_server, DefaultCmd, Plugin, VoyagerMessage,
};
use voyager_vm::{pass::PassResult, BoxDynError, Op};

mod call;
mod callback;
mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_plugin_server::<Module>().await;
}

/// The configuration for the Ethereum IBC Eureka transaction module
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {}

/// The Ethereum IBC Eureka transaction module
#[derive(Debug, Clone)]
pub struct Module {}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(_config: Self::Config) -> Result<Self, BoxDynError> {
        todo!()
    }

    fn info(_config: Self::Config) -> PluginInfo {
        todo!()
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    async fn call(&self, _: &Extensions, _msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
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
        unimplemented!()
    }
}

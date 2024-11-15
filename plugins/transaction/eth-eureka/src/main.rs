//! # Ethereum IBC Eureka transaction module

#![deny(clippy::nursery, clippy::pedantic, warnings, missing_docs)]

use std::{collections::VecDeque, str::FromStr};

use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::Address,
    providers::{
        fillers::{FillProvider, JoinFill, WalletFiller},
        Identity, Provider, ProviderBuilder, RootProvider,
    },
    signers::local::PrivateKeySigner,
    transports::BoxTransport,
};
use call::ModuleCall;
use callback::ModuleCallback;
use error::TxSubmitError;
use ibc_eureka_solidity::{ics02::client::clientInstance, ics26::router::routerInstance};
use ibc_eureka_types::msg::IbcEurekaVoyagerMessage;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use tracing::{info, instrument, warn};
use unionlabs::ErrorReporter;
use voyager_message::{
    core::ChainId,
    data::Data,
    module::{PluginInfo, PluginServer},
    run_plugin_server, DefaultCmd, Plugin, VoyagerMessage,
};
use voyager_vm::{pass::PassResult, BoxDynError, Op};

mod call;
mod callback;
mod data;
mod error;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_plugin_server::<Module>().await;
}

/// The configuration for the Ethereum IBC Eureka transaction module
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Chain ID
    pub chain_id: ChainId<'static>,

    /// The address of the `IBCHandler` smart contract.
    pub ics26_router_address: String,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,

    /// The private key for the Ethereum account.
    // TODO: Use a more secure way to store the private key.
    pub private_key: String,

    /// The maximum gas price for any submitted transaction.
    #[serde(default)]
    pub max_gas_price: Option<u128>,
}

/// The Ethereum IBC Eureka transaction module
#[derive(Debug, Clone)]
pub struct Module {
    /// Chain ID
    pub chain_id: ChainId<'static>,

    /// The ics26 router contract instance
    #[allow(clippy::type_complexity)]
    pub ics26_router: routerInstance<
        BoxTransport,
        FillProvider<
            JoinFill<Identity, WalletFiller<EthereumWallet>>,
            RootProvider<BoxTransport>,
            BoxTransport,
            Ethereum,
        >,
    >,

    /// The maximum gas price for any submitted transaction.
    pub max_gas_price: Option<u128>,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let wallet = EthereumWallet::from(
            config
                .private_key
                .strip_prefix("0x")
                .unwrap_or(&config.private_key)
                .parse::<PrivateKeySigner>()?,
        );

        let provider = ProviderBuilder::new()
            .wallet(wallet.clone())
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let raw_chain_id = provider.get_chain_id().await?;
        let chain_id = ChainId::new(raw_chain_id.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        let ics26_router =
            routerInstance::new(Address::from_str(&config.ics26_router_address)?, provider);

        Ok(Self {
            chain_id,
            ics26_router,
            max_gas_price: config.max_gas_price,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: format!(
                r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all transaction data messages
    ($data."@type" == "identified_ibc_message_batch" or $data."@type" == "identified_ibc_message")
        and $data."@value".chain_id == "{chain_id}"
else
    false
end
"#,
                chain_id = config.chain_id,
            ),
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
            ModuleCall::SubmitCall(msg) => {
                self.submit_tx(msg).await.map_err(|err| {
                    ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>)
                })?;

                // TODO: Add error handling/retry logic here
                Ok(Op::Noop)
            }
        }
    }

    async fn run_pass(
        &self,
        _: &Extensions,
        _msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        // TODO: Implement this
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

fn plugin_name(chain_id: &ChainId<'_>) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{chain_id}")
}

impl Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn submit_tx(&self, msg: IbcEurekaVoyagerMessage) -> Result<(), TxSubmitError> {
        match msg {
            IbcEurekaVoyagerMessage::UpdateClient(update_msg) => {
                let ics02_address = self.ics26_router.ICS02_CLIENT().call().await?._0;
                let ics02_client = clientInstance::new(ics02_address, self.ics26_router.provider());

                self.validate_gas().await?;

                // TODO: Add retry logic here, similar to
                // https://github.com/unionlabs/union/blob/18c86b4ff81408d31bec998f5d23bc1b03c9fda3/voyager/plugins/transaction/ethereum/src/main.rs#L351
                let _ = ics02_client
                    .updateClient(update_msg.client_id, update_msg.msg.into())
                    .send()
                    .await?
                    .watch()
                    .await?;

                info!("client updated successfully");

                Ok(())
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn validate_gas(&self) -> Result<(), TxSubmitError> {
        if let Some(max_gas_price) = self.max_gas_price {
            let gas_price = self.ics26_router.provider().get_gas_price().await?;

            if gas_price > max_gas_price {
                warn!(%max_gas_price, %gas_price, "gas price is too high");
                return Err(TxSubmitError::GasPriceTooHigh {
                    max: max_gas_price,
                    price: gas_price,
                });
            }
        }

        Ok(())
    }
}

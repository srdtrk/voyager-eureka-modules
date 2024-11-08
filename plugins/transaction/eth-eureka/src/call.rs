//! Module Calls for Ethereum IBC Eureka transaction module

use ibc_eureka_types::msg::IbcEurekaVoyagerMessage;

/// Module Calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions, clippy::large_enum_variant)]
pub enum ModuleCall {
    SubmitMulticall(Vec<IbcEurekaVoyagerMessage>),
}

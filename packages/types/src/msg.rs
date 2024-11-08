//! Messages for IBC Eureka Voyager Modules

use sp1_ics07_tendermint_solidity::IUpdateClientMsgs::MsgUpdateClient;

/// IBC Eureka Voyager Messages
// TODO: Add more messages
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum IbcEurekaVoyagerMessage {
    /// Update the client
    UpdateClient(MsgUpdateClient),
}

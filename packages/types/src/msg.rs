//! Messages for IBC Eureka Voyager Modules

/// IBC Eureka Voyager Messages
// TODO: Add more messages
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum IbcEurekaVoyagerMessage {
    /// Update the client
    UpdateClient(UpdateClientMsg),
}

/// Update the client message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct UpdateClientMsg {
    /// The client ID
    pub client_id: String,
    /// The update message
    pub msg: Vec<u8>,
}

impl From<UpdateClientMsg> for IbcEurekaVoyagerMessage {
    fn from(msg: UpdateClientMsg) -> Self {
        Self::UpdateClient(msg)
    }
}

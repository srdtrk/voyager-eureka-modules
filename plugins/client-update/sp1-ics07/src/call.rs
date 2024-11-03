//! Module Calls for SP1 ICS07 Light Client Update Plugin

use unionlabs::ibc::core::client::height::Height;

/// Module Calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub enum ModuleCall {
    /// Fetch Update Call
    FetchUpdate(FetchUpdate),
}

/// Fetch Update Call
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FetchUpdate {
    /// Height to update from
    pub update_from: Height,
    /// Height to update to
    pub update_to: Height,
}

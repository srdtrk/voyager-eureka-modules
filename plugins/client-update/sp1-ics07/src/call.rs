//! Module Calls for SP1 ICS07 Light Client Update Plugin

use ibc_client_tendermint_types::{ConsensusState, Header};
use unionlabs::ibc::core::client::height::Height;

/// Module Calls
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions, clippy::large_enum_variant)]
pub enum ModuleCall {
    /// Fetch Update Call
    FetchUpdate(FetchUpdate),
    /// Fetch Prove Request Call
    FetchSP1Proof(FetchSP1Proof),
}

/// Fetch Update Call
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FetchUpdate {
    /// Height to update from
    pub update_from: Height,
    /// Height to update to
    pub update_to: Height,
}

/// Turn the `FetchUpdate` into an SP1 Proof
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FetchSP1Proof {
    /// The trusted consensus state
    pub trusted_consensus_state: ConsensusState,
    /// The proposed header
    pub proposed_header: Header,
}

impl From<FetchUpdate> for ModuleCall {
    fn from(fetch_update: FetchUpdate) -> Self {
        Self::FetchUpdate(fetch_update)
    }
}

impl From<FetchSP1Proof> for ModuleCall {
    fn from(fetch_prove_request: FetchSP1Proof) -> Self {
        Self::FetchSP1Proof(fetch_prove_request)
    }
}

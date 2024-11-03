//! Data structures returned from this plugin

use ibc_client_tendermint_types::{ConsensusState, Header};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions, clippy::large_enum_variant)]
pub enum ModuleData {
    ProveResponse(ProveResponse),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ProveResponse {
    /// The trusted consensus state
    pub trusted_consensus_state: ConsensusState,
    /// The proposed header
    pub proposed_header: Header,
    /// Encoded [`sp1_ics07_tendermint_solidity::ISP1Msgs::SP1Proof`] proof
    pub sp1_proof: Vec<u8>,
}

impl From<ProveResponse> for ModuleData {
    fn from(prove_response: ProveResponse) -> Self {
        Self::ProveResponse(prove_response)
    }
}

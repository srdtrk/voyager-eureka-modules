//! Height extension for voyager

use sp1_ics07_tendermint_solidity::IICS02ClientMsgs::Height as SP1ICS07Height;
use unionlabs::ibc::core::client::height::Height;

/// Trait for converting to [`Height`]
#[allow(clippy::module_name_repetitions)]
pub trait IntoUnionHeight {
    /// Converts the type into a [`Height`]
    fn into_unionlabs_height(self) -> Height;
}

impl IntoUnionHeight for SP1ICS07Height {
    fn into_unionlabs_height(self) -> Height {
        Height {
            revision_number: self.revisionNumber.into(),
            revision_height: self.revisionHeight.into(),
        }
    }
}

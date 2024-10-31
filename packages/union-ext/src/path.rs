//! Path Extensions for IBC Eureka

use std::fmt::Display;

use unionlabs::{ethereum::keccak256, ics24::Path};

/// The IBC Eureka Path Extension trait
pub trait IbcEurekaPathExt: Display {
    /// Converts the path to a storage key.
    /// All Eureka paths are 32 bytes long.
    fn to_storage_key(&self) -> [u8; 32];
}

impl IbcEurekaPathExt for Path {
    fn to_storage_key(&self) -> [u8; 32] {
        let key_str = format!("{self}");
        keccak256(key_str.as_bytes()).into()
    }
}

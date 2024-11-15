//! Defines [`TxSubmitError`].

use alloy::{
    contract::Error,
    providers::PendingTransactionError,
    transports::{RpcError, TransportErrorKind},
};
use ibc_eureka_types::msg::IbcEurekaVoyagerMessage;

/// Errors that can occur when submitting a transaction.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::module_name_repetitions, dead_code)]
pub enum TxSubmitError {
    #[error(transparent)]
    Error(#[from] Error),
    #[error("error waiting for transaction")]
    PendingTransactionError(#[from] PendingTransactionError),
    #[error("out of gas")]
    OutOfGas,
    #[error("0x revert")]
    EmptyRevert(Vec<IbcEurekaVoyagerMessage>),
    #[error("gas price is too high: max {max}, price {price}")]
    GasPriceTooHigh { max: u128, price: u128 },
    #[error("eth rpc call failed: {0}")]
    EthRpcError(#[from] RpcError<TransportErrorKind>),
}

// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use seanerbus_schemas::message::SchemaError;

/// Error type for seanerbus_async operations
#[derive(thiserror::Error, Debug)]
pub enum ConnectionError {
    /// The underlying stream encountered an Error
    #[error("Error from underlying stream: {0}")]
    StreamError(#[from] std::io::Error),
    /// Encoding or decoding the message as a SeanerBUS message failed
    #[error("Error from Cap'n'Proto: {0}")]
    CapnpError(#[from] SchemaError),
    /// Any other error.
    #[error("Error: {0}")]
    GeneralError(String),
}

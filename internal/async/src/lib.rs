// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

//! # Shared SeanerBUS async Server/Client Code
//!
//! This crate contains a [`MessageReader`](seanerbus_async/reader/struct.MessageReader.html) and
//! [`MessageWriter`](seanerbus_async/writer/struct.MessageWriter.html) that wrap around an
//! [`AsyncRead`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html)/[`AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html)-able
//! stream and provide the ability to directly read whole SeanerBUS messages.

/// A module containing the seanerbus_async Error type.
pub mod error;
/// A module containing the SeanerBUS MessageReader.
pub mod reader;
/// A module containing the SeanerBUS MessageWriter.
pub mod writer;

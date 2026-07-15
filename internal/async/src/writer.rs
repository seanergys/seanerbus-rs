// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use crate::error::ConnectionError;
use log::trace;
use seanerbus_schemas::message::Message;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

/// A SeanerBUS message writer
pub struct MessageWriter {
    writer: Box<dyn AsyncWrite + Unpin + Send + 'static>,
}
impl MessageWriter {
    /// Creates a new `MessageWriter`. `writer` must implement
    /// [`AsyncWrite`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html)
    pub fn new<Writer>(writer: Writer) -> MessageWriter
    where
        Writer: AsyncWrite + Unpin + Send + 'static,
    {
        MessageWriter {
            writer: Box::new(writer),
        }
    }

    /// Writes a `Message` to the underlying stream.
    ///
    /// Returns `()` if the `Message` could be successfully send.
    ///
    /// Returns [`ConnectionError`](seanerbus_async/error/enum.ConnectionError.html) if sending the
    /// `Message` failed for some reason.
    pub async fn write_message(&mut self, msg: &Message) -> Result<(), ConnectionError> {
        let ser_msg = msg.to_capnp()?;
        trace!("serialized message, serialized size={}", ser_msg.len());
        self.writer.write_all(&ser_msg).await?;
        trace!("Wrote message to network!");
        Ok(())
    }
}

// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use crate::error::ConnectionError;
use bytes::{Buf, BytesMut};
use log::trace;
use seanerbus_schemas::message::Message;
use tokio::io::{AsyncRead, AsyncReadExt};

/// Represents a message reader
pub struct MessageReader {
    reader: Box<dyn AsyncRead + Unpin + std::marker::Send + 'static>,
    buffer: BytesMut,
}

impl MessageReader {
    /// Creates a new SeanerBUS message reader from the given reader,
    /// which must implement AsyncRead.
    pub fn new<Reader>(reader: Reader) -> MessageReader
    where
        Reader: AsyncRead + Unpin + Send + 'static,
    {
        MessageReader {
            reader: Box::new(reader),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// Tries to read a SeanerBUS `Message` from the connection.
    ///
    /// Returns a `Result` containing the read message or an Error
    /// Returns:
    /// * [`ConnectionError`](seanerbus_async/error/enum.ConnectionError.html) if the Connection exploded.
    /// * `None` if the end of the stream was reached.
    /// * `Some(Message)` If a message could be successfully read.
    pub async fn read_message(&mut self) -> Result<Option<Message>, ConnectionError> {
        loop {
            if let Some((msg, size)) = Message::from_capnp(&self.buffer) {
                trace!(
                    "Successfully parsed message from network, msg_type={:?}, size={}",
                    msg.dp_type, size
                );
                self.buffer.advance(size);
                trace!(
                    "Advanced buffer by {}, buffer size is now {}",
                    size,
                    self.buffer.len()
                );
                return Ok(Some(msg));
            }
            let n = self.reader.read_buf(&mut self.buffer).await?;

            trace!("Read {} bytes, buffer size is now {}", n, self.buffer.len());

            if 0 == n {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ConnectionError::GeneralError(format!(
                        "connection reset by peer, but there are still {} bytes left in the buffer",
                        self.buffer.len()
                    )));
                }
            }
        }
    }
}

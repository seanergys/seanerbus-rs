// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

//! # tokio async based SeanerBUS client library
//! ## Example
//!
//! This subscribes to a topic with Entity `0x0000 0000 0000 000f0`
//!
//!
//! ```ignore
//! let mut conn = Connection::new("localhost:5398").await?;
//!
//!    conn.subscribe(seanerbus_schemas::message::Entity(Uuid::from_u64_pair(
//!        0, 64,
//!    )))
//!    .await?;
//!    while let Some(msg) = conn.read_msg().await? {
//!        match Message::from_capnp(msg.payload_type, &msg.payload)? {
//!            Message::TimeValueV1(t, v) => {
//!                println!("time={}, value{}", t, v);
//!            },
//!            _ => {
//!                panic!("Waited for TimeValue pairs, got: {} instead.", msg.payload_type);
//!            }
//!        }
//!    }
//! ```

use seanerbus_async::reader::MessageReader;
use seanerbus_async::writer::MessageWriter;
use seanerbus_schemas::error_payload::{ErrorPayload, ErrorPayloadError};
use seanerbus_schemas::message::{Entity, Message, MessageType};
use thiserror::Error;
use tokio::net::TcpStream;

/// A connection to the SeanerBUS
pub struct Connection {
    reader: MessageReader,
    writer: MessageWriter,
}
/// Type for SeanerBUS client errors.
#[derive(Error, Debug)]
pub enum ConnectionError {
    /// Underlying network connection failed.
    #[error("Underlying network failed: {0}")]
    IoError(#[from] std::io::Error),
    /// Reading a `Message` from the network using seanerbus_async failed.
    #[error("Underlying network failed: {0}")]
    ConnError(#[from] seanerbus_async::error::ConnectionError),
    /// Connection was closed during the execution of a request.
    #[error("Connection closed while requesting!")]
    ConnClosedError,
    /// The Broker reported an Error.
    #[error("Communication with broker failed: {0}")]
    ProtocolError(String),
    #[error(
        "Communication with broker failed, but decoding error message from broker also failed: {0}"
    )]
    /// Decoding the Payload of the Message failed.
    ErrorPayloadDecodeError(#[from] ErrorPayloadError),
}

impl Connection {
    /// Connects to the broker given by the URL `endpoint` (e.g. "localhost:5398")
    ///
    /// Returns a `Connection` if the connect succeeded.
    ///
    /// Returns a [`ConnectionError`](enum.ConnectionError.html) if an error occurred.
    pub async fn new(endpoint: &str) -> Result<Connection, ConnectionError> {
        let stream = TcpStream::connect(endpoint).await?;

        let (reader, writer) = stream.into_split();
        Ok(Connection {
            reader: MessageReader::new(reader),
            writer: MessageWriter::new(writer),
        })
    }

    /// Subscribes to topic given by `entity`, which is a UUIDv4.
    ///
    /// Returns a [`ConnectionError`](enum.ConnectionError.html) if sending the subscribe message
    /// failed.
    pub async fn subscribe(&mut self, entity: Entity) -> Result<(), ConnectionError> {
        Ok(self
            .writer
            .write_message(&Message::subscribe_to_msg(entity))
            .await?)
    }

    /// Reads a `Message` from the `Connection`
    ///
    /// Returns [`ConnectionError`](enum.ConnectionError.html) if reading the connection failed.
    ///
    /// Returns `Some(Message)` if a Message could be received
    ///
    /// Returns `None` if the Connection was closed.
    pub async fn read_msg(&mut self) -> Result<Option<Message>, ConnectionError> {
        Ok(self.reader.read_message().await?)
    }

    /// Publishes a `Message` to the UUIDv4 `entity` with the given payload and payload type.
    ///
    /// Returns [`ConnectionError`](enum.ConnectionError.html) if publishing the message failed.
    pub async fn publish(
        &mut self,
        entity: Entity,
        payload_type: u64,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        Ok(self
            .writer
            .write_message(&Message::publish_msg(entity, payload_type, payload))
            .await?)
    }

    /// Posts a request to the given Entity with the given payload_type and payload.
    ///
    /// Returns either a [`ConnectionError`](enum.ConnectionError.html) if the request failed or the
    /// `Message` gotten as a result.
    pub async fn request(
        &mut self,
        entity: Entity,
        payload_type: u64,
        payload: &[u8],
    ) -> Result<Message, ConnectionError> {
        self.writer
            .write_message(&Message::request_msg(entity, payload_type, payload))
            .await?;

        let msg = match self.reader.read_message().await? {
            Some(msg) => msg,
            None => {
                return Err(ConnectionError::ConnClosedError);
            }
        };

        match msg.dp_type {
            MessageType::Response => Ok(msg),
            MessageType::Error => {
                let err = ErrorPayload::from_capnp(&msg.payload)?;

                Err(ConnectionError::ProtocolError(err.error_str))
            }
            _ => {
                panic!(
                    "Got {:?} SEANERBUS message when error or request expected!",
                    msg.dp_type
                );
            }
        }
    }

    /// Responds to an incoming request with the given payload and payload_type
    pub async fn respond_to(
        &mut self,
        responding_to: &Message,
        payload_type: u64,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        Ok(self
            .writer
            .write_message(&Message::response_msg(responding_to, payload_type, payload))
            .await?)
    }

    /// Registers a topic for request/response.
    ///
    /// After the register Message, incoming requests can be read with
    /// [`read_msg`](struct.Connection.html#method.read_msg)
    pub async fn register(&mut self, entity: Entity) -> Result<(), ConnectionError> {
        Ok(self
            .writer
            .write_message(&Message::register_msg(entity))
            .await?)
    }
}

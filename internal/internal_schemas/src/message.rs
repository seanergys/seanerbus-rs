// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use crate::error_payload::ErrorPayload;
pub use crate::internal_capnp;
use thiserror::Error;
use uuid::Uuid;
use rand::RngExt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageType {
    Subscribe,
    Publish,
    Error,
    Request,
    Response,
    Register,
}

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Could not serialize Cap'n'Proto message: {0}")]
    CapnpError(#[from] capnp::Error),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ConnectionId(pub Uuid);

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "conn({})", self.0)
    }
}

impl From<crate::internal_capnp::message::MessageType> for MessageType {
    fn from(item: internal_capnp::message::MessageType) -> Self {
        match item {
            internal_capnp::message::MessageType::Subscribe => MessageType::Subscribe,
            internal_capnp::message::MessageType::Publish => MessageType::Publish,
            internal_capnp::message::MessageType::Error => MessageType::Error,
            internal_capnp::message::MessageType::Request => MessageType::Request,
            internal_capnp::message::MessageType::Response => MessageType::Response,
            internal_capnp::message::MessageType::Register => MessageType::Register,
        }
    }
}

impl From<MessageType> for internal_capnp::message::MessageType {
    fn from(item: MessageType) -> Self {
        match item {
            MessageType::Subscribe => internal_capnp::message::MessageType::Subscribe,
            MessageType::Publish => internal_capnp::message::MessageType::Publish,
            MessageType::Error => internal_capnp::message::MessageType::Error,
            MessageType::Response => internal_capnp::message::MessageType::Response,
            MessageType::Request => internal_capnp::message::MessageType::Request,
            MessageType::Register => internal_capnp::message::MessageType::Register,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct Address(pub Uuid);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "addr({})", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub dp_type: MessageType,
    pub to: Option<ConnectionId>,
    pub from: Option<ConnectionId>,
    pub address: Address,
    pub payload: Vec<u8>,
    pub payload_type: u64,
    pub matching_id: u64
}

impl Message {
    /// Creates a Message from the given Cap'n'Proto buffer.
    ///
    /// * `data`: Buffer with Cap'n'Proto
    ///
    /// Returns Some(Message) if the data is parsable as a Message and None if not.
    pub fn from_capnp(data: &[u8]) -> Option<(Message, usize)> {
        let reader =
            match capnp::serialize::read_message(data, capnp::message::ReaderOptions::new()) {
                Ok(r) => r,
                Err(_) => return None,
            };

        let msg = match reader.get_root::<internal_capnp::message::Reader>() {
            Ok(msg) => msg,
            Err(_) => return None,
        };

        let payload = msg.get_payload().unwrap();
        let payload_type = msg.get_payload_type();
        let address = msg.get_address().unwrap();
        let from = match msg.has_from() {
            true => {
                let from = msg.get_from().unwrap();
                Some(ConnectionId(Uuid::from_u64_pair(
                    from.get_upper(),
                    from.get_lower(),
                )))
            }
            false => None,
        };
        let to = match msg.has_to() {
            true => {
                let to = msg.get_to().unwrap();
                Some(ConnectionId(Uuid::from_u64_pair(
                    to.get_upper(),
                    to.get_lower(),
                )))
            }
            false => None,
        };
        let matching_id = msg.get_matching_id();

        let dp_type = MessageType::from(msg.get_type().unwrap());
        Some((
            Message {
                dp_type,
                payload_type,
                payload: payload.to_vec(),
                address: Address(Uuid::from_u64_pair(
                    address.get_upper(),
                    address.get_lower(),
                )),
                to,
                from,
                matching_id
            },
            (reader.size_in_words() + 1) * 8,
        )) // This is always short one word, don't ask me why.
    }

    /// Serializes the Message to the Cap'n'Proto format.
    ///@0xccea65a7e11c9c74
    /// Returns the serialized message or an Error if no Serialization could be performed.
    pub fn to_capnp(&self) -> Result<Vec<u8>, SchemaError> {
        let mut bld = capnp::message::TypedBuilder::<internal_capnp::message::Owned>::new_default();
        let mut msg = bld.init_root();

        msg.set_payload(&self.payload);
        msg.set_payload_type(self.payload_type);
        msg.set_type(self.dp_type.clone().into());
        msg.set_matching_id(self.matching_id);
        let mut uuid_builder = msg.reborrow().init_address();

        let uuid_pair = self.address.0.as_u64_pair();
        uuid_builder.set_upper(uuid_pair.0);
        uuid_builder.set_lower(uuid_pair.1);
        if let Some(id) = self.to {
            let mut uuid_builder = msg.reborrow().init_to();

            let uuid_pair = id.0.as_u64_pair();
            uuid_builder.set_upper(uuid_pair.0);
            uuid_builder.set_lower(uuid_pair.1);
        }
        if let Some(id) = self.from {
            let mut uuid_builder = msg.reborrow().init_from();

            let uuid_pair = id.0.as_u64_pair();
            uuid_builder.set_upper(uuid_pair.0);
            uuid_builder.set_lower(uuid_pair.1);
        }
        let mut buffer = vec![];
        capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

        Ok(buffer)
    }

    pub fn create_error(
        to: ConnectionId,
        addr: Address,
        error_kind: internal_capnp::error_payload::ErrorKind,
        error_str: &str,
    ) -> Message {
        Message {
            dp_type: MessageType::Error,
            to: Some(to),
            from: None,
            address: addr,
            payload_type: 0,
            payload: ErrorPayload {
                error_kind,
                error_str: error_str.to_string(),
            }
            .to_capnp()
            .unwrap(),
            matching_id: 0
        }
    }

    pub fn subscribe_to_msg(addr: Address) -> Message {
        Message {
            dp_type: MessageType::Subscribe,
            to: None,
            from: None,
            address: addr,
            payload: vec![],
            payload_type: 0,
            matching_id: 0
        }
    }

    pub fn publish_msg(addr: Address, payload_type: u64, payload: &[u8]) -> Message {
        Message {
            dp_type: MessageType::Publish,
            to: None,
            from: None,
            address: addr,
            payload: payload.to_vec(),
            payload_type,
            matching_id: 0
        }
    }

    pub fn request_msg(addr: Address, payload_type: u64, payload: &[u8]) -> Message {
        let mut rng = rand::rng();
        Message {
            dp_type: MessageType::Request,
            to: None,
            from: None,
            address: addr,
            payload: payload.to_vec(),
            payload_type,
            matching_id: rng.random()
        }
    }

    pub fn response_msg(responds_to: &Message, payload_type: u64, payload: &[u8]) -> Message {
        Message {
            dp_type: MessageType::Response,
            to: responds_to.from,
            from: None,
            address: responds_to.address,
            payload: payload.to_vec(),
            payload_type,
            matching_id: responds_to.matching_id
        }
    }

    pub fn register_msg(addr: Address) -> Message {
        Message {
            dp_type: MessageType::Register,
            to: None,
            from: None,
            address: addr,
            payload: vec![],
            payload_type: 0,
            matching_id: 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let str = "foo";
        let msg1 = Message {
            dp_type: MessageType::Error,
            to: Some(ConnectionId(Uuid::new_v4())),
            from: Some(ConnectionId(Uuid::new_v4())),
            address: Address(Uuid::new_v4()),
            payload: str.as_bytes().to_vec(),
            payload_type: 42,
            matching_id: 69
        };

        let foo = msg1.to_capnp().expect("Can serialize message");
        let msg2 = Message::from_capnp(&foo).expect("Can deserialize message");

        assert_eq!(msg1, msg2.0);
    }

    #[test]
    fn test_random_bytes() {
        let str = "This is certainly not a Cap'n'Proto message";

        assert_eq!(Message::from_capnp(&str.as_bytes()), None);
    }

    #[test]
    fn test_create_empty_addrs() {
        let str = "foo";
        let msg1 = Message {
            dp_type: MessageType::Error,
            to: None,
            from: None,
            address: Address(Uuid::new_v4()),
            payload: str.as_bytes().to_vec(),
            payload_type: 42,
            matching_id: 0
        };

        let foo = msg1.to_capnp().expect("Can serialize message");
        let msg2 = Message::from_capnp(&foo).expect("Can deserialize message");

        assert_eq!(msg1, msg2.0);
    }

    #[test]
    fn can_create_error() {
        let msg_err = Message::create_error(
            ConnectionId(Uuid::new_v4()),
            Address(Uuid::new_v4()),
            internal_capnp::error_payload::ErrorKind::NoSuchInterface,
            "not an interface",
        );
        let foo = msg_err.to_capnp().expect("Can serialize Error message");
        let msg_err2 = Message::from_capnp(&foo).expect("Can deserialize Error message");

        assert_eq!(msg_err, msg_err2.0);

        assert_eq!(msg_err2.0.dp_type, MessageType::Error);

        let err = ErrorPayload::from_capnp(&msg_err2.0.payload).expect("Can decode error");

        assert_eq!(
            err.error_kind,
            internal_capnp::error_payload::ErrorKind::NoSuchInterface
        );
        assert_eq!(err.error_str, "not an interface");
    }
}

// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use crate::internal_capnp;
use thiserror::Error;

#[derive(PartialEq, Eq, Debug)]
pub struct ErrorPayload {
    pub error_kind: internal_capnp::error_payload::ErrorKind,
    pub error_str: String,
}

#[derive(Error, Debug)]
pub enum ErrorPayloadError {
    #[error("Could not deserialize error payload: {0}!")]
    CapnpError(#[from] capnp::Error),
}

impl ErrorPayload {
    pub fn new(kind: internal_capnp::error_payload::ErrorKind, str: String) -> ErrorPayload {
        ErrorPayload {
            error_kind: kind,
            error_str: str,
        }
    }
    pub fn from_capnp(data: &[u8]) -> Result<ErrorPayload, ErrorPayloadError> {
        let reader = capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

        let msg = reader.get_root::<internal_capnp::error_payload::Reader>()?;

        Ok(ErrorPayload {
            error_kind: msg.get_error_kind().unwrap(),
            error_str: msg.get_error_str().unwrap().to_str().unwrap().to_string(),
        })
    }

    pub fn to_capnp(&self) -> Result<Vec<u8>, capnp::Error> {
        let mut bld =
            capnp::message::TypedBuilder::<internal_capnp::error_payload::Owned>::new_default();

        let mut msg = bld.init_root();

        msg.set_error_kind(self.error_kind);
        msg.set_error_str(self.error_str.clone());

        let mut buffer = vec![];
        capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

        Ok(buffer)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let err1 = ErrorPayload::new(
            internal_capnp::error_payload::ErrorKind::NoSuchInterface,
            "no such interface!".to_string(),
        );

        let foo = err1.to_capnp().expect("Can serialize Error Payload");
        let err2 = ErrorPayload::from_capnp(&foo).expect("Can deserialize Error Payload");

        assert_eq!(err1, err2);
    }

    #[test]

    fn test_random_bytes() {
        let foo = "I am certainly not a Capnp message";

        assert!(matches!(ErrorPayload::from_capnp(foo.as_bytes()), Err(_)));
    }
}

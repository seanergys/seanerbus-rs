// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use seanerbus_client::Connection;
use seanerbus_msg::message::Message;
use seanerbus_schemas::message::{Address, MessageType};
use uuid::Uuid;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("resolve failed!")]
    ResolveFailure,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("USAGE: ./request_test UUID");
        return Ok(());
    }

    let mut conn = Connection::new("localhost:5398").await?;

    let (payload_type, payload) = Message::HelloWorldReqV1("Hello World".to_owned())
        .to_capnp()
        .unwrap();

    let msg = conn
        .request(
            Address(Uuid::parse_str(&args[1]).unwrap()),
            payload_type,
            &payload,
        )
        .await?;

    if msg.dp_type == MessageType::Response {
        let payload = Message::from_capnp(msg.payload_type, &msg.payload).unwrap();
        if let Message::HelloWorldResV1(msg) = payload {
            println!("Got an answer: {}", msg);
        } else {
            panic!(
                "Got something else than a Message::TimeValueV1 as a response: {:?}",
                payload
            );
        }
    } else {
        panic!("Expected Response to Request, got {:?}", msg.dp_type);
    }
    Ok(())
}

// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use seanerbus_client::Connection;
use seanerbus_msg::message::Message;
use uuid::Uuid;

/// Subscribes to the topic given on the command line,
/// printing the time and value field of the message
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("USAGE: ./subscribe_test UUID");
        return Ok(());
    }

    let mut conn = Connection::new("localhost:5398").await?;

    conn.subscribe(seanerbus_schemas::message::Address(
        Uuid::parse_str(&args[1]).unwrap(),
    ))
    .await?;
    while let Some(msg) = conn.read_msg().await? {
        match Message::from_capnp(msg.payload_type, &msg.payload)? {
            Message::TimeValueV1(t, v) => {
                println!("time={}, value{}", t, v);
            }
            _ => {
                panic!(
                    "Waited for TimeValue pairs, got: {} instead.",
                    msg.payload_type
                );
            }
        }
    }

    Ok(())
}

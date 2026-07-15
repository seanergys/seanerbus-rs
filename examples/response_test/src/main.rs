// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use seanerbus_client::Connection;
use seanerbus_schemas::message::{Address, MessageType};
use uuid::Uuid;

//Listens on the topic given on the command line for incoming requests.
//
//If the requests is a TimeValueV1, it responds with a TimeValueV1
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("USAGE: ./response_test UUID");
        return Ok(());
    }

    let mut conn = Connection::new("localhost:5398").await?;

    conn.register(Address(Uuid::parse_str(&args[1]).unwrap()))
        .await?;

    while let Some(msg) = conn.read_msg().await? {
        if msg.dp_type == MessageType::Request {
            let payload_msg =
                seanerbus_msg::message::Message::from_capnp(msg.payload_type, &msg.payload)?;

            if let seanerbus_msg::message::Message::HelloWorldReqV1(str_msg) = payload_msg {
                println!("Got a HelloWorldV1Req request, msg: {}", str_msg);

                let (payload_type, payload) = seanerbus_msg::message::Message::HelloWorldResV1(
                    format!("Appreciate your \"{}\"", str_msg),
                )
                .to_capnp()
                .unwrap();
                conn.respond_to(&msg, payload_type, &payload).await?;
            } else {
                panic!("Expected HelloWorldV1Req from request, got something else instead!");
            }
        } else {
            panic!("Expected Request, got {:?}", msg.dp_type);
        }
    }
    Ok(())
}

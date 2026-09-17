// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use seanerbus_client::Connection;
use uuid::Uuid;

pub async fn resolve(
    conn: &mut Connection,
    name: Vec<(String, String)>,
) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
    let (payload_type, payload) = seanerbus_msg::message::Message::ResolveReqV1(name)
        .to_capnp()
        .unwrap();

    let msg = conn
        .request(
            seanerbus_schemas::message::Entity(Uuid::from_u128(0x10CA105)),
            payload_type,
            &payload,
        )
        .await?;

    if msg.dp_type != seanerbus_schemas::message::MessageType::Response {
        panic!("Expected Response to Request, got {:?}", msg.dp_type);
    }

    let payload =
        seanerbus_msg::message::Message::from_capnp(msg.payload_type, &msg.payload).unwrap();
    if let seanerbus_msg::message::Message::ResolveResV1(entitiy_vec) = payload {
        Ok(entitiy_vec)
    } else {
        Err("Entity resolution failed!".into())
    }
}

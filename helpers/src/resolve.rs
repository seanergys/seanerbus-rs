// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT


use uuid::Uuid;

pub async fn resolve(name: Vec<(String, String)> ) -> Result<Vec<Uuid>, Box< dyn std::error::Error>>{
    let mut conn = seanerbus_client::Connection::new("localhost:5398").await?;

    let (payload_type, payload) = seanerbus_msg::message::Message::ResolveReqV1(name)
        .to_capnp()
        .unwrap();


    let msg = conn
        .request(
            seanerbus_schemas::message::Address(Uuid::from_u128(0x10CA105)),
            payload_type,
            &payload,
        )
        .await?;

    if msg.dp_type != seanerbus_schemas::message::MessageType::Response {
        panic!("Expected Response to Request, got {:?}", msg.dp_type);
    }

    let payload = seanerbus_msg::message::Message::from_capnp(msg.payload_type, &msg.payload).unwrap();
    if let seanerbus_msg::message::Message::ResolveResV1(addr_vec) = payload {
        return Ok(addr_vec)
    } else {
        return Err("Address resolution failed!".into())
    }
}

use uuid::Uuid;


async fn resolve(name: Vec<(String, String)> ) -> Result<Vec<Uuid>, Box< dyn std::error::Error>>{
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

pub async fn get_timeline(metric_uuid: Uuid, start_time: i64, end_time: i64, interval_max: i64, history_type: seanerbus_msg::msg_capnp::HistoryType) -> Result<seanerbus_msg::message::History, Box< dyn std::error::Error>> {
    let name: Vec<(String, String)> = Vec::from([("service".to_owned(), "time_series_db".to_owned())]);

    let db_addrs = resolve(name).await?;

    if db_addrs.len() == 0 {
        return Err("DB Address not found!".into())

    }

    let db_addr = db_addrs[0];

    let mut conn = seanerbus_client::Connection::new("localhost:5398").await?;

    let (payload_type, payload) = seanerbus_msg::message::Message::HistoryReqV1(
        metric_uuid, start_time, end_time, interval_max, history_type
    )
    .to_capnp()
    .unwrap();

    let msg = conn
        .request(
            seanerbus_schemas::message::Address(db_addr),
            payload_type,
            &payload,
        )
        .await?;

    if msg.dp_type != seanerbus_schemas::message::MessageType::Response {
        panic!("Expected Response to Request, got {:?}", msg.dp_type);
    }

    let payload = seanerbus_msg::message::Message::from_capnp(msg.payload_type, &msg.payload).unwrap();
    if let seanerbus_msg::message::Message::HistoryResV1(response) = payload {
        match response {
            Ok(history) => Ok(history),
            Err(string) => Err(string.into()),
        }
    } else {
        return Err("Get timeline reequest failed on response!".into())
    }
}
// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use uuid::Uuid;

pub async fn get_timeline(
    conn: &mut seanerbus_client::Connection,
    metric_uuid: Uuid,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
    interval_max: i64,
    history_type: seanerbus_msg::msg_capnp::HistoryType,
) -> Result<seanerbus_msg::message::History, Box<dyn std::error::Error>> {
    let name: Vec<(String, String)> =
        Vec::from([("service".to_owned(), "time_series_db".to_owned())]);

    let db_entities = crate::resolve::resolve(conn, name).await?;

    if db_entities.is_empty() {
        return Err("DB Entity not found!".into());
    }

    let db_entity = db_entities[0];
    let (payload_type, payload) = seanerbus_msg::message::Message::HistoryReqV1(
        metric_uuid,
        start_time,
        end_time,
        interval_max,
        history_type,
    )
    .to_capnp()
    .unwrap();

    let msg = conn
        .request(
            seanerbus_schemas::message::Entity(db_entity),
            payload_type,
            &payload,
        )
        .await?;

    if msg.dp_type != seanerbus_schemas::message::MessageType::Response {
        panic!("Expected Response to Request, got {:?}", msg.dp_type);
    }

    let payload =
        seanerbus_msg::message::Message::from_capnp(msg.payload_type, &msg.payload).unwrap();
    match payload {
        seanerbus_msg::message::Message::HistoryResV1(history) => Ok(history),
        seanerbus_msg::message::Message::ErrorResV1(_, what) => Err(what.into()),
        _ => Err("Unknown message type!".into()),
    }
}

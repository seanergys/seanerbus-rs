// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use thiserror::Error;
use uuid::Uuid;

use crate::msg_capnp::MessageTypes;
use crate::msg_capnp::error_res_v1;
use crate::msg_capnp::flux_lifecycle_event_v1;
use crate::msg_capnp::get_req_v1;
use crate::msg_capnp::get_res_v1;
use crate::msg_capnp::hello_world_req_v1;
use crate::msg_capnp::history;
use crate::msg_capnp::history_req_v1;
use crate::msg_capnp::history_res_v1;
use crate::msg_capnp::register_resolve_req_v1;
use crate::msg_capnp::resolve_req_v1;
use crate::msg_capnp::resolve_res_v1;
use crate::msg_capnp::reverse_resolve_req_v1;
use crate::msg_capnp::reverse_resolve_res_v1;
use crate::msg_capnp::set_req_v1;
use crate::msg_capnp::time_value_v1;
use crate::msg_capnp::unregister_resolve_req_v1;
use crate::msg_capnp::vector_req_v1;
use crate::msg_capnp::vector_res_v1;

#[derive(Clone, Debug, PartialEq)]
pub struct Aggregate {
    pub minimum: f64,
    pub maximum: f64,
    pub sum: f64,
    pub count: u64,
    pub integral: f64,
    pub active_time: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HistoryValues {
    Raw(Vec<f64>),
    Aggregate(Vec<Aggregate>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct History {
    pub time_delta: Vec<i64>,
    pub values: HistoryValues,
}

/// FOOBAR
#[derive(Debug)]
pub enum Message {
    TimeValueV1(chrono::DateTime<chrono::Utc>, f64),
    ResolveReqV1(Vec<(String, String)>),
    ResolveResV1(Vec<Uuid>),
    HelloWorldReqV1(String),
    HelloWorldResV1(String),
    VectorReqV1(Vec<f64>),
    VectorResV1(Vec<f64>),
    ReverseResolveReqV1(Uuid),
    ReverseResolveResV1(Vec<(String, String)>),
    FluxLifecycleEventV1(String),
    RegisterResolveReqV1(Vec<(String, String)>, Uuid),
    UnregisterResolveReqV1(Uuid),
    HistoryReqV1(
        Uuid,
        chrono::DateTime<chrono::Utc>,
        chrono::DateTime<chrono::Utc>,
        i64,
        crate::msg_capnp::HistoryType,
    ),
    HistoryResV1(History),
    ErrorResV1(u64, String),
    GetReqV1(),
    GetResV1(u64, chrono::DateTime<chrono::Utc>),
    SetReqV1(u64),
    UserDefined(u64, Vec<u8>),
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Deserialization failed: {0}")]
    CapnpError(#[from] capnp::Error),
    #[error("Got message of unknown payload type: {0}")]
    PayloadError(u64),
    #[error("Can not deserialize message: {0}")]
    ParseError(String),
}

impl Message {
    /// Creates a Message from the given Cap'n'Proto buffer.
    ///
    /// * `data`: Buffer with Cap'n'Proto
    ///
    /// Returns Some(Message) if the data is parsable as a Message and None if not.
    pub fn from_capnp(payload_type: u64, data: &[u8]) -> Result<Message, MessageError> {
        if payload_type > 1 << 63 {
            Ok(Message::UserDefined(payload_type, data.to_vec()))
        } else {
            match MessageTypes::try_from(payload_type as u16).unwrap() {
                MessageTypes::TimeValueV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

                    let reader = reader.get_root::<time_value_v1::Reader>()?;

                    Ok(Message::TimeValueV1(
                        chrono::DateTime::from_timestamp_nanos(reader.get_time() as i64),
                        reader.get_value(),
                    ))
                }
                MessageTypes::ResolveReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<resolve_req_v1::Reader>()?;

                    let mut attrs: Vec<(String, String)> = vec![];
                    for kv in reader.get_name()? {
                        attrs.push((
                            kv.get_key()?.to_string().unwrap(),
                            kv.get_value()?.to_string().unwrap(),
                        ));
                    }

                    Ok(Message::ResolveReqV1(attrs))
                }
                MessageTypes::ResolveResV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<resolve_res_v1::Reader>()?;

                    let mut attrs: Vec<Uuid> = vec![];
                    for entity in reader.get_entities()? {
                        attrs.push(Uuid::from_u64_pair(entity.get_upper(), entity.get_lower()));
                    }

                    Ok(Message::ResolveResV1(attrs))
                }

                MessageTypes::HelloWorldReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<hello_world_req_v1::Reader>()?;

                    Ok(Message::HelloWorldReqV1(
                        reader.get_msg().unwrap().to_str().unwrap().to_owned(),
                    ))
                }
                MessageTypes::HelloWorldResV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<hello_world_req_v1::Reader>()?;

                    Ok(Message::HelloWorldResV1(
                        reader.get_msg().unwrap().to_str().unwrap().to_owned(),
                    ))
                }

                MessageTypes::VectorReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<vector_req_v1::Reader>()?;

                    Ok(Message::VectorReqV1(
                        reader.get_values().unwrap().iter().collect(),
                    ))
                }

                MessageTypes::VectorResV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<vector_res_v1::Reader>()?;

                    Ok(Message::VectorResV1(
                        reader.get_results().unwrap().iter().collect(),
                    ))
                }
                MessageTypes::ReverseResolveReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<reverse_resolve_req_v1::Reader>()?;

                    Ok(Message::ReverseResolveReqV1(Uuid::from_u64_pair(
                        reader.get_entity().unwrap().get_upper(),
                        reader.get_entity().unwrap().get_lower(),
                    )))
                }
                MessageTypes::ReverseResolveResV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<reverse_resolve_res_v1::Reader>()?;

                    let mut attrs: Vec<(String, String)> = vec![];
                    for kv in reader.get_name()? {
                        attrs.push((
                            kv.get_key()?.to_string().unwrap(),
                            kv.get_value()?.to_string().unwrap(),
                        ));
                    }

                    Ok(Message::ResolveReqV1(attrs))
                }
                MessageTypes::FluxLifecycleEventV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<flux_lifecycle_event_v1::Reader>()?;

                    Ok(Message::FluxLifecycleEventV1(
                        reader.get_data().unwrap().to_str().unwrap().to_owned(),
                    ))
                }
                MessageTypes::RegisterResolveReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<register_resolve_req_v1::Reader>()?;
                    let entity = Uuid::from_u64_pair(
                        reader.get_entity().unwrap().get_upper(),
                        reader.get_entity().unwrap().get_lower(),
                    );

                    let mut name: Vec<(String, String)> = vec![];
                    for kv in reader.get_name()? {
                        name.push((
                            kv.get_key()?.to_string().unwrap(),
                            kv.get_value()?.to_string().unwrap(),
                        ));
                    }

                    Ok(Message::RegisterResolveReqV1(name, entity))
                }
                MessageTypes::ErrorResV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

                    let reader = reader.get_root::<error_res_v1::Reader>()?;

                    Ok(Message::ErrorResV1(
                        reader.get_error_code(),
                        reader.get_what().unwrap().to_string().unwrap(),
                    ))
                }
                MessageTypes::UnregisterResolveReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

                    let reader = reader.get_root::<unregister_resolve_req_v1::Reader>()?;

                    Ok(Message::UnregisterResolveReqV1(Uuid::from_u64_pair(
                        reader.get_entity().unwrap().get_upper(),
                        reader.get_entity().unwrap().get_lower(),
                    )))
                }

                MessageTypes::HistoryReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<history_req_v1::Reader>()?;
                    let entity = reader.get_entity()?;

                    Ok(Message::HistoryReqV1(
                        Uuid::from_u64_pair(entity.get_upper(), entity.get_lower()),
                        chrono::DateTime::from_timestamp_nanos(reader.get_start_time() as i64),
                        chrono::DateTime::from_timestamp_nanos(reader.get_end_time()),
                        reader.get_interval_max(),
                        reader.get_history_type().map_err(|err| {
                            capnp::Error::failed(format!("invalid history type: {}", err.0))
                        })?,
                    ))
                }
                MessageTypes::HistoryResV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<history_res_v1::Reader>()?;

                    let history = reader.reborrow().get_hist()?;
                    let time_delta = history.get_time_delta()?.iter().collect();
                    let values = match history.get_raw_or_agg().which().map_err(|err| {
                        capnp::Error::failed(format!("invalid history values union: {}", err.0))
                    })? {
                        history::raw_or_agg::Which::Value(values) => {
                            HistoryValues::Raw(values?.iter().collect())
                        }
                        history::raw_or_agg::Which::Aggregate(aggregates) => {
                            HistoryValues::Aggregate(
                                aggregates?
                                    .iter()
                                    .map(|aggregate| Aggregate {
                                        minimum: aggregate.get_minimum(),
                                        maximum: aggregate.get_maximum(),
                                        sum: aggregate.get_sum(),
                                        count: aggregate.get_count(),
                                        integral: aggregate.get_integral(),
                                        active_time: aggregate.get_active_time(),
                                    })
                                    .collect(),
                            )
                        }
                    };

                    Ok(Message::HistoryResV1(History { time_delta, values }))
                }
                MessageTypes::FluxClassifyJobReqV1 => {
                    todo!("Implement me!");
                }
                MessageTypes::FluxClassifyJobResV1 => {
                    todo!("Implement me!");
                }

                MessageTypes::FluxLifecycleHeartbeatV1 => {
                    todo!("Implement me!");
                }
                MessageTypes::RegisterResolveResV1 => Err(MessageError::ParseError(
                    "RegisterResolveResV1 is deprecated!".to_string(),
                )),
                MessageTypes::UnregisterResolveResV1 => Err(MessageError::ParseError(
                    "UnregisterResolveResV1 is deprecated!".to_string(),
                )),
                MessageTypes::GetReqV1 => Ok(Message::GetReqV1()),

                MessageTypes::GetResV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<get_res_v1::Reader>()?;

                    Ok(Message::GetResV1(
                        reader.get_value(),
                        chrono::DateTime::from_timestamp_nanos(reader.reborrow().get_time() as i64),
                    ))
                }
                MessageTypes::SetReqV1 => {
                    let reader =
                        capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                    let reader = reader.get_root::<set_req_v1::Reader>()?;

                    Ok(Message::SetReqV1(reader.reborrow().get_value()))
                }
            }
        }
    }

    /// Serializes the Message to the Cap'n'Proto format.
    ///
    /// Returns the serialized message or an Error if no Serialization could be performed.
    pub fn to_capnp(&self) -> Result<(u64, Vec<u8>), MessageError> {
        match self {
            Message::TimeValueV1(t, v) => {
                let mut bld = capnp::message::TypedBuilder::<time_value_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                msg.set_time(t.timestamp_nanos_opt().unwrap() as u64);
                msg.set_value(*v);
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::TimeValueV1 as u64, buffer))
            }
            Message::ResolveReqV1(attrs) => {
                let mut bld = capnp::message::TypedBuilder::<resolve_req_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut attr_builder = msg.init_name(attrs.len().try_into().unwrap());

                for (i, attr) in attrs.iter().enumerate() {
                    attr_builder
                        .reborrow()
                        .get(i as u32)
                        .set_key(attr.0.clone());
                    attr_builder
                        .reborrow()
                        .get(i as u32)
                        .set_value(attr.1.clone());
                }

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::ResolveReqV1 as u64, buffer))
            }
            Message::ResolveResV1(uuids) => {
                let mut bld = capnp::message::TypedBuilder::<resolve_res_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut attr_builder = msg.init_entities(uuids.len().try_into().unwrap());

                for (i, uuid) in uuids.iter().enumerate() {
                    let (upper, lower) = uuid.as_u64_pair();
                    attr_builder.reborrow().get(i as u32).set_upper(upper);
                    attr_builder.reborrow().get(i as u32).set_lower(lower);
                }
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::ResolveResV1 as u64, buffer))
            }
            Message::HelloWorldReqV1(str_msg) => {
                let mut bld =
                    capnp::message::TypedBuilder::<hello_world_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                msg.set_msg(str_msg);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::HelloWorldReqV1 as u64, buffer))
            }
            Message::HelloWorldResV1(str_msg) => {
                let mut bld =
                    capnp::message::TypedBuilder::<hello_world_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                msg.set_msg(str_msg);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::HelloWorldResV1 as u64, buffer))
            }
            Message::VectorReqV1(vec) => {
                let mut bld = capnp::message::TypedBuilder::<vector_req_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut values_builder = msg.init_values(vec.len() as u32);

                for (i, elem) in vec.iter().enumerate() {
                    values_builder.reborrow().set(i as u32, *elem);
                }
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::VectorReqV1 as u64, buffer))
            }
            Message::VectorResV1(vec) => {
                let mut bld = capnp::message::TypedBuilder::<vector_res_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut values_builder = msg.init_results(vec.len() as u32);

                for (i, elem) in vec.iter().enumerate() {
                    values_builder.reborrow().set(i as u32, *elem);
                }
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::VectorResV1 as u64, buffer))
            }
            Message::ReverseResolveReqV1(entity) => {
                let mut bld =
                    capnp::message::TypedBuilder::<reverse_resolve_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                let mut uuid_builder = msg.reborrow().init_entity();

                let uuid_pair = entity.as_u64_pair();
                uuid_builder.set_upper(uuid_pair.0);
                uuid_builder.set_lower(uuid_pair.1);
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::ReverseResolveReqV1 as u64, buffer))
            }
            Message::ReverseResolveResV1(name) => {
                let mut bld = capnp::message::TypedBuilder::<resolve_req_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut attr_builder = msg.init_name(name.len().try_into().unwrap());

                for (i, attr) in name.iter().enumerate() {
                    attr_builder
                        .reborrow()
                        .get(i as u32)
                        .set_key(attr.0.clone());
                    attr_builder
                        .reborrow()
                        .get(i as u32)
                        .set_value(attr.1.clone());
                }
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::ReverseResolveResV1 as u64, buffer))
            }
            Message::FluxLifecycleEventV1(str_data) => {
                let mut bld =
                    capnp::message::TypedBuilder::<flux_lifecycle_event_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                msg.set_data(str_data);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::FluxLifecycleEventV1 as u64, buffer))
            }

            Message::RegisterResolveReqV1(name, entity) => {
                let mut bld =
                    capnp::message::TypedBuilder::<register_resolve_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                let mut uuid_builder = msg.reborrow().init_entity();

                let uuid_pair = entity.as_u64_pair();
                uuid_builder.set_upper(uuid_pair.0);
                uuid_builder.set_lower(uuid_pair.1);

                let mut attr_builder = msg.init_name(name.len().try_into().unwrap());
                for (i, attr) in name.iter().enumerate() {
                    attr_builder
                        .reborrow()
                        .get(i as u32)
                        .set_key(attr.0.clone());
                    attr_builder
                        .reborrow()
                        .get(i as u32)
                        .set_value(attr.1.clone());
                }
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::RegisterResolveReqV1 as u64, buffer))
            }

            Message::UnregisterResolveReqV1(entity) => {
                let mut bld =
                    capnp::message::TypedBuilder::<unregister_resolve_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                let mut uuid_builder = msg.reborrow().init_entity();

                let uuid_pair = entity.as_u64_pair();
                uuid_builder.set_upper(uuid_pair.0);
                uuid_builder.set_lower(uuid_pair.1);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::UnregisterResolveReqV1 as u64, buffer))
            }

            Message::HistoryReqV1(entity, start, end, interval_max, history_type) => {
                let mut bld = capnp::message::TypedBuilder::<history_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                let mut entity_builder = msg.reborrow().init_entity();
                let (upper, lower) = entity.as_u64_pair();
                entity_builder.set_upper(upper);
                entity_builder.set_lower(lower);
                msg.set_start_time(start.timestamp_nanos_opt().unwrap());
                msg.set_end_time(end.timestamp_nanos_opt().unwrap());
                msg.set_interval_max(*interval_max);
                msg.set_history_type(*history_type);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::HistoryReqV1 as u64, buffer))
            }

            Message::HistoryResV1(history) => {
                let mut bld = capnp::message::TypedBuilder::<history_res_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut history_builder = msg.init_hist();
                let mut time_delta_builder = history_builder
                    .reborrow()
                    .init_time_delta(history.time_delta.len() as u32);
                for (index, value) in history.time_delta.iter().enumerate() {
                    time_delta_builder.set(index as u32, *value);
                }

                match &history.values {
                    HistoryValues::Raw(values) => {
                        let mut values_builder = history_builder
                            .get_raw_or_agg()
                            .init_value(values.len() as u32);
                        for (index, value) in values.iter().enumerate() {
                            values_builder.set(index as u32, *value);
                        }
                    }
                    HistoryValues::Aggregate(aggregates) => {
                        let mut aggregates_builder = history_builder
                            .get_raw_or_agg()
                            .init_aggregate(aggregates.len() as u32);
                        for (index, aggregate) in aggregates.iter().enumerate() {
                            let mut aggregate_builder =
                                aggregates_builder.reborrow().get(index as u32);
                            aggregate_builder.set_minimum(aggregate.minimum);
                            aggregate_builder.set_maximum(aggregate.maximum);
                            aggregate_builder.set_sum(aggregate.sum);
                            aggregate_builder.set_count(aggregate.count);
                            aggregate_builder.set_integral(aggregate.integral);
                            aggregate_builder.set_active_time(aggregate.active_time);
                        }
                    }
                }

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::HistoryResV1 as u64, buffer))
            }

            Message::UserDefined(payload_type, payload) => Ok((*payload_type, payload.to_vec())),
            Message::ErrorResV1(error_code, what) => {
                let mut bld = capnp::message::TypedBuilder::<error_res_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                msg.set_what(what);
                msg.set_error_code(*error_code);
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::ErrorResV1 as u64, buffer))
            }
            Message::GetReqV1() => {
                let mut bld = capnp::message::TypedBuilder::<get_req_v1::Owned>::new_default();
                bld.init_root();
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::GetReqV1 as u64, buffer))
            }
            Message::GetResV1(value, time) => {
                let mut bld = capnp::message::TypedBuilder::<get_res_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                msg.reborrow().set_value(*value);
                msg.reborrow()
                    .set_time(time.timestamp_nanos_opt().unwrap() as u64);
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::GetReqV1 as u64, buffer))
            }
            Message::SetReqV1(value) => {
                let mut bld = capnp::message::TypedBuilder::<set_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                msg.reborrow().set_value(*value);
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::SetReqV1 as u64, buffer))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;

    fn round_trip(message: Message) -> Message {
        let (payload_type, data) = message.to_capnp().unwrap();
        Message::from_capnp(payload_type, &data).unwrap()
    }

    #[test]
    fn history_request_round_trips() {
        let entity = Uuid::from_u64_pair(0x1234, 0x5678);
        match round_trip(Message::HistoryReqV1(
            entity,
            DateTime::from_timestamp_nanos(100),
            DateTime::from_timestamp_nanos(200),
            30,
            crate::msg_capnp::HistoryType::FlexTimeline,
        )) {
            Message::HistoryReqV1(actual_entity, start, end, interval_max, history_type) => {
                assert_eq!(actual_entity, entity);
                assert_eq!(
                    (start, end, interval_max),
                    (
                        DateTime::from_timestamp_nanos(100),
                        DateTime::from_timestamp_nanos(200),
                        30
                    )
                );
                assert_eq!(history_type, crate::msg_capnp::HistoryType::FlexTimeline);
            }
            message => panic!("unexpected message: {message:?}"),
        }
    }

    #[test]
    fn raw_history_response_round_trips() {
        let history = History {
            time_delta: vec![0, 10, 20],
            values: HistoryValues::Raw(vec![1.5, 2.5, 3.5]),
        };

        match round_trip(Message::HistoryResV1(history.clone())) {
            Message::HistoryResV1(actual) => assert_eq!(actual, history),
            message => panic!("unexpected message: {message:?}"),
        }
    }

    #[test]
    fn aggregate_history_response_round_trips() {
        let history = History {
            time_delta: vec![5],
            values: HistoryValues::Aggregate(vec![Aggregate {
                minimum: 1.0,
                maximum: 4.0,
                sum: 10.0,
                count: 4,
                integral: 7.5,
                active_time: 42,
            }]),
        };

        match round_trip(Message::HistoryResV1(history.clone())) {
            Message::HistoryResV1(actual) => assert_eq!(actual, history),
            message => panic!("unexpected message: {message:?}"),
        }
    }
}

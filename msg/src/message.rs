// SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>
//
// SPDX-License-Identifier: MIT

use thiserror::Error;
use uuid::Uuid;

use crate::msg_capnp::MessageTypes;
use crate::msg_capnp::flux_lifecycle_event_v1;
use crate::msg_capnp::hello_world_req_v1;
use crate::msg_capnp::register_resolve_req_v1;
use crate::msg_capnp::register_resolve_res_v1;
use crate::msg_capnp::resolve_req_v1;
use crate::msg_capnp::resolve_res_v1;
use crate::msg_capnp::reverse_resolve_req_v1;
use crate::msg_capnp::reverse_resolve_res_v1;
use crate::msg_capnp::time_value_v1;
use crate::msg_capnp::unregister_resolve_req_v1;
use crate::msg_capnp::unregister_resolve_res_v1;
use crate::msg_capnp::vector_req_v1;
use crate::msg_capnp::vector_res_v1;

/// FOOBAR
#[derive(Debug)]
pub enum Message {
    TimeValueV1(u64, f64),
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
    RegisterResolveResV1(bool, String),
    UnregisterResolveReqV1(Uuid),
    UnregisterResolveResV1(bool, String),
    UserDefined(u64, Vec<u8>)
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Deserialization failed: {0}")]
    CapnpError(#[from] capnp::Error),
    #[error("Got message of unknown payload type: {0}")]
    PayloadError(u64),
}

impl Message {
    /// Creates a Message from the given Cap'n'Proto buffer.
    ///
    /// * `data`: Buffer with Cap'n'Proto
    ///
    /// Returns Some(Message) if the data is parsable as a Message and None if not.
    pub fn from_capnp(payload_type: u64, data: &[u8]) -> Result<Message, MessageError> {

        if payload_type > 1 << 63
        {
            Ok(Message::UserDefined(payload_type, data.to_vec()))
        }
        else
        {
        match MessageTypes::try_from(payload_type as u16).unwrap() {
            MessageTypes::TimeValueV1 => {
                let reader =
                    capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

                let reader = reader.get_root::<time_value_v1::Reader>()?;

                Ok(Message::TimeValueV1(reader.get_time(), reader.get_value()))
            }
            MessageTypes::ResolveReqV1 => {
                let reader =
                    capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;
                let reader = reader.get_root::<resolve_req_v1::Reader>()?;

                let mut attrs: Vec<(String, String)> = vec![];
                for kv in reader.get_attributes()? {
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
                for addr in reader.get_matching_interfaces()? {
                    attrs.push(Uuid::from_u64_pair(addr.get_upper(), addr.get_lower()));
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
                    reader.get_address().unwrap().get_upper(),
                    reader.get_address().unwrap().get_lower(),
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
                let address = Uuid::from_u64_pair(
                    reader.get_address().unwrap().get_upper(),
                    reader.get_address().unwrap().get_lower(),
                );

                let mut name: Vec<(String, String)> = vec![];
                for kv in reader.get_name()? {
                    name.push((
                        kv.get_key()?.to_string().unwrap(),
                        kv.get_value()?.to_string().unwrap(),
                    ));
                }

                Ok(Message::RegisterResolveReqV1(name, address))
            }
            MessageTypes::RegisterResolveResV1 => {
                let reader =
                    capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

                let reader = reader.get_root::<register_resolve_res_v1::Reader>()?;

                Ok(Message::RegisterResolveResV1(
                    reader.get_success(),
                    reader.get_err_msg().unwrap().to_string().unwrap(),
                ))
            }
            MessageTypes::UnregisterResolveResV1 => {
                let reader =
                    capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

                let reader = reader.get_root::<unregister_resolve_res_v1::Reader>()?;

                Ok(Message::UnregisterResolveResV1(
                    reader.get_success(),
                    reader.get_err_msg().unwrap().to_string().unwrap(),
                ))
            }
            MessageTypes::UnregisterResolveReqV1 => {
                let reader =
                    capnp::serialize::read_message(data, capnp::message::ReaderOptions::new())?;

                let reader = reader.get_root::<unregister_resolve_req_v1::Reader>()?;

                Ok(Message::UnregisterResolveReqV1(Uuid::from_u64_pair(
                    reader.get_address().unwrap().get_upper(),
                    reader.get_address().unwrap().get_lower(),
                )))
            }

            MessageTypes::HistoryReqV1 => {
                todo!("Implement me!");
            }
            MessageTypes::HistoryResV1 => {
                todo!("Implement me!");
            }
            MessageTypes::FluxClassifyJobReqV1 => {
                todo!("Implement me!");
            }
            MessageTypes::FluxClassifyJobResV1 => {
                todo!("Implement me!");
            }

            MessageTypes::FluxLifecycleHeartbeatV1=> {
                todo!("Implement me!");
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

                msg.set_time(*t);
                msg.set_value(*v);
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::TimeValueV1 as u64, buffer))
            }
            Message::ResolveReqV1(attrs) => {
                let mut bld = capnp::message::TypedBuilder::<resolve_req_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut attr_builder = msg.init_attributes(attrs.len().try_into().unwrap());

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

                let mut attr_builder =
                    msg.init_matching_interfaces(uuids.len().try_into().unwrap());

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
            Message::ReverseResolveReqV1(address) => {
                let mut bld =
                    capnp::message::TypedBuilder::<reverse_resolve_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                let mut uuid_builder = msg.reborrow().init_address();

                let uuid_pair = address.as_u64_pair();
                uuid_builder.set_upper(uuid_pair.0);
                uuid_builder.set_lower(uuid_pair.1);
                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;

                Ok((MessageTypes::ReverseResolveReqV1 as u64, buffer))
            }
            Message::ReverseResolveResV1(name) => {
                let mut bld = capnp::message::TypedBuilder::<resolve_req_v1::Owned>::new_default();
                let msg = bld.init_root();

                let mut attr_builder = msg.init_attributes(name.len().try_into().unwrap());

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

            Message::RegisterResolveReqV1(name, addr) => {
                let mut bld =
                    capnp::message::TypedBuilder::<register_resolve_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                let mut uuid_builder = msg.reborrow().init_address();

                let uuid_pair = addr.as_u64_pair();
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

            Message::RegisterResolveResV1(success, err_msg) => {
                let mut bld =
                    capnp::message::TypedBuilder::<register_resolve_res_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                msg.set_success(*success);
                msg.set_err_msg(err_msg);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::RegisterResolveResV1 as u64, buffer))
            }
            Message::UnregisterResolveResV1(success, err_msg) => {
                let mut bld =
                    capnp::message::TypedBuilder::<unregister_resolve_res_v1::Owned>::new_default();
                let mut msg = bld.init_root();

                msg.set_success(*success);
                msg.set_err_msg(err_msg);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::UnregisterResolveResV1 as u64, buffer))
            }

            Message::UnregisterResolveReqV1(addr) => {
                let mut bld =
                    capnp::message::TypedBuilder::<unregister_resolve_req_v1::Owned>::new_default();
                let mut msg = bld.init_root();
                let mut uuid_builder = msg.reborrow().init_address();

                let uuid_pair = addr.as_u64_pair();
                uuid_builder.set_upper(uuid_pair.0);
                uuid_builder.set_lower(uuid_pair.1);

                let mut buffer = vec![];
                capnp::serialize::write_message(&mut buffer, bld.borrow_inner())?;
                Ok((MessageTypes::UnregisterResolveReqV1 as u64, buffer))
            }

            Message::UserDefined(payload_type, payload) => {
                Ok((*payload_type, payload.to_vec()))
            }
        }
    }
}

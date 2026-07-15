# Client library for the SeanerBUS

This sub-directory contains Rust and C++ implementations of the Data Plane communication API.

In pseudo-code, all the implementations expose the following functions:

```c
// Subscribe to pub/sub exposed by "addr".
void subscribe(Address addr);

// Publish a payload "payload" of "payload_type" to "addr"
void publish(Address addr, u64 payload_type, blob payload);

// Wait for incoming Messages (incoming requests or subscribed pub/sub messages)
Message read_msg();

// Register a req/res interface at "addr"
void register_reqres(Address addr);

// Perform a request to "addr" with a "payload" of "payload_type", returning the
// answer of the requester.
Message request(Address addr, u64 payload_type, blob payload);

// Respond to an "inc_request" with a "payload" of "payload_type"
void respond_to(Message inc_request, u64 payload_type, blob payload);
```

This library is only concerned with allowing the basic communication patterns of pub/sub and req/res.

To transfer useful data, encode your your message as one of the Cap'n'Proto formats defined in `seanerbus/msg`.

To publish a time-series value, you would use (again, pseudo-code):
```c

// Encode a single, time-value pair.
(payload_type, payload) = TimeValueV1(42, 1.0).to_capnp();

publish(some_addr, payload_type, payload);
```

`seanerbus/rust_examples` contains full examples for publishing, subscribing, requesting and responding with the Rust client library.

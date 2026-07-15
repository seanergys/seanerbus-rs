<!--
SPDX-FileCopyrightText: 2026 SEANERGYS Partners <https://seanergys.eu>

SPDX-License-Identifier: MIT
-->

# Examples of using the Rust client library

This directory contains basic examples of using the Rust client library with SeanerBUS.

## publish_test

This example will start publishing TimeValueV1 payloads to the
UUID given on the command line:

```
$ cargo run --bin publish_test 7bc3d210-a49f-4422-8fe8-26ca480d0e1f

Publishing time=1, value=1
Publishing time=2, value=2
[...]
```

## subscribe_test

This example will subscribe to the UUID given on the command line, expecting
TimeValueV1 values:

```
$ cargo run --bin subscribe_test 7bc3d210-a49f-4422-8fe8-26ca480d0e1f

time=1, value=1
time=2, value=2
time=3, value=3
[...]
```

## request_test

This will send a `HelloWorldV1` request to the UUID given on the command line, and print the result it has got:

```
$ cargo run --bin request_test 7bc3d210-a49f-4422-8fe8-26ca480d0e1f

Got an answer: Appreciate your "Hello World"
```

## response_test

This will listen on the UUID given on the command line and, if it is a HelloWorldV1Req, respond with a HelloWorldV1Res.

```
$ cargo run --bin response_test 7bc3d210-a49f-4422-8fe8-26ca480d0e1f

Got a HelloWorldV1Req request, msg: Hello World
Got a HelloWorldV1Req request, msg: Hello World
```

---
permalink: /rpc
---

# RPC

This directory contains an RPC implementation over attested TLS connection
written in Rust, providing trusted channels to send and handle requests.
RPC interfaces and request/response messages can be defined in ProtoBuf and
used for generating Rust structs and traits to implement services or client
function to send requests.

Similar with other RPC frameworks, there are several concepts of RPC in
Teaclave.

## Channel and Client

A channel in RPC represent a connection to the target service. Clients can use
the channel to send requests. In Teaclave, we implement `SgxTrustedTlsChannel`,
which can establish and attested a remote connection. For example, to connect
the management service, you need to establish a trusted channel with the service
first. Then, create a client of management service with the channel. At last,
you can use this client to send requests like `InvokeTask`.

When constructing a client, you can use the `SgxTrustedTlsClientConfig` to setup
TLS and attestation configs.

## Server and Service

Server is an entity to listening a network address, processing incoming
messages, and forwarding requests to certain service. Similar with channel in
Teaclave, we implement `SgxTrustedTlsServer`, which can establish a attested TLS
channel with clients.

Similar with the client, you can use `SgxTrustedTlsServerConfig` to setup TLS
and attestation configs.

## Protocol

There are many RPC protocols can be implemented in the RPC framework. Currently,
there's only one simple protocol called `JsonProtocol`. Simply speaking, for
the json protocol, one RPC message will contain a length of the following
requests (in big endian) and a json serialized request.

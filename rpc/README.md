--- 
permalink: /docs/codebase/rpc 
---

# RPC

This directory contains TLS configuration over an attested TLS connection,
providing trusted channels to send and handle requests.

Re-export [Tonic](https://github.com/hyperium/tonic) to support the general
gRPC framework. `Tonic` is a gRPC over HTTP/2 implementation focused on high
performance, interoperability, and flexibility.

## Channel and Client

A channel in gRPC represents a connection to the target service. Clients can
use the channel to send requests. When constructing a client, you can use the
`SgxTrustedTlsClientConfig` to set up TLS and attestation configurations so
that we can establish and attest to a remote connection. For example, to
connect the management service, you need to establish a trusted channel with
the service first. Then, create a client for the management service with the
channel. At last, you can use this client to send requests like `InvokeTask`.


## Server and Service

A server is an entity that listens to a network address, processes incoming
messages, and forwards requests to certain services. Similar to the client, you
can use `SgxTrustedTlsServerConfig` to set up TLS and attestation
configurations for the channel with clients.


## Interceptor

In Teaclave, we implement `CredentialService` based on the `Interceptor` trait
to add a credential to the MetadataMap of each request before it is sent, so
servers can check the authentication credential of each request.

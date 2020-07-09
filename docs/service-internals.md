---
permalink: /docs/service-internals
---

# Teaclave Service Internals

Teaclave Service is one of the most important abstraction in the platform.
Basically, the Teaclave FaaS platform is the combination of different functional
services and connected through trusted channels. The Teaclave services include
authentication service, frontend service, management service, storage service,
access control service, scheduler service, and execution service. They play
different roles in the system.

To understand the design and internal implementation of these services, we need
to discuss in these sections: RPC and protocol, app-enclave structure, and
attestation mechanism.

## RPC and Protocols

We use Protocol Buffers (version 3) to define message and RPC interfaces of the
Teaclave services. For example, the authentication service has this definition.

```proto
message UserLoginRequest {
  string id = 1;
  string password = 2;
}

message UserLoginResponse {
  string token = 1;
}

service TeaclaveAuthenticationApi {
  rpc UserLogin (UserLoginRequest) returns (UserLoginResponse);
}
```

This means that the authentication service (for the API endpoint) has a RPC
interface called "`UserLogin`", which takes a "`UserLoginRequest`" message with
`id` and `password` inside (in string type) and reply a "`UserLoginResponse`"
message with `token` inside (in string type).

With this definition, the build system will help to generate utility functions,
traits and structures for clients to send RPC requests, and for service to
implement functions of handling requests. This is done by a build time tool
called [`proto_gen`](https://github.com/apache/incubator-teaclave/tree/master/services/proto/proto_gen).

There is another layer in the `teaclave_proto` crate to help convert protobuf's
simple data type to Rust's more concise data type. For example, a URL is defined
in the string type in protobuf, while in Rust we can use the `Url` struct to
store a URL.

For more protocol definitions for other services, please see proto files in
the [`proto` directory](https://github.com/apache/incubator-teaclave/tree/master/services/proto/src/proto).

## Service Implementation Structure

A service in Teaclave consists of two parts: the app (untrusted) part and the
enclave (trusted) part. The app part is responsible for managing the service,
launching and terminating the enclave part, while the enclave part is to serve
RPC requests from clients (or other services) through trusted channels, execute
logic and process data in the trusted execution environment.

### App (Untrusted)

Basically, the app part of a service does the followings:
- Load the runtime configuration from the config file.
- Create a service launcher: prepare the binder and set the serialized config as
  an input.
- Start the service enclave (*ecall* to the trusted enclave part).
- Misc: register signal handlers so that the app/enclave can response with some
  signals.

### Enclave (Trusted)

Typically, a service's implementation in the enclave part contains two important
structs and one trait. Let's take the frontend service as an example.

- `TeaclaveFrontendService` (struct): Define properties or configurations along
  with the lifetime of the service. For example, the frontend service need to
  hold clients (with established trusted channels) to communicate with the
  authentication service and management service.
- `TeaclaveFrontendError` (struct): Define errors which may occur in this
  service, authentication error, for example.
- `TeaclaveFrontend` (trait): Define functions (requests) the service need to
  handle. The trait will be automatically derived from definitions in the
  ProtoBuf file and can be imported from the `teaclave_proto` crate.
  
You will see some `#[handle_ecall]` annotations on functions and the
`register_ecall_handler` macro to help with the function registration.
The lifecycle of a Teaclave service consists of enclave initialized, service
started, and enclave finalized, which will invoke the corresponding command
handlers - `InitEnclave`, `StartService`, and `FinalizeEnclave`.

The start service function is the entry point of an enclave service. Here are
steps to prepare and start to serving requests.

- Initialize the attestation config and make an remote attestation.
- With the endorsed attestation report, initialize a attested TLS config.
- Initialize a TLS server with TLS config and listening address.
- If needed, initialize service endpoints this service want to connect to. For
  example, the frontend service need to connect to the authentication service
  and management service.
- Start the service (with endpoint handlers) and begin to serve requests.

## Attestation in Services

To explain the usages of remote attestation mechanism in services, we need to
consider two different scenarios: 1) the service wants serve other RPC requests,
2) the service wants to connect and send requests to other services.

For the first scenario, the endorsed attestation report is used for creating the
trusted TLS server, so that clients can attest the service's report to verify
the platform. In the meantime, if the service wants to attest clients (a.k.a.,
establishing mutual attestation), we need to get the accepted enclave attributes
from the *enclave info* first to create the trusted TLS server. By this, the
server can also attest clients' attestation reports and only accept expected
connections.

For the second scenario, the report is used to create a trusted TLS channel so
that the client can present its report when establishing the channel. Also, the
server's report will be verified.

## Customize a Standalone Service

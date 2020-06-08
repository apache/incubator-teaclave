---
permalink: /services
---

# Teaclave Services

This directory contains protocol definitions and implementations of services in
the Teaclave platform.

## Services

The platform includes several services (or subsystem) running inside TEE, and
communicated with *mutual-attested channels*. They coordinate together to provide
a safe and secure FaaS platform.

- **Authentication Service**: A reference implementation of user authentication
  infrastructure. Here, we use JSON Web Token (JWT), a simple and widely-used
  authentication standard, to provide a secure authentication mechanism in the
  platform. Clients need to get valid token before interacting with the platform.
- **Frontend Service**: This is the entry point of all requests from users. It will
  validate user's identity/token and forward requests to appropriate services.
- **Management Service**: This service plays an important role in the whole services.
  It handles almost all requests, such as registering functions/data, creating
  tasks, and invoking tasks. Also, the management service will contact the
  access control service to authorize operations when needed. In addition, task
  and function information will be persistent into the storage services.
- **Storage Service**: Basically, the storage service stores persistent data like
  function, execution data, and task information in the platform. Here, we
  deploy a key-value database (an implementation of LevelDB) in TEE and use the
  protected file system (secured by the enclave) for data persistence.
- **Access Control Service**: Provides a flexible access control domain specific
  language to support access control rules for secure multi-party computation.
  The access control engine is written in Python and evaluated in SGX. Please
  read [this document](../docs/access-control.md) to learn more about the design of it.
- **Scheduler Service**: Schedules staged tasks ready for execution to a proper
  execution node with desirable capabilities.
- **Execution Service**: A host of different executors interacting with the
  scheduler service to complete tasks. There could be many execution service
  instances (or nodes) with different capabilities deployed in a cloud
  infrastructure.

## Structure

A service is consist of two parts: app (untrusted) and enclave (trusted). The
app part is responsible for launching and terminating the service, which the
enclave part is to serve RPC requests from trusted channels. Typically, a service's
implementation contains two important structs and one trait. Let's take the
frontend service as an example.

- `TeaclaveFrontendService` (struct): Define properties or configurations along
  with the lifetime of the service. For example, the frontend service need to
  hold clients (with established trusted channels) to communicate with the
  authentication service and management service.
- `TeaclaveFrontendError` (struct): Define errors which may occur in this
  service, authentication error, for example.
- `TeaclaveFrontend` (trait): Define functions (requests) the service need to
  handle. The trait will be automatically derived from definitions in the
  ProtoBuf file and can be imported from the `teaclave_proto` crate.
  
## RPC and Protocols

We use ProtoBuf to define messages and RPC interfaces of the Teaclave services.
You can find detailed protocol definitions in the
[`proto` directory](https://github.com/apache/incubator-teaclave/tree/master/services/proto/src/proto).
In addition, utility functions, traits, and structures will be automatically
generated to help implementing services to handle requests and send responses.
This is done by a build time tool called
[`proto_gen`](https://github.com/apache/incubator-teaclave/tree/master/services/proto/proto_gen).

## Topology

These services are communicating through RPC with remote attestation.
This topological graph illustrates connections between services.

```
clients => authentication <-+       +----> storage <----+
                            |       |                   |
clients => frontend ----------> management            scheduler <-- execution
                                    |
                                    +--> access_control


                                                  =>      api endpoint connections
                                                  -> internal endpoint connections
```

Internal endpoint connections will be established and verified with mutual
remote attestation to ensure the integrity and confidentiality of the whole system.
Therefore, clients can trust the whole platform and safely interacting with the
system through the attested authentication and frontend services.

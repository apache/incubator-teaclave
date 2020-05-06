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
  deploy a key-value database (an implementation of LevelDB) in TEE and use
  protected file system (secured by the enclave) for data persistence.
- **Access Control Service**: Provides a flexible access control configurations
  to support access control rules for multi-party secure computation.
- **Scheduler Service**: Schedules staged task ready for execution to a proper
  execution node with desirable capabilities.
- **Execution Service**: A host of different executors interacting with the
  scheduler service to complete tasks. There could be many execution service
  instances (or nodes) with different capabilities deployed in a cloud
  infrastructure.
  
## Protocols

We use ProtoBuf to define messages and RPC interfaces of the Teaclave services.
Utility functions, traits, and structures will be automatically generated to
help implementing services to handle requests and send responses. You can find
detailed protocol definitions in the [`proto` files](./proto/src/proto).

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
remote attestation to ensure the integrity and confidentiality of whole system.
Therefore, clients can trust the whole platform and safely interacting with the
system through the attested authentication and frontend services.

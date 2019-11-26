# Repository Structure

+ Project Root
	+ [auditors](../auditors)
		- auditor credentials
	+ [cmake](../cmake)
		- build scripts/utilities
	+ [cert](../cert)
		- pre-generated keys/certificates that are used in the prototype. Note
		  that these are only testing keys. Do not use them in production.
	+ [docs](../docs)
		- documentations
	+ [examples](../examples)
		- examples
	+ [mesatee_binder](../mesatee_binder)
		- abstract communication interfaces between TEE untrusted/trusted worlds
	+ [mesatee_config](../mesatee_config)
		- compile-time and runtime configuration utilities
	+ [mesatee_core](../mesatee_core)
		- core of MesaTEE, including [IPC](../mesatee_core/src/ipc)/[RPC](../mesatee_core/src/rpc)/[Error-handling](../mesatee_core/src/error.rs)/[Database](../esatee_core/src/db.rs)/etc. -- everything you need to develop a TEE services and clients
	+ [mesatee_sdk](../mesatee_sdk)
		- client SDK. Applications can utilize it to invoke MesaTEE services
	+ [mesatee_services](../mesatee_services)
		- [fns](../mesatee_services/fns): function node service, trusted gateway in front of the actual worker
		- [kms](../mesatee_services/kms): key management service
		- [tdfs](../mesatee_services/tdfs): trusted distributed file system
		- [tms](../mesatee_services/tms): task management service
	+ [tests](../tests)
		- functional and integration tests
	+ [third_party](../third_party)
		- third party dependencies vendored as git submodules.
	+ [build.toml](../build.toml)
		- compile-time configuration
	+ [config.toml](../config.toml)
		- runtime configuration

# FAQs in Build and Run

## Why did I see ``SGX launch check failed: ias_spid.txt or ias_key.txt does NOT exist``?

Because the Intel Attestation Service (IAS) requires mutual authentication in
TLS communications. So if you have followed [build
prerequisite](how_to_build.md#prerequisite) document for Intel Attestation
Service (IAS) registration, you should be able to obtain the SPID, Primary Key,
and Secondary Key . Please configure their paths in the ``ias_client_config``
section of [config.toml](../config.toml) accordingly. 

MesaTEE uses the most recent [Intel IAS API version 5](https://api.trustedservices.intel.com/documents/sgx-attestation-api-spec.pdf).
It no longer requires certificate from IAS client. Instead, it requires a **Subscription Key** for access. Please read the [manual](https://api.trustedservices.intel.com/documents/sgx-attestation-api-spec.pdf) and [build prerequisite](how_to_build.md#prerequisite) for details.

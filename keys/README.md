# Keys and Certificates in Teaclave

This directory contains keys and certificates used in the Teaclave platform.
Note that these are only for demonstration. *DO NOT use them in production.*

- `enclave_signing_key.pem`: private key to sign SGX enclaves
- `ias_root_ca_cert.pem`: attestation report root CA certificate for Intel SGX
  Attestation Service, obtained from the
  [service website](https://api.portal.trustedservices.intel.com/EPID-attestation)
- `dcap_root_ca_cert.pem` and `dcap_root_ca_cert.der`: Root CA certificate (in PEM
  and DER formats, respectively) used for conntecting to the reference DCAP
  attestation server and verifying ECDSA attestation reports. Examples. Should
  NOT be used in production.
- `dcap_server_cert.pem` and `dcap_server_key.pem`: DCAP attestation server
  end-entity certificate and private key. Certificate is signed by DCAP root CA.
- `auditors`: contains auditors' keys to sign the *enclave info* for mutual
  attestation

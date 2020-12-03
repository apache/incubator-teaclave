---
permalink: /docs/codebase/cli
---

# Teaclave Command Line Tool

The Teaclave command line tool (`teaclave_cli`) provides utilities to
interactive with the platform. The command line tool has several sub-commands:

- `encrypt`/`decrypt`: These two subcommands are to encrypt/decrypt data used on
  the platform. Supported algorithms include AES-GCM (128bit and 256 bit), and
  Teaclave File (128bit).
- `verify`: Verify the signatures of the enclave info (which contains `MRSIGNER`
  and `MRENCLAVE`) signed by auditors with their public keys. The enclave info
  is used for remote attestation, Please verify it before connecting the
  platform with the client SDK.
- `attest`: Establish an attested TLS with one of the Teaclave services and get
  an attestation report, validate it with attestation service's cert and display
  the report details.

## Encrypt/Decrypt

Here are two examples to encrypt and decrypt files with the CLI.

```
$ ./teaclave_cli encrypt \
    --algorithm teaclave-file-128 \
    --key 00000000000000000000000000FF1234 \
    --input-file ${FILE} \
    --output-file ${ENCRYPTED_FILE} \
    --print-cmac
cfba09e4c2bc72ea9e5392d779c2926c

$ ./teaclave_cli decrypt \
    --algorithm teaclave-file-128 \
    --key 00000000000000000000000000FF1234 \
    --input-file ${ENCRYPTED_FILE} \
    --output-file ${DECRYPTED_FILE}
```

## Verify

Here is an example to verify auditors' signatures of the enclave info file.

```
$ ./teaclave_cli verify \
    --enclave-info ../examples/enclave_info.toml \
    --public-keys $(find ../examples -name "*.public.pem") \
    --signatures $(find ../examples -name "*.sign.sha256")
Verify successfully.
```

## Attest

Here is an example to display the attestation report from a Teaclave service.

```
$ ./teaclave_cli attest --address accvm-dev:7776 --as-ca-cert ../../keys/ias_root_ca_cert.pem
Report Freshness: 1854s
SGX Quote status: SwHardeningNeeded
Version and signature/key type: Version 2, EPID Linkable signature
GID or reserved: 3014
Security version of the QE: 11
Security version of the PCE: 10
ID of the QE vendor: 00000000-XXXX-XXXX-XXXX-XXXXXXXXXXXX
Custom user-defined data (hex): 75b6024c00000000000000000000000000000000
CPU version (hex): 0f0f0305ff8006000000000000000000
SSA Frame extended feature set: 0
Attributes of the enclave (hex): 07000000000000000700000000000000
Enclave measurement (hex): eadeb5537962d2451a8619fb6a4b10b72f56479e0b7db0bb9c3f5edc143ca6eb
Hash of the enclave singing key (hex): 83d719e77deaca1470f6baf62a4d774303c899db69020f9c70ee1dfc08c7ce9e
Enclave product ID: 0
Security version of the enclave: 0
The value of REPORT (hex): 317cb5c0d9a26747a08833e51bac8ca2ce814aa362c8cd0e2672fdcb6bfee77b9ba32ed7d605778aa52b9f2d2ce698f83ec49e6beecb89c684d861bb078d7dc2
```

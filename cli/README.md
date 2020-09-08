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

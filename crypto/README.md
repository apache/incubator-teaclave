# Crypto Primitives

This directory hosts all the implementations of cryptographic primitives used in
Teaclave for encryption/decryption of registered data.

Current crypto primitives include:

- AES GCM: Commonly used symmetric-key cryptographic block ciphers. Supported
  key sizes are: 128bits, 256bits.
- Teaclave File Key: Key for Teaclave file system (i.e., protected FS). Only
  128bits key is supported.

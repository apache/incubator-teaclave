# Mutual Attestation: Why and How

The standard procedure to establish a secure and trusted communication channel
from a client to an enclave is through remote attestation. However, when the
client itself is also an enclave and *mutual* trust between two enclaves is
required, we need additional design and implementation effort. The Teaclave
platform consists of multiple enclave services and most of the
enclave-to-enclave RPC communications need bidirectional authentication. This
document entails the methodology and process of Teaclave's mutual enclave remote
attestation.

## Problem

The identity of an enclave is defined through a pair of cryptographically secure
hash values, i.e., MRSIGNER and MRENCLAVE. MRSIGNER indicates the builder of the
enclave, thus shared by enclaves signed by the same party. MRENCLAVE is unique
to each individual enclave. Teaclave assumes that users do not trust the
software builder, so verifying MRSIGNER is not enough. For each enclave service
in Teaclave, it must strictly check the unique identity of the other enclaves it
communicates to through MRENCLAVE.

Since the SGX enclave trusts no outside input, the MRENCLAVE should be
hard-coded into source files used for identity verification logic. Therefore,
changing the MRENCLAVE value an enclave tries to match against will change the
MRENCLAVE of the enclave itself. When two enclaves want to remotely attest each
other, it is impossible to decide which enclave is to be built first.

## Solution

Teaclave resolves this problem by replying on third-party auditors. We assume
that there will be several parties trusted by all participants of Teaclave's
computation tasks (could platforms, data providers, and customers, etc). The
source code and binaries of Teaclave are audited by these trusted parties. Once
the auditors decided that Teaclave is secure, they sign and publish the
identities of audited enclaves. The *public keys* of the auditors are
hard-coded in Teaclave enclave source via build time configuration, while the
enclave measures and their signatures are loaded from outside at runtime. Each
enclave will verify that the enclave measures are indeed signed by the auditors
before serving any requests.

## In the Repository 

The [keys/auditors](../keys) directory in the source tree contain the key pairs
of three fake auditing parties for PoC purposes. Private keys are also included
to deliver a smooth build and test process. In production, builders of Teaclave
should obtain the public keys, enclave identities, and the signatures directly
from the auditors.

# Mutual Attestation: Why and How
The standard procedure to establish a secure and trusted communication channel
from a client to an enclave is through remote attestation.  However, when the
client itself is also an enclave and **mutual** trust between two enclaves is
required, we need additional design and implementation effort. The MesaTEE
framework consists of multiple enclave services and most of the
enclave-to-enclave communications need bidirectional authentication. This
document entails the methodology and process of MesaTEE's mutual enclave remote
attestation. 

## Problem
The identity of an enclave is defined through a pair of cryptographically
secure hash values, i.e., MRSIGNER and MRENCLAVE. MRSIGNER indicates the
builder of the enclave, thus shared by enclaves signed by the same party.
MRENCLAVE is unique to each individual enclave. MesaTEE assumes that users do
not trust the software builder, so verifying MRSIGNER is not enough.  For each
enclave service in MesaTEE, it must strictly check the unique identity of the
other enclaves it communicates to through MRENCLAVE.

Since the SGX enclave trusts no outside input, the MRENCLAVE should be
hard-coded into source files used for identity verification logic. Therefore,
changing the MRENCLAVE value an enclave tries to match against will change the
MRENCLAVE of the enclave itself. When two enclaves want to remotely attest each
other, it is impossible to decide which enclave is to be built first.

## Solution
MesaTEE resolves this problem by replying on third-party auditors. We assume
that there will be several parties trusted by all participants of MesaTEE's
computation tasks (could platforms, data providers, and customers, etc). The
source code and binaries of MesaTEE are audited by these trusted parties. Once
the auditors decided that MesaTEE is secure, they sign and publish the
identities of audited enclaves.  The **public keys** of the auditors are
hard-coded in MesaTEE enclave source, while the enclave measures and their
signatures are loaded from outside at runtime. Each enclave will verify that
the enclave measures are indeed signed by the auditors before serving any
requests.

## In the Repository 
The [auditors](../auditors) directory in the source tree contain the key pairs
of three fake auditing parties for PoC purposes. Private keys are also included
to deliver a smooth build and test process. In production, builders of MesaTEE
should obtain the public keys, enclave identities, and the signatures directly
from the auditors.

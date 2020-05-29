---
permalink: /attestation
---

This directory contains the implementation of the attestation in Apache
Teaclave.

# Attestation in Teaclave

Attestation is the process of demonstrating that a software component is running
properly on a Trusted Execution Environment (e.g., Intel SGX).

Teaclave combines the remote attestation with a TLS connection to improve the
trustworthiness of two endpoints. Once established, it has attested that the
running parties are inside trusted enclaves and provided trusted channels with
end-to-end encryption, the enclave code's identity, and other information.

The platform includes several services, and each service is running inside an
enclave. Those services communicate with mutual-attested TLS channels.

## How it Works

We integrate the attestation process in the TLS handshake. The attested-TLS
handshake is similar to a normal TLS handshake, except the extension of the
certificate includes an SGX attestation report. We make the certificate
cryptographically bound to a specific enclave instance by adding the public key
of the certificate in the attestation report.

During the build time, the public keys of the auditor enclaves are hard-coded in
Teaclave services enclave, and enclave measurements and signatures are loaded
from outside during the runtime. Auditor enclaves verify and sign the identity
of each service enclave. After each service receives the attestation report, it
will verify whether the `MR_SIGNER` and `MR_ENCLAVE` from the attestation report
match the identity information signed by auditor enclaves. After that, it will
verify the TLS certificate. If all the verifications pass, a secure attested
channel is established between two enclaves.

Please note the trusted channel can also have one-way (client -> server)
attestation. Under the circumstances, only the server needs to run inside TEEs.

## Attestation Report

## Verification

There are many information included in an attestation report such as CPU
version, ISV version, product ID, etc. By default, Teaclave will check
`MR_ENCLAVE` and other basic information. Users can also define a customized
verification function to check more information in attestation reports by
implementing the `AttestationReportVerificationFn` function.

## Freshness

To make sure the platform is always up-to-date and trusted, Teaclave will update
attestation report periodically. By default, the validity time of an attestation
report is 3600 seconds. It can be changed in the
[`build.config.toml`](https://github.com/apache/incubator-teaclave/blob/master/config/build.config.toml)
file.

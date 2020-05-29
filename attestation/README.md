---
permalink: /config
---

This directory contains the implementation of the attestation in Apache Teaclave.

# Attestation in Teaclave

Attestation is the process of demonstrating that a software component is running properly on a Trusted Execution Environment (e.g, Intel SGX). 

Teaclave combines the remote attestation with a TLS connection to improve the trustworthiness of two endpoints. Once established, it can attest the running party is inside an enclave, the identity of the enclave code, and other information.

The platform include several services and each service is running inside an enclave. Those services communicate with a mutual-attested channel.

## How it works

The attested-TLS handshake is similar to a normal TLS handshake, except the extension of the certificate includes an SGX attestation report. We make the certificate cryptographically bound a specific enclave instance by adding the public key of the certificate in the attestation report.  

During the build time, the public keys of the auditor enclaves are hard-coded in Teaclave services enclave, and enclave measurements and signatures are loaded from outside during the runtime. Auditor enclaves verify and sign the identity of each service enclave. After each service receives the attestation report, it will verify whether the `MR_SIGNER` and `MR_ENCLAVE` from the attestation report match the identity information signed by auditor enclaves. After that, it will verify the
TLS certificate. If all the verifications pass, a secure attested channel is 
established between two enclaves.

Please note the the trusted channel can also has one-way (client -> server) attestation. Under the circumstances, only the server is running inside TEE. 

## Freshness

The attestation report is update periodically. Currently, the validity of the
attestation report is 3600 seconds. It can be changed in the [`build.config.toml`](https://github.com/apache/incubator-teaclave/blob/master/config/build.config.toml)
file.

## Customized Attestation 

Users can defined their own rules to verifying attestation reports by implementing 
function `AttestationReportVerificationFn`.




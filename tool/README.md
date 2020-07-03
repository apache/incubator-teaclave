---
permalink: /docs/codebase/tool
---

# Teaclave SGX Tool

This tool is to dump some SGX related information, e.g., hardware and software
information, remote attestation report. This can help to diagnose some issues
which may caused by the platform settings.

## Hardware/Software Status

To dump the SGX related hardware and software information, you can use this
command:

```
$ ./teaclave_sgx_tool status
Vendor: GenuineIntel
CPU Model: Intel(R) Core(TM) i9-9900K CPU @ 3.60GHz
SGX:
  Has SGX: true
  Has SGX1: true
  Has SGX2: false
  Supports ENCLV instruction leaves EINCVIRTCHILD, EDECVIRTCHILD, and ESETCONTEXT: false
  Supports ENCLS instruction leaves ETRACKC, ERDINFO, ELDBC, and ELDUC: false
  Bit vector of supported extended SGX features: 0x00000000
  Maximum supported enclave size in non-64-bit mode: 2^31
  Maximum supported enclave size in 64-bit mode: 2^36
  Bits of SECS.ATTRIBUTES[127:0] set with ECREATE: 0x0000000000000036 (lower) 0x000000000000001F (upper)
  EPC physical base: 0x00000000B0200000
  EPC size: 0x0000000005D80000 (93M)
  Supports flexible launch control: true

...
```

## Remote Attestation Report

Use the following command to dump remote attestation report and configure the
platform accordingly:

```
$ ./teaclave_sgx_tool attestation --key {as_key} --spid {as_spid} --url {as_url} --algorithm {as_algorithm}
Remote Attestation Report:
{
  "advisoryIDs": [
    "INTEL-SA-00161",
    "INTEL-SA-00320",
    "INTEL-SA-00329",
    "INTEL-SA-00220",
    "INTEL-SA-00270",
    "INTEL-SA-00293",
    "INTEL-SA-00233"
  ],
  "advisoryURL": "https://security-center.intel.com",
  ...
}
```

---
permalink: /docs/codebase/binder
---

# Binder

The binder library provides communication interfaces between TEE's app/enclave
worlds. More specific, the binder implements a message passing protocol for
intra-procedure communication. The protocol provides a secure and (type) safe
channel to pass information. For example, in Teaclave, we use the binder library
to launch Teaclave services and pass runtime configurations to trusted enclaves.

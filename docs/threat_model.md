# Threat Model

With its strongest security setting applied, Teaclave guarantees data
confidentiality even if all parties along the computation path, privileged or
not, are untrusted. This includes:

- Internet service provider
- Cloud provider
- Function provider
- Other data providers 

Consider the following scenario. A small business needs to employ image
classification techniques in its daily production. However, the business does
not have the capabilities to train a high-quality machine learning model, nor
does it have the hardware resources to host the machine learning
infrastructures. Under such circumstances, the only solution is to subscribe to
some cloud computing service and run the needed image classification tasks
remotely. However, this solution requires the small business to upload its
private data to the cloud, which may deeply concerns the business owner and
hinders the deployment of such techniques.

With Teaclave, privacy concerns above are no more. The small business can
subscribe to the cloud service from company A, rent the machine learning model
from company B, and use the deep learning inference engine provided by company
C. None of these parties need to trust another, yet the computation can commence
with everyone's privacy respected.

In the settings above, the root of trust converges to Intel and its SGX-enabled
CPU chips. Before the computation starts, Teaclave is booted as a secure SGX
enclave on one of these CPUs owned by the cloud service provider. After that,
each party can **remotely** attest the authenticity of the hardware and the
integrity of Teaclave platform. Private data are securely provisioned to the
Teaclave enclave only if the attestation passes. After the provision, no
privileged software is able to access the memory content owned by the enclave
from outside.

The remote attestation functionality implemented by Teaclave is augmented from
the method described by an Intel [white paper](https://arxiv.org/abs/1801.05863).
The complicated structure of Teaclave requires additional work for remote
attestation, which is explained in details via a separate
[documentation](mutual_attestation.md).

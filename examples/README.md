# Examples

This directory contains several examples to illustrate the application
scenarios. The [case studies](../docs/case_study.md) document contains more
descriptive information.

* [quickstart](quickstart)
	- Quickstart demo of how to invoke MesaTEE services. This can be a great
	  starting point for new users.
* [image_resizing](image_resizing)
	- One can invoke MesaTEE services similar to [AWS
	  Lambda](https://aws.amazon.com/lambda/).  On data uploading or new events
coming, MesaTEE function services are immediately triggered. For example, you
can use MesaTEE to thumbnail images, transcode videos, index files, process
logs, validate content, and aggregate and filter data in real-time. In this
specific example, we demonstrate image resizing.
* [ml_predict](ml_predict)
	- MesaTEE also supports a variety of big data analyses and machine learning
	  algorithms, such as GBDT, Linear Regression, as well as neural networks.
In this specific example, we demonstrate how to utilize a GBDT model to perform
data prediction -- in the trusted secure fashion, without concerning privacy
leakage.
* [rsa_sign](rsa_sign) and [online_decrypt](online_decrypt)
	- Another killing feature of MesaTEE is to serve as a key vault or an HSM.
	  MesaTEE can conveniently provide secret management (securely store and
control accesses to tokens, passwords, certificates, API keys, and other
secrets), key management (create and control encryption keys), and certificate
management (provision and manage certificates).
* [private_join_and_compute](private_join_and_compute)
	- When cross-department or cross-company data collaboration happens,
	  privacy concerns arise. Thus secure multi-party computation (SMC) has
become more and more important nowadays to enable joint big data analyses.
However, traditional crypto-based SMC has quite a few limitations, and MesaTEE
can solve them effectively, with way better performance/flexibility
improvements. Details are discussed
[here](../docs/case_study.md#secure-multi-party-computation).
* [py_matrix_multiply](py_matrix_multiply)
	- In the era of FaaS and AI, Python rules them all. So we have another
	  dedicated project called
[MesaPy](https://github.com/mesalock-linux/mesapy). In this specific example,
we demonstrate how to invoke the MesaPy engine integrated into MesaTEE.

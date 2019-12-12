# Testing Keys/Certificates

This directory contains keys/certificates that are used in the prototype. Note
that these are only testing keys. Do not use them in production.

* AttestationReportSigningCACert.pem:
	- Intel Attestation Service (IAS) certificate obtained from
	  [here](https://software.intel.com/sites/default/files/managed/7b/de/RK_PUB.zip).
* ca.crt:
	- clients are authenticated during mutual TLS communications, so we need to
	  (offline) issue certificates to them. This is the CA certificate for
testing purpose.
* client.crt:
	- client's certificate used in mutual TLS authentication (issued by
	  ca.crt).
* client.pkcs8:
	- client's private key used in mutual TLS authentication (matching
	  client.crt).
* mr_signer:
	- SHA256 digest of the big endian format modulus of the RSA public key of
	  the enclave’s signing key. The value we put here matches our [testing
signing key](../build/Enclave_private.pem).

After the registration with IAS, you will be issued a service provider ID
(SPID) via email. You need to provide an spid.txt file containing your SPID
string such as ``ABCDEFGHIJKLMNOPQRSTUVWXYZ012345`` in this directory.

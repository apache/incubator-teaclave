One of the attractive applications of MesaTEE is the trusted secure crypto
processing. Traditionally, this cannot be easily achieved without the help of
an expensive Hardware Security Module (HSM). Similar to HSM, MesaTEE can
safeguard and manage digital keys for strong authentication and provides crypto
processing, and provide tamper resistance or evidence promises.

MesaTEE HSM supports both symmetric and asymmetric cryptography as well as
certificate management. It accepts user-uploaded crypto materials, or can help
users to generate them within the protected environment.

Here we simply demonstrate the RSA signing functionality based on a
user-uploaded private key. The key is always encrypted, no matter in
transmission or in storage; the only chance of using it in plaintext is in the
MesaTEE enclave memory. So MesaTEE can be as secure as hardware HSMs while
significantly reducing the cost, adding more flexibilities, yet preserving
compatibilites/functionalities.

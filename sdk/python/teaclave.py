#!/usr/bin/env python3

import struct
import json
import base64
import toml
import os

from cryptography import x509
from cryptography.hazmat.backends import default_backend

from OpenSSL.crypto import load_certificate, FILETYPE_PEM, FILETYPE_ASN1
from OpenSSL.crypto import X509Store, X509StoreContext
from OpenSSL import crypto


def write_message(sock, message):
    message = json.dumps(message)
    message = message.encode()
    sock.write(struct.pack(">Q", len(message)))
    sock.write(message)


def read_message(sock):
    response_len = struct.unpack(">Q", sock.read(8))
    response = sock.read(response_len[0])
    response = json.loads(response)
    return response


def verify_report(as_root_ca_cert_path, enclave_info_path, cert, endpoint_name):
    if os.environ.get('SGX_MODE') == 'SW':
        return

    cert = x509.load_der_x509_certificate(cert, default_backend())
    ext = json.loads(cert.extensions[0].value.value)

    report = bytes(ext["report"])
    signature = bytes(ext["signature"])
    signing_cert = bytes(ext["signing_cert"])
    signing_cert = load_certificate(FILETYPE_ASN1, signing_cert)

    # verify signing cert with AS root cert
    with open(as_root_ca_cert_path) as f:
        as_root_ca_cert = f.read()
    as_root_ca_cert = load_certificate(FILETYPE_PEM, as_root_ca_cert)
    store = X509Store()
    store.add_cert(as_root_ca_cert)
    store.add_cert(signing_cert)
    store_ctx = X509StoreContext(store, as_root_ca_cert)
    store_ctx.verify_certificate()

    # verify report's signature
    crypto.verify(signing_cert, signature, bytes(ext["report"]), 'sha256')

    report = json.loads(report)
    quote = report['isvEnclaveQuoteBody']
    quote = base64.b64decode(quote)

    # get mr_enclave and mr_signer from the quote
    mr_enclave = quote[112:112+32].hex()
    mr_signer = quote[176:176+32].hex()

    # get enclave_info
    enclave_info = toml.load(enclave_info_path)

    # verify mr_enclave and mr_signer
    enclave_name = "teaclave_" + endpoint_name + "_service"
    if mr_enclave != enclave_info[enclave_name]["mr_enclave"]:
        raise Exception("mr_enclave error")

    if mr_signer != enclave_info[enclave_name]["mr_signer"]:
        raise Exception("mr_signer error")

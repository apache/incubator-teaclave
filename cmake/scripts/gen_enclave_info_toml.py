#!/usr/bin/env python3
import sys


def find_hex_value(content, section):
    index = content.index(section)
    # assume each element in content is ending with '\n'
    hex_bytes = ''.join(content[index + 1:index + 3]).split()
    return ''.join(['%02x' % int(x, 16) for x in hex_bytes])


mr_signer = "mrsigner->value:\n"
mr_enclave = "metadata->enclave_css.body.enclave_hash.m:\n"

content = sys.stdin.readlines()

mr_signer_hex = find_hex_value(content, mr_signer)
mr_enclave_hex = find_hex_value(content, mr_enclave)

sys.stdout.write("""[{}]
mr_enclave = "{}"
mr_signer  = "{}"
""".format(sys.argv[1], mr_enclave_hex, mr_signer_hex))

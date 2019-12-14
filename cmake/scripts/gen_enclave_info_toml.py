import sys

def find_hex_value(content, section):
    index = content.index(section)
    # assume each element in content is ending with '\n'
    hex_bytes = ''.join(content[index+1:index+3]).split()
    return ''.join(['%02x' % int(x, 16) for x in hex_bytes])

mrsigner = "mrsigner->value:\n"
enclave_hash = "metadata->enclave_css.body.enclave_hash.m:\n"

content = sys.stdin.readlines()

mrsigner_hex = find_hex_value(content, mrsigner)
enclave_hash_hex = find_hex_value(content, enclave_hash)

sys.stdout.write("""[{}]
mrsigner     = "{}"
enclave_hash = "{}"
""".format(sys.argv[1], mrsigner_hex, enclave_hash_hex))

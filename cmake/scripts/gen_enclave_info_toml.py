import sys

def find_hex_value(content, section):
    index = content.index(section)
    content = content[index+1:index+3]
    content = content[0].split() + content[1].split()
    return ''.join([x[2:4] for x in content])

mrsigner = "mrsigner->value:\n"
enclave_hash = "metadata->enclave_css.body.enclave_hash.m:\n"

content = sys.stdin.readlines()

mrsigner_hex = find_hex_value(content, mrsigner)
enclave_hash_hex = find_hex_value(content, enclave_hash)

sys.stdout.write("""[{}]
mrsigner     = "{}"
enclave_hash = "{}"
""".format(sys.argv[1], mrsigner_hex, enclave_hash_hex))

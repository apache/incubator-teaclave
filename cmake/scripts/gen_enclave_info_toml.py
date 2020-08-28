#!/usr/bin/env python3

# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

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

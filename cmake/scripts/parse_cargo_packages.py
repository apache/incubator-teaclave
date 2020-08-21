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
'''
[usage] python parse_cargo_package.py <cargo_toml_path> <teaclave_root>
This script parses Cargo.toml to print three lists for CMake
"<pkg_name_list>\n<pkg_path_list>\n<pkg_category_list>"
The items in each list are separated by ":"

In Cargo.toml, lines ending with "# ignore" will be omitted
e.g.
members = [
  "examples/neural_net",
  "teaclave_sdk", # ignore
]
'''
import os
import re
import sys


def parse_members_for_workspace(toml_path):
    """Parse members from Cargo.toml of the worksapce"""
    with open(toml_path, mode='rb') as f:
        data = f.read()

    manifest = data.decode('utf8')

    regex = re.compile(r"^members\s*=\s*\[(.*?)\]", re.S | re.M)
    members_block = regex.findall(manifest)[0]

    out = []

    members = members_block.split('\n')
    regex2 = re.compile(r'\s*"(.*?)".*')
    for mem in members:
        if (len(mem.strip()) == 0) or re.match(r".*#\signore", mem):
            continue
        out.append(regex2.findall(mem)[0])

    return out


def parse_package_name(package_toml_path):
    """Parse package name from Cargo.toml"""
    with open(package_toml_path, mode='rb') as f:
        data = f.read()

    manifest = data.decode('utf8')

    regex = re.compile(r'^name\s*=\s*"(.*?)"', re.M)

    return regex.findall(manifest)[0]


def pkg_path_2_category(pkg_path):
    """
    Take pkg path and return its category.
    Return services/examples/tests/cli/dcap depends on the beginning of pkg_path.
    (lib not used by this function)
    """
    if pkg_path.startswith('services/'):
        return 'services'
    elif pkg_path.startswith('examples/'):
        return 'examples'
    elif pkg_path.startswith('tests/'):
        return 'tests'
    elif pkg_path.startswith('tool/'):
        return 'tool'
    elif pkg_path == 'cli':
        return 'cli'
    elif pkg_path == 'dcap':
        return 'dcap'
    else:
        sys.stderr.write(
            '[Error]: Unknown category for package_path {}\n'.format(pkg_path))
        sys.exit(-1)


DEFAULT_EDL_LIB = "Enclave_common_t"
PKG_NAME_TO_EDL_LIB = {
    "teaclave_unit_tests_enclave": "Enclave_fa_t",
    "teaclave_execution_service_enclave": "Enclave_fa_t",
}


def pkg_name_2_edl_lib_name(pkg_name):
    """
    Take pkg_name and return its configured edl libary name, default is DEFAULT_EDL_LIB.
    """
    return PKG_NAME_TO_EDL_LIB.get(pkg_name, DEFAULT_EDL_LIB)


def main():
    """parses Cargo.toml to generate a list of package to be built"""
    if len(sys.argv) < 3:
        err = "[usage] python {} cargo_toml_path teaclave_root".format(
            sys.argv[0])
        raise ValueError(err)

    toml_path = sys.argv[1]
    teaclave_root = sys.argv[2]

    pkg_names = []
    pkg_paths = []
    pkg_categories = []
    edl_lib_names = []

    members = parse_members_for_workspace(toml_path)
    for pkg_path in members:
        pkg_toml_path = os.path.join(teaclave_root, pkg_path, "Cargo.toml")
        pkg_name = parse_package_name(pkg_toml_path)

        pkg_names.append(pkg_name)
        pkg_paths.append(pkg_path)
        pkg_categories.append(pkg_path_2_category(pkg_path))
        edl_lib_names.append(pkg_name_2_edl_lib_name(pkg_name))

    out = [
        ":".join(pkg_names), ":".join(pkg_paths), ":".join(pkg_categories),
        ":".join(edl_lib_names)
    ]
    sys.stdout.write("\n".join(out))


if __name__ == "__main__":
    main()

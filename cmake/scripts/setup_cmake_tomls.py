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
[usage] setup_cmake_tomls.py [project_root_dir] [project_build_dir]
Create cmake_tomls under build_dir
Create separate folders for unix_app|sgx_trusted|sgx_untrusted under build_dir/cmake_tomls
Create symlinks for Cargo.*.toml and folders so cargo build can run in separate folders
Setup Cargo config for enclaves
'''
import os
import os.path as osp
import sys

# symlinks won't be created for the following directories
SYM_FOLDER_BLACKLIST = ['docs', 'cmake', 'out', 'bin', 'build']

CATEGORIES = ['sgx_trusted_lib', 'sgx_untrusted_app', 'unix_app']


def exec_cmd(cmd):
    """execute a shell command"""
    # print(cmd)
    os.system(cmd)


def filter_sym_dir(root_dir, name):
    """return true if name corresponds to a folder and is not in blacklist"""
    return not name.startswith(
        '.') and not name in SYM_FOLDER_BLACKLIST and osp.isdir(
            osp.join(root_dir, name))


def create_symlinks(root_dir, build_dir):
    """create symolic links"""
    exec_cmd('mkdir -p {build_dir}/cmake_tomls'.format(build_dir=build_dir))

    sym_folders = list(
        filter(lambda name: filter_sym_dir(root_dir, name),
               os.listdir(root_dir)))

    for cate in CATEGORIES:
        cate_dir = '{build_dir}/cmake_tomls/{cate}'.format(build_dir=build_dir,
                                                           cate=cate)
        cmd = 'mkdir -p {cate_dir} && [ ! -f {cate_dir}/Cargo.toml ] && \
            ln -s {root_dir}/cmake/tomls/Cargo.{cate}.toml \
            {cate_dir}/Cargo.toml'.format(root_dir=root_dir,
                                          cate=cate,
                                          cate_dir=cate_dir)
        exec_cmd(cmd)

        for folder in sym_folders:
            cmd = '[ ! -d {cate_dir}/{folder} ] && ln -sn {root_dir}/{folder} \
                {cate_dir}/'.format(root_dir=root_dir,
                                    folder=folder,
                                    cate_dir=cate_dir)
            exec_cmd(cmd)


def setup_cargo_for_sgx(root_dir, build_dir):
    """setup cargo related files for sgx"""
    third_party_dir = os.path.join(root_dir, 'third_party')
    cmd = r'''mkdir -p {build_dir}/cmake_tomls/sgx_trusted_lib/.cargo \
    && cp -f {third_party_dir}/crates-sgx/Cargo.lock {build_dir}/cmake_tomls/sgx_trusted_lib/Cargo.lock \
    && cp -f {third_party_dir}/crates-sgx/config {build_dir}/cmake_tomls/sgx_trusted_lib/.cargo/config \
    && sed -i 's/directory = "vendor"/directory = "third_party\/crates-sgx\/vendor"/' \
    {build_dir}/cmake_tomls/sgx_trusted_lib/.cargo/config'''
    cmd = cmd.format(build_dir=build_dir, third_party_dir=third_party_dir)
    exec_cmd(cmd)


def setup_cargo_for_unix(root_dir, build_dir):
    """setup cargo related files for sgx"""
    third_party_dir = os.path.join(root_dir, 'third_party')
    for target in ["unix_app", "sgx_untrusted_lib", "sgx_untrusted_app"]:
        cmd = r'''mkdir -p {build_dir}/cmake_tomls/{target}/.cargo \
        && cp -f {third_party_dir}/crates-io/Cargo.lock {build_dir}/cmake_tomls/{target}/Cargo.lock \
        && cp -f {third_party_dir}/crates-io/config {build_dir}/cmake_tomls/{target}/.cargo/config \
        && sed -i 's/directory = "vendor"/directory = "third_party\/crates-io\/vendor"/' \
        {build_dir}/cmake_tomls/{target}/.cargo/config'''
        cmd = cmd.format(build_dir=build_dir,
                         third_party_dir=third_party_dir,
                         target=target)
        exec_cmd(cmd)


def main():
    """setup tomls for cmake"""
    if len(sys.argv) != 3:
        print(
            '[usage] setup_cmake_tomls.py [project_root_dir] [project_build_dir]'
        )
        sys.exit(-1)
    root_dir = sys.argv[1]
    build_dir = sys.argv[2]

    create_symlinks(root_dir, build_dir)
    setup_cargo_for_unix(root_dir, build_dir)
    setup_cargo_for_sgx(root_dir, build_dir)


if __name__ == "__main__":
    main()

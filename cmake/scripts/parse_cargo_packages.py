'''
[usage] python parse_cargo_package.py <cargo_toml_path> <workspace_path>
This script parses Cargo.toml to generate a list of package to be built for CMake
lines ending with "# ignore" will be omitted
e.g.
members = [
  "examples/neural_net",
  "mesatee_sdk", # ignore
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


def main():
    """parses Cargo.toml to generate a list of package to be built"""
    if len(sys.argv) < 3:
        err = "[usage] python {} cargo_toml_path workspace_path".format(
            sys.argv[0])
        raise ValueError(err)

    toml_path = sys.argv[1]
    workspace_path = sys.argv[2]

    packages = []

    members = parse_members_for_workspace(toml_path)
    for mem in members:
        pkg_toml_path = os.path.join(workspace_path, mem, "Cargo.toml")
        pkg_name = parse_package_name(pkg_toml_path)

        packages.append(pkg_name)

    sys.stdout.write(";".join(packages))


if __name__ == "__main__":
    main()

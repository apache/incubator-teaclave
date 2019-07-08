# Dependencies and Rust Packages (Crates) Vendoring

In order to ease auditing, ensure product stability, as well as reduce the
possibility of the [supply chain
attack](https://en.wikipedia.org/wiki/Supply_chain_attack), we vendored all TEE
dependencies here.  During the build process, the trusted components will only
consumes packages from this designated repository and will not download any code
from external sources such as [crates.io](https://crates.io).

## To Add A New Vendored Dependency

If a crate is not available in the vendor directory, it can to be added with
the following steps:

1. Add the crates you need in the corresponding Cargo.toml (e.g.
   [crates-sgx/Cargo.toml](https://github.com/mesalock-linux/crates-sgx/blob/master/Cargo.toml))
and update the crate list in the README.txt (e.g.
[crates-sgx/README.txt](https://github.com/mesalock-linux/crates-sgx/blob/master/README.txt)).
2. Run ``cargo build`` and ensure that it passes.
3. Run ``cargo vendor`` and update the config file (e.g. crates-sgx/config).
   You may also utilize
[crates-sgx/Makefile](https://github.com/mesalock-linux/crates-sgx/blob/master/Makefile)
for automation.
4. ``git add/commit`` the changes of
   Cargo.toml/Cargo.lock/config/README.txt/vendor and submit a pull request.

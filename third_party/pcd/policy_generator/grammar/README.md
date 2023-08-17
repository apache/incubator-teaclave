# Grammar of the Policy Definition Language (PDL)

This folder contains the grammar definition files for processing the human-readable policy files into source code that contains annotated structs for the Rust programming language, from which the API generator will generate the corresponding policy-compliant APIs for the target data.

The parent crate uses the `lalrpop` parser (LR(1) parser) for lexing and AST generating.

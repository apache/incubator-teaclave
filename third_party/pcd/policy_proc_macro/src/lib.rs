#![cfg_attr(test, allow(unused))]

use executors::ExecutorContext;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod executors;

pub(crate) mod fields;

/// This attribute will parse the struct that is annotated with `#[policy_carrying]` and automatically implement some
/// necessary interfaces for the given struct. Typically, this attribute should be accompanied with a schema/data over-
/// view that describes the input data format.
///
/// This procedural macro attribute generates the corresponding the interfaces for the executor.
#[proc_macro_attribute]
pub fn policy_carrying(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);

    let mut ctx = ExecutorContext::new(item_struct);

    println!("[+] Context built");
    let tt: TokenStream = ctx.executor_generation(args).into();
    println!("[+] tt built");

    println!("{}", tt.to_string());
    tt
}

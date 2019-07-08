// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn handle_ecall(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let ident = &f.ident;

    let input_types: Vec<_> = f
        .decl
        .inputs
        .iter()
        .map(|arg| match arg {
            &syn::FnArg::Captured(ref val) => &val.ty,
            _ => unreachable!(),
        })
        .collect();

    let args_type = match input_types.first().unwrap() {
        &syn::Type::Reference(ref r) => &r.elem,
        _ => unreachable!(),
    };

    let ret_type = match &f.decl.output {
        syn::ReturnType::Default => unreachable!(),
        syn::ReturnType::Type(_, ty) => ty,
    };

    let generic_type = match **ret_type {
        syn::Type::Path(ref path) => {
            let type_params = &path.path.segments.iter().next().unwrap().arguments;
            let generic_arg = match type_params {
                syn::PathArguments::AngleBracketed(params) => params.args.iter().next().unwrap(),
                _ => {
                    panic!("IPC Macro Attribute: unexpected return type, no AngleBracketed found.")
                }
            };

            match generic_arg {
                syn::GenericArgument::Type(ty) => ty.clone(),
                _ => panic!("IPC Macro Attribute: unexpected return type, no generic found."),
            }
        }
        _ => panic!("IPC Macro Attribute: unexpected return type."),
    };

    quote!(
        impl HandleRequest<#generic_type> for #args_type {
            fn handle(&self) -> #ret_type {
                #ident(self)
            }
        }

        #f
    )
    .into()
}

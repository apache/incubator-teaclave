// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn test_case(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);
    let f_ident = &f.sig.ident;
    let q = quote!(
        #f

        inventory::submit!(
            teaclave_test_utils::TestCase(
                concat!(module_path!(), "::", stringify!(#f_ident)).to_string(),
                #f_ident
            )
        );
    );

    q.into()
}

#[proc_macro_attribute]
pub fn async_test_case(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;

    if input.sig.asyncness.is_none() {
        let msg = "the async keyword is missing from the function declaration";
        return syn::Error::new_spanned(input.sig.fn_token, msg)
            .to_compile_error()
            .into();
    } else if !input.sig.inputs.is_empty() {
        let msg = "the test function cannot accept arguments";
        return syn::Error::new_spanned(&input.sig.inputs, msg)
            .to_compile_error()
            .into();
    } else if input.sig.output != syn::ReturnType::Default {
        let msg = "the test function cannot return outputs";
        return syn::Error::new_spanned(input.sig.output, msg)
            .to_compile_error()
            .into();
    }

    let result = quote!(
        #input

        inventory::submit!(
            teaclave_test_utils::AsyncTestCase(
                concat!(module_path!(), "::", stringify!(#name)).to_string(),
                || async move {#name().await }.boxed()
        )
    ););
    result.into()
}

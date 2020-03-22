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
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse_macro_input;
use syn::ItemStruct;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn into_request(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr_str = attr.to_string();
    let splits: Vec<&str> = attr_str.split("::").map(|s| s.trim()).collect();
    let name = Ident::new(splits[0], Span::call_site());
    let item = Ident::new(splits[1], Span::call_site());
    let f = parse_macro_input!(input as ItemStruct);
    let struct_ident = &f.ident;
    let q = quote!(
    #f


    impl teaclave_rpc::IntoRequest<#name> for #struct_ident {
        fn into_request(self) -> teaclave_rpc::Request<#name> {
            teaclave_rpc::Request::new(#name::#item(self.into()))
        }
    }
    );

    q.into()
}

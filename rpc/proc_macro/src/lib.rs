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

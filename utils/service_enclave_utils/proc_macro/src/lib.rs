extern crate proc_macro;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse_macro_input;
use syn::ItemStruct;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn teaclave_service(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr_str = attr.to_string();
    let splits: Vec<&str> = attr_str.split(", ").collect();
    let crate_name = Ident::new(splits[0], Span::call_site());
    let trait_name = splits[1];
    let trait_name_ident = Ident::new(trait_name, Span::call_site());
    let request = Ident::new(&format!("{}Request", trait_name), Span::call_site());
    let response = Ident::new(&format!("{}Response", trait_name), Span::call_site());

    let f = parse_macro_input!(input as ItemStruct);
    let struct_ident = &f.ident;
    let q = quote!(
        #f

        impl teaclave_rpc::TeaclaveService<#crate_name::proto::#request, #crate_name::proto::#response>
            for #struct_ident
        {
            fn handle_request(
                &self,
                request: #crate_name::proto::#request,
            ) -> std::result::Result<#crate_name::proto::#response, teaclave_types::TeaclaveServiceError> {
                use #crate_name::proto::#trait_name_ident;
                use log::trace;
                trace!("Dispatching request.");
                self.dispatch(request)
            }
        }
    );
    q.into()
}

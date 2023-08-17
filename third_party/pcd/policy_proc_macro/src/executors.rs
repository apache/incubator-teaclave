//! Generates the executor definitions.

use std::collections::HashMap;

use policy_core::{ast::Policy, types::DataType};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{ItemStruct, Meta, Type};

use crate::fields::handle_field_attribute;

const DATAFRAME_EXECUTOR: &str = "DataFrameExec";
const PROJECTION_EXECUTOR: &str = "ProjectionExec";
const FILTER_EXECUTOR: &str = "FilterExec";
const PARTITION_GROUPBY_EXECUTOR: &str = "PartitionGroupByExec";

const SUPPORTED_TYPES: &[&str] = &[
    "bool", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "String", "str",
];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ExecutorType {
    DataFrameScan,
    Projection,
    Filter,
    PartitionGroupBy,
}

#[derive(Clone)]
pub(crate) struct ExecutorIdent {
    pub(crate) base: Ident,
    pub(crate) full: Ident,
}

/// A context for generating the procedural macros for the executors.
pub(crate) struct ExecutorContext {
    pub(crate) item_struct: ItemStruct,
    /// A list of registed executor names.
    pub(crate) registered_executors: HashMap<ExecutorType, ExecutorIdent>,
}

impl ExecutorIdent {
    pub(crate) fn new(base: Ident, full: Ident) -> Self {
        Self { base, full }
    }
}

impl ExecutorContext {
    pub(crate) fn new(item_struct: ItemStruct) -> Self {
        let prefix = item_struct.ident.to_string();

        let base_df_executor = format_ident!("{}", DATAFRAME_EXECUTOR);
        let base_projection_executor = format_ident!("{}", PROJECTION_EXECUTOR);
        let base_filter_executor = format_ident!("{}", FILTER_EXECUTOR);
        let base_partition_executor = format_ident!("{}", PARTITION_GROUPBY_EXECUTOR);

        let df_executor = format_ident!("{}{}", prefix, base_df_executor);
        let projection_executor = format_ident!("{}{}", prefix, base_projection_executor);
        let filter_executor = format_ident!("{}{}", prefix, base_filter_executor);
        let partition_executor = format_ident!("{}{}", prefix, base_partition_executor);

        let mut ctx = Self {
            item_struct,
            registered_executors: HashMap::new(),
        };

        ctx.registered_executors.insert(
            ExecutorType::DataFrameScan,
            ExecutorIdent::new(base_df_executor, df_executor),
        );
        ctx.registered_executors.insert(
            ExecutorType::Projection,
            ExecutorIdent::new(base_projection_executor, projection_executor),
        );
        ctx.registered_executors.insert(
            ExecutorType::Filter,
            ExecutorIdent::new(base_filter_executor, filter_executor),
        );
        ctx.registered_executors.insert(
            ExecutorType::PartitionGroupBy,
            ExecutorIdent::new(base_partition_executor, partition_executor),
        );

        ctx
    }

    fn parse_fields(&mut self) -> Policy {
        let mut policy = Policy::default();

        // We collect the information from the struct.
        let mut clause = Vec::new();
        for field in self.item_struct.fields.iter() {
            match &field.ty {
                Type::Path(ty)
                    if SUPPORTED_TYPES
                        .contains(&ty.path.to_token_stream().to_string().as_str()) =>
                {
                    for attribute in field.attrs.iter() {
                        match &attribute.meta {
                            Meta::List(attr_list) => {
                                clause.push(attr_list.to_token_stream());
                                policy.schema_mut().push((
                                    field.ident.to_token_stream().to_string(),
                                    DataType::try_from(ty.path.to_token_stream().to_string())
                                        .unwrap(),
                                ));
                            }
                            meta => {
                                panic!("{} is not supported", meta.to_token_stream())
                            }
                        }
                    }
                }
                ty => panic!(
                    "unsupported type {} for field name {}",
                    ty.to_token_stream(),
                    field.ident.as_ref().unwrap(),
                ),
            }
        }

        policy.clause_mut().extend(
            clause
                .into_iter()
                .map(|clause| handle_field_attribute(clause.into()).unwrap()),
        );

        policy
    }

    pub(crate) fn executor_generation(&mut self, ts: proc_macro::TokenStream) -> TokenStream {
        let mut policy = self.parse_fields();
        // Collect extra policies from the whole struct.
        if let Some(top_policy) = handle_field_attribute(ts.into()) {
            policy.clause_mut().push(top_policy);
            policy.postprocess();
        }

        println!("policy => {policy:?}");
        let import = self.executor_import();
        let decl = self.executor_decl();
        let r#impl = self.executor_impl();
        quote! {
            #import
            #decl
            #r#impl
        }
    }

    fn executor_import(&self) -> TokenStream {
        let mut imports = vec![];
        for ty in self.registered_executors.keys() {
            let path = match ty {
                ExecutorType::DataFrameScan => quote! { scan::* },
                ExecutorType::Projection => quote! { projection::* },
                ExecutorType::Filter => quote! { filter::* },
                ExecutorType::PartitionGroupBy => {
                    quote! { groupby_partitioned::* }
                }
            };

            imports.push(path);
        }

        quote! {
            use policy_core::{types::*, error::*};
            use policy_carrying_data::DataFrame;
            use policy_execution::{executor::{*, #(#imports),*}};
        }
    }

    fn executor_decl(&self) -> TokenStream {
        let (base, full): (Vec<_>, Vec<_>) = self
            .registered_executors
            .values()
            .cloned()
            .map(|ident| (ident.base, ident.full))
            .unzip();
        quote! {
            #(#[derive(Clone)] pub struct #full(#base);)*
        }
    }

    /// Generates the implementation details of the executors.
    fn executor_impl(&self) -> TokenStream {
        let mut ts = vec![];
        let basic_impl = quote! {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn clone_box(&self) -> policy_execution::executor::Executor {
                Box::new(self.clone())
            }
        };

        for executor_ident in self.registered_executors.values() {
            let raw_name = executor_ident.full.to_string();
            let full = executor_ident.full.clone();

            let format = quote! {
                impl std::fmt::Debug for #full {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", #raw_name)
                    }
                }
            };

            let executor_impl = quote! {
                impl policy_execution::executor::PhysicalExecutor for #full {
                    #basic_impl

                    fn execute(&mut self, state: &ExecutionState) -> PolicyCarryingResult<DataFrame> {
                        todo!()
                    }
                }
            };

            let cur = quote! {#format #executor_impl};
            ts.push(cur);
        }

        quote! {
            #(#ts)*
        }
    }
}

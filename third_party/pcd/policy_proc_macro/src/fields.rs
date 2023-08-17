use std::{
    fmt::{Debug, Formatter},
    str::FromStr,
};

use policy_core::ast::Clause;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
// use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Error, ExprArray, Ident, Token,
};

static SUPPORTED_KEYWORD_LIST: [&str; 2] = ["attribute_list", "scheme"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum ClauseType {
    Allow,
    Deny,
    #[default]
    Unknown,
}

struct ItemClause(ItemKvEntry);

/// An expression that may take the inner part of the following form:
///
/// ```
/// allows(
///     attribute_list => ["foo", "bar"],
///     scheme => {
///         redact(3),
///         dp(1.0),
///     },
/// )
/// ```
struct ItemClauseParen {
    pub empty: bool,
    pub ty: ClauseType,
    pub tt: Vec<TokenStream>,
}

struct ItemKvEntry {
    pub key: String,
    pub value: TokenStream,
}

impl Debug for ItemKvEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

impl Parse for ItemClauseParen {
    /// This removes the outer parenthesis.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) {
            let ty = input.parse::<Ident>()?;

            let ty = match ty.to_string().to_ascii_lowercase().as_str() {
                "allow" | "allows" => ClauseType::Allow,
                "deny" | "denies" => ClauseType::Deny,
                _ => {
                    return Err(Error::new(
                        input.span(),
                        format!("Unexpected clause type {}", ty),
                    ))
                }
            };

            // Cannot directly convert `input` to a string because it must be fully consumed.
            let tt = input.parse::<TokenStream>()?.to_string().trim().to_string();
            let chars = tt.chars().collect::<Vec<_>>();
            if chars.first().unwrap() == &'(' && chars.last().unwrap() == &')' {
                let tt = tt[1..tt.len() - 1]
                    .split(";")
                    .filter(|&x| !x.is_empty())
                    .map(|tt| TokenStream::from_str(tt))
                    .collect::<std::result::Result<Vec<_>, proc_macro2::LexError>>()?;

                Ok(Self {
                    empty: false,
                    ty,
                    tt,
                })
            } else {
                Err(Error::new(
                    Span::call_site(),
                    "Expecting token '(' and/or ')'",
                ))
            }
        } else if input.is_empty() {
            Ok(Self {
                empty: true,
                ty: Default::default(),
                tt: Vec::new(),
            })
        } else {
            Err(Error::new(
                Span::call_site(),
                "unexpected token encountered",
            ))
        }
    }
}

impl Parse for ItemClause {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) {
            let key = input.parse::<Ident>()?.to_string().to_lowercase();
            if !SUPPORTED_KEYWORD_LIST.contains(&key.as_str()) {
                return Err(Error::new(input.span(), format!("Unexpected key: {key}")));
            }

            if input.peek(Token![=>]) {
                input.parse::<Token![=>]>()?;
                Ok(Self(ItemKvEntry {
                    key,
                    value: input.parse::<TokenStream>()?,
                }))
            } else {
                Err(Error::new(
                    input.span(),
                    format!("unexpected token encountered {input}"),
                ))
            }
        } else {
            Err(Error::new(input.span(), format!("Parsing error @ {input}")))
        }
    }
}

pub fn handle_field_attribute(input: TokenStream) -> Option<Clause> {
    let clauses = syn::parse2::<ItemClauseParen>(input).unwrap();
    if clauses.empty {
        return None;
    }

    let clauses_inner = clauses
        .tt
        .iter()
        .map(|tt| syn::parse2::<ItemClause>(tt.clone()).unwrap())
        .collect::<Vec<_>>();

    let mut clause = match clauses.ty {
        ClauseType::Allow => Clause::Allow {
            attribute_list: Vec::new(),
            scheme: Vec::new(),
        },
        ClauseType::Deny => Clause::Deny(Default::default()),
        ClauseType::Unknown => return None,
    };

    for kv_entry in clauses_inner {
        if handle_value(&mut clause, &kv_entry.0).is_err() {
            return None;
        }
    }

    Some(clause)
}

/// Parse the value of each key value pair into a clause.
fn handle_value(clause: &mut Clause, kv_entry: &ItemKvEntry) -> syn::Result<()> {
    match kv_entry.key.to_ascii_lowercase().as_str() {
        "attribute_list" => {
            let attribute_list = syn::parse2::<ExprArray>(kv_entry.value.clone())?;
            let attribute_list = attribute_list
                .elems
                .into_iter()
                .map(|expr| expr.to_token_stream().to_string());
            clause.attribute_list_mut().extend(attribute_list);
        }

        // TODO: implement this.
        "scheme" => (),
        _ => return Err(Error::new(Span::call_site(), "Unknown key")),
    }

    Ok(())
}

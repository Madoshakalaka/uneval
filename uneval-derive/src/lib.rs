//! `#[derive(Uneval)]` — emit Rust source code that reconstructs a value.
//!
//! See the `uneval` crate for full documentation. The derive walks the type
//! definition at macro expansion time, so it uses the *original Rust*
//! identifiers (not whatever `serde` would rename them to) and dispatches to
//! the `Uneval` impl of each field's actual type. That lets each type pick the
//! constructor expression that suits it: bare literals for `&'static str`,
//! `"...".to_string()` for `String`, `vec![...]` for `Vec<T>`, and so on.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, GenericParam, Ident, Index,
    parse_macro_input, parse_quote,
};

#[proc_macro_derive(Uneval)]
pub fn derive_uneval(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let body = match &input.data {
        Data::Struct(s) => emit_struct(&input.ident, s),
        Data::Enum(e) => emit_enum(&input.ident, e),
        Data::Union(u) => {
            return syn::Error::new(u.union_token.span, "Uneval cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let mut generics = input.generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(::uneval::Uneval));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::uneval::Uneval for #name #ty_generics #where_clause {
            fn uneval(&self, __w: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                #body
            }
        }
    };
    expanded.into()
}

fn emit_struct(name: &Ident, data: &DataStruct) -> TokenStream2 {
    let name_lit = ident_lit(name);
    match &data.fields {
        Fields::Unit => quote! {
            ::std::write!(__w, "{}", #name_lit)
        },
        Fields::Named(fields) => {
            let stmts = fields.named.iter().enumerate().map(|(i, f)| {
                let fname = f.ident.as_ref().unwrap();
                let fname_lit = ident_lit(fname);
                let sep = if i == 0 { "" } else { ", " };
                quote! {
                    ::std::write!(__w, "{}{}: ", #sep, #fname_lit)?;
                    ::uneval::Uneval::uneval(&self.#fname, __w)?;
                }
            });
            quote! {
                ::std::write!(__w, "{} {{ ", #name_lit)?;
                #(#stmts)*
                ::std::write!(__w, " }}")
            }
        }
        Fields::Unnamed(fields) => {
            let stmts = fields.unnamed.iter().enumerate().map(|(i, _)| {
                let idx = Index::from(i);
                let sep = if i == 0 { "" } else { ", " };
                quote! {
                    ::std::write!(__w, "{}", #sep)?;
                    ::uneval::Uneval::uneval(&self.#idx, __w)?;
                }
            });
            quote! {
                ::std::write!(__w, "{}(", #name_lit)?;
                #(#stmts)*
                ::std::write!(__w, ")")
            }
        }
    }
}

fn emit_enum(name: &Ident, data: &DataEnum) -> TokenStream2 {
    let name_lit = ident_lit(name);
    let arms = data.variants.iter().map(|v| {
        let vname = &v.ident;
        let vname_lit = ident_lit(vname);
        match &v.fields {
            Fields::Unit => quote! {
                Self::#vname => ::std::write!(__w, "{}::{}", #name_lit, #vname_lit),
            },
            Fields::Named(fields) => {
                let field_idents: Vec<&Ident> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let stmts = field_idents.iter().enumerate().map(|(i, fname)| {
                    let fname_lit = ident_lit(fname);
                    let sep = if i == 0 { "" } else { ", " };
                    quote! {
                        ::std::write!(__w, "{}{}: ", #sep, #fname_lit)?;
                        ::uneval::Uneval::uneval(#fname, __w)?;
                    }
                });
                quote! {
                    Self::#vname { #(#field_idents),* } => {
                        ::std::write!(__w, "{}::{} {{ ", #name_lit, #vname_lit)?;
                        #(#stmts)*
                        ::std::write!(__w, " }}")
                    },
                }
            }
            Fields::Unnamed(fields) => {
                let bindings: Vec<Ident> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("__f{}", i))
                    .collect();
                let stmts = bindings.iter().enumerate().map(|(i, b)| {
                    let sep = if i == 0 { "" } else { ", " };
                    quote! {
                        ::std::write!(__w, "{}", #sep)?;
                        ::uneval::Uneval::uneval(#b, __w)?;
                    }
                });
                quote! {
                    Self::#vname(#(#bindings),*) => {
                        ::std::write!(__w, "{}::{}(", #name_lit, #vname_lit)?;
                        #(#stmts)*
                        ::std::write!(__w, ")")
                    },
                }
            }
        }
    });
    quote! {
        match self {
            #(#arms)*
        }
    }
}

/// Format an identifier for emission into the generated Rust source.
///
/// `syn::Ident`'s `Display` impl already prints `r#foo` for raw identifiers, so
/// fields that the user wrote as `r#type` round-trip correctly. We additionally
/// guard against the (theoretical) case of receiving a bare keyword by adding
/// the `r#` prefix ourselves.
fn ident_lit(ident: &Ident) -> String {
    let s = ident.to_string();
    if s.starts_with("r#") {
        return s;
    }
    if needs_raw_prefix(&s) {
        format!("r#{}", s)
    } else {
        s
    }
}

fn needs_raw_prefix(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "else"
            | "enum"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "static"
            | "struct"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "gen"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

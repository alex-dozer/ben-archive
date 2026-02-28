mod data_objs;
mod dsl;
mod helpers;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;

use crate::bitspec::helpers::{make_codegen_items, parse_struct_fields};

pub fn expand_bitspec(ast: &DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &ast.ident;

    let parsed = parse_struct_fields(ast);

    let (pred_items, thresh_items, bit_count) = make_codegen_items(&parsed)?;

    let mod_name = syn::Ident::new(
        &format!("__bitspec_generated_{}", struct_name),
        struct_name.span(),
    );

    let expanded = quote! {
        mod #mod_name {
            use super::*;
            pub static PRED_LIST: &[PredicateSpec] = &[
                #(#pred_items),*
            ];

            pub static THRESH_LIST: &[ThresholdSpec] = &[
                #(#thresh_items),*
            ];
        }

        impl #struct_name {
            pub const BITSPEC_PACK: BitspecPack = BitspecPack {
                predicates: #mod_name::PRED_LIST,
                thresholds: #mod_name::THRESH_LIST,
                bit_count: #bit_count,
            };
        }
    };

    Ok(expanded)
}

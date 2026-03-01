mod encoding;
mod helpers;
mod render;
mod validate;

use darling::FromDeriveInput;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, LitStr, spanned::Spanned};

use crate::schema::{
    encoding::{decode_expr_for_type, encode_stmt_for_field},
    helpers::{clickhouse_type_for, option_inner_ty},
};

pub fn derive_ben_schema(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;

    // Parse bschema(...) attributes
    let spec = validate::SchemaSpec::from_derive_input(&input)
        .map_err(|e| syn::Error::new_spanned(&input, e.to_string()))?;

    // Validate basic bschema invariants
    validate::validate(&spec)?;

    // Build ClickHouse columns for manifest
    let mut column_defs = Vec::new();

    if let darling::ast::Data::Struct(ref fields) = spec.data {
        for f in fields.iter() {
            let f_ident = f.ident.as_ref().unwrap();
            let name = f_ident.to_string();
            let ty = &f.ty;

            let ch_ty = if let Some(ref et) = f.enum_type {
                match et.as_str() {
                    "string" => "String".to_string(),
                    "u64" => "UInt64".to_string(),
                    "i64" => "Int64".to_string(),
                    "f64" => "Float64".to_string(),
                    other => {
                        return Err(syn::Error::new(
                            f.ty.span(),
                            format!("Unknown enum backing type '{}'", other),
                        ));
                    }
                }
            } else {
                clickhouse_type_for(ty)?
            };
            let nullable = option_inner_ty(ty).is_some();
            let default_opt = match &f.default {
                Some(s) => quote! { Some(#s.to_string()) },
                None => quote! { None },
            };

            column_defs.push(quote! {
                ::ben_contracts::schema::ColumnDef {
                    name: #name.to_string(),
                    // `#ch_ty` becomes a string literal, we turn it into a runtime String:
                    ch_type: #ch_ty.to_string(),
                    nullable: #nullable,
                    default: #default_opt,
                }
            });
        }
    }

    let json = render::manifest_json(&spec)?;
    let fingerprint = render::fingerprint(&json);
    let ddl = render::ddl(&spec)?;

    let table = &spec.table;
    let version = spec.version;
    let table_lit = LitStr::new(table, Span::call_site());
    let json_lit = LitStr::new(&json, Span::call_site());
    let fingerprint_lit = LitStr::new(&fingerprint, Span::call_site());
    let ddl_lit = LitStr::new(&ddl, Span::call_site());

    let evt_hash_bytes = render::evt_hash(&json);
    let evt_hash_tokens = {
        let bytes = evt_hash_bytes;
        quote! { [ #(#bytes),* ] }
    };

    let field_count_u16: u16 = match &spec.data {
        darling::ast::Data::Struct(fields) => u16::try_from(fields.len()).unwrap_or(u16::MAX),
        _ => 0,
    };

    let mut encode_stmts: Vec<TokenStream2> = Vec::new();
    let mut decode_stmts: Vec<TokenStream2> = Vec::new();
    let mut field_inits: Vec<TokenStream2> = Vec::new();

    if let darling::ast::Data::Struct(ref fields) = spec.data {
        for f in fields.iter() {
            let field_ident = f
                .ident
                .as_ref()
                .expect("BenSchema only supports named struct fields");
            let field_ty = &f.ty;
            let stmt = encode_stmt_for_field(field_ident, field_ty, f).map_err(|e| {
                syn::Error::new(
                    field_ident.span(),
                    format!("encode error on field `{}`: {}", field_ident, e),
                )
            })?;
            encode_stmts.push(stmt);

            let decode_expr = decode_expr_for_type(field_ty, f).map_err(|e| {
                syn::Error::new(
                    field_ident.span(),
                    format!("decode error on field `{}`: {}", field_ident, e),
                )
            })?;
            decode_stmts.push(quote! {
                let #field_ident = #decode_expr;
            });
            field_inits.push(quote! { #field_ident });
        }
    }

    Ok(quote! {
        impl #ident {
            pub const __BEN_SCHEMA_TABLE: &'static str        = #fingerprint_lit;
            pub const __BEN_SCHEMA_VERSION: u32               = #version;
            pub const __BEN_SCHEMA_FINGERPRINT: &'static str  = #fingerprint_lit;
            pub const __BEN_SCHEMA_DDL: &'static str          = #ddl_lit;
            pub const __BEN_SCHEMA_JSON: &'static str         = #json_lit;
            pub const __BEN_SCHEMA_EVT_HASH: [u8; 32]         = #evt_hash_tokens;
            pub const __BEN_SCHEMA_FIELD_COUNT: u16           = #field_count_u16;
        }

        impl ::ben_contracts::schema::BenSchema for #ident {
            const __BEN_SCHEMA_TABLE: &'static str       = #fingerprint_lit;
            const __BEN_SCHEMA_VERSION: u32              = #version;
            const __BEN_SCHEMA_FINGERPRINT: &'static str = #fingerprint_lit;
            const __BEN_SCHEMA_DDL: &'static str         = #ddl_lit;
            const __BEN_SCHEMA_JSON: &'static str        = #json_lit;
        }

        pub fn schema_manifest() -> ::ben_contracts::schema::SchemaManifest {
            ::ben_contracts::schema::SchemaManifest {
                tables: vec![
                    ::ben_contracts::schema::TableDef {
                        name: #fingerprint_lit.to_string(),
                        ddl: #ddl_lit.to_string(),
                        columns: vec![
                            #(#column_defs),*
                        ],
                    },
                ],
            }
        }

        impl ::ben_wire::rowbinary::RowBinaryEncode for #ident {
            fn encode_rowbinary(&self, out: &mut ::std::vec::Vec<u8>) -> ::ben_wire::rowbinary::RowBinaryResult {
                #( #encode_stmts )*
                Ok(())
            }
        }

        impl ::ben_wire::rowbinary::RowBinaryDecode for #ident {
            const EVT_HASH: [u8; 32] = #evt_hash_tokens;
            const FIELD_COUNT: u16 = #field_count_u16;

            fn from_rowbinary(cur: &mut ::ben_wire::rowbinary::RowBinCursor<'_>) -> ::anyhow::Result<Self> {
                #(#decode_stmts)*
                Ok(Self {
                    #(#field_inits),*
                })
            }
        }

        impl ::ben_wire::rowbinary::EncodeQuic for #ident {
            const EVT_HASH: [u8; 32] = #evt_hash_tokens;
            const FIELD_COUNT: u16   = #field_count_u16;
        }

        impl ::ben_wire::rowbinary::DecodeQuic for #ident {
            const EVT_HASH: [u8; 32] = #evt_hash_tokens;
            const FIELD_COUNT: u16   = #field_count_u16;
        }
    })
}

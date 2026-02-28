// use crate::schema::parse::parse_enum_pairs;

use heck::ToSnakeCase;
use quote::ToTokens;

use darling::{FromDeriveInput, FromField};
use syn::{Ident, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(bschema), supports(struct_named))]
pub struct SchemaSpec {
    #[darling(default)]
    pub table: String,
    #[darling(default)]
    pub version: u32,
    #[darling(default)]
    pub order_by: Option<String>,
    #[darling(default)]
    pub partition_by: Option<String>,
    #[darling(default)]
    pub engine: Option<String>,
    #[darling(default)]
    pub compression: Option<String>,
    #[darling(default)]
    pub description: Option<String>,
    #[darling(default)]
    pub store: Option<bool>,

    pub ident: Ident,
    pub data: darling::ast::Data<(), SchemaField>,
}

#[derive(Debug, FromField, Clone)]
#[darling(attributes(bschema))]
pub struct SchemaField {
    pub ident: Option<Ident>,
    pub ty: Type,

    // flags
    #[darling(default)]
    pub key: bool,
    #[darling(default)]
    pub unique: bool,
    #[darling(default)]
    pub tags: bool,
    #[darling(default)]
    pub ben_meta: bool,
    #[darling(default)]
    pub nullable: bool,

    #[darling(default)]
    pub enum_type: Option<String>,
    #[darling(default)]
    pub cardinality: Option<String>,
    #[darling(default)]
    pub default: Option<String>,
    #[darling(default)]
    pub ttl: Option<String>,
}

pub fn validate(spec: &SchemaSpec) -> syn::Result<()> {
    // table name: snake_case, sane length
    if spec.table.is_empty() {
        return err(spec, "bschema(table = \"...\") is required");
    }
    let t = spec.table.to_snake_case();
    if t != spec.table || t.len() < 3 || t.len() > 48 {
        return err(spec, "table must be snake_case, 3..=48 chars");
    }

    // keys
    let mut key_count = 0usize;
    for f in fields(spec) {
        if f.key {
            key_count += 1;
        }
        // illegal combos
        if f.key && (f.tags || f.ben_meta) {
            return syn::Result::Err(syn::Error::new(
                span(f),
                "#[bschema(key)] cannot be used on tags/ben_meta maps",
            ));
        }
        // tags/ben_meta must be HashMap<String,String>
        if (f.tags || f.ben_meta) && !is_string_map(&f.ty) {
            return syn::Result::Err(syn::Error::new(
                span(f),
                "tags/ben_meta require HashMap<String,String>",
            ));
        }
        // enum mapping must be well-formed if provided
        if let Some(ref r#type) = f.enum_type {}
    }
    if key_count == 0 {
        return err(spec, "at least one #[bschema(key)] is required");
    }

    // order_by references must exist (if provided)
    if let Some(ref ob) = spec.order_by {
        for name in ob.split(',').map(|s| s.trim()) {
            if !fields(spec).iter().any(|f| ident_name(f) == name) {
                return err(spec, &format!("order_by references unknown field '{name}'"));
            }
        }
    }
    Ok(())
}

fn fields(spec: &SchemaSpec) -> Vec<&SchemaField> {
    match &spec.data {
        darling::ast::Data::Struct(fields) => fields.iter().collect(),
        _ => vec![],
    }
}
fn span(f: &SchemaField) -> proc_macro2::Span {
    f.ident
        .as_ref()
        .map(|i| i.span())
        .unwrap_or(proc_macro2::Span::call_site())
}
fn ident_name(f: &SchemaField) -> String {
    f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default()
}

fn err(spec: &SchemaSpec, msg: &str) -> syn::Result<()> {
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("BenSchema: {msg} (table {}", spec.table),
    ))
}

fn is_string_map(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };

    if segment.ident != "HashMap" {
        return false;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    if args.args.len() < 2 {
        return false;
    }

    let Some(syn::GenericArgument::Type(syn::Type::Path(key_ty))) = args.args.first() else {
        return false;
    };
    if !key_ty.path.is_ident("String") {
        return false;
    }

    let Some(syn::GenericArgument::Type(syn::Type::Path(val_ty))) = args.args.get(1) else {
        return false;
    };
    if !val_ty.path.is_ident("String") {
        return false;
    }

    true
}

// pub fn valid_label(s: &str) -> bool {
//     s.chars()
//         .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '.')
// }

// fn parse_i16_liberal(s: &str) -> Option<i16> {
//     if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
//         i16::from_str_radix(hex, 16).ok()
//     } else {
//         s.parse::<i16>().ok()
//     }
// }

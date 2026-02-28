use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Expr, ExprCall, ExprPath, Ident, LitStr, Path, spanned::Spanned};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct BlotInput {
    ident: Ident,
    data: darling::ast::Data<(), BlotFieldAttr>,
}

#[derive(Debug, FromField)]
#[darling(attributes(blotspec))]
struct BlotFieldAttr {
    ident: Option<Ident>,

    #[darling(default)]
    rule: Option<String>,

    #[darling(default)]
    op: Option<Expr>,

    #[darling(default)]
    note: Option<String>,
}

pub fn derive_blot(input: DeriveInput) -> syn::Result<TokenStream2> {
    let spec = BlotInput::from_derive_input(&input)
        .map_err(|e| syn::Error::new_spanned(&input, e.to_string()))?;

    let ident = spec.ident;

    let syn_fields: Vec<&syn::Field> = match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(named) => named.named.iter().collect(),
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Blot only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Blot only supports structs with named fields",
            ));
        }
    };

    let fields = match spec.data {
        darling::ast::Data::Struct(fields) => fields.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &ident,
                "Blot only supports structs with named fields",
            ));
        }
    };

    let mut rules_tokens = Vec::new();

    for (idx, field_attr) in fields.into_iter().enumerate() {
        let syn_field = syn_fields
            .get(idx)
            .ok_or_else(|| syn::Error::new_spanned(&ident, "field index out of bounds"))?;

        // Does this field actually have #[blotspec(...)]?
        let has_blot_attr = syn_field
            .attrs
            .iter()
            .any(|a| a.path().is_ident("blotspec"));

        if !has_blot_attr {
            continue;
        }

        let field_ident = field_attr.ident.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Unnamed field not supported for Blot",
            )
        })?;
        let field_name_str = field_ident.to_string();

        let rule_tokens = if let Some(ref rule) = field_attr.rule {
            let lit = LitStr::new(rule, field_ident.span());
            quote! { Some(#lit) }
        } else {
            quote! { None }
        };

        let note_tokens = if let Some(ref note) = field_attr.note {
            let lit = LitStr::new(note, field_ident.span());
            quote! { Some(#lit) }
        } else {
            quote! { None }
        };

        let op_expr = field_attr.op.ok_or_else(|| {
            syn::Error::new(field_ident.span(), "missing `op = ...` in #[blotspec(...)]")
        })?;
        let op_tokens = parse_blot_op(&op_expr)?;

        rules_tokens.push(quote! {
            ::ben_contracts::BlotFieldRule {
                field: #field_name_str,
                rule: #rule_tokens,
                op: #op_tokens,
                note: #note_tokens,
            }
        });
    }

    let expanded = quote! {
        impl ::ben_contracts::BlotSpec for #ident {
            const BLOT_RULES: &'static [::ben_contracts::BlotFieldRule] = &[
                #( #rules_tokens ),*
            ];
        }
    };

    Ok(expanded)
}

fn parse_blot_op(expr: &syn::Expr) -> syn::Result<TokenStream2> {
    match expr {
        syn::Expr::Path(p) => {
            let ident = p.path.segments.last().unwrap().ident.to_string();
            match ident.as_str() {
                "mask_all" => Ok(quote! { ::ben_contracts::BlotOp::MaskAll }),
                "hash_sha256" => Ok(quote! { ::ben_contracts::BlotOp::HashSha256 }),
                "drop" => Ok(quote! { ::ben_contracts::BlotOp::Drop }),
                op => Err(syn::Error::new_spanned(
                    p,
                    format!("Unknown Blot operation: {op}"),
                )),
            }
        }

        syn::Expr::Call(call) => {
            let func = match &*call.func {
                syn::Expr::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                other => return Err(syn::Error::new_spanned(other, "invalid blot operation")),
            };

            if func == "mask_suffix" {
                let arg = &call.args[0];
                Ok(quote! { ::ben_contracts::BlotOp::MaskSuffix { keep: (#arg) } })
            } else if func == "mask_prefix" {
                let arg = &call.args[0];
                Ok(quote! { ::ben_contracts::BlotOp::MaskPrefix { keep: (#arg) } })
            } else if func == "truncate" {
                let arg = &call.args[0];
                Ok(quote! { ::ben_contracts::BlotOp::Truncate { len: (#arg) } })
            } else {
                Err(syn::Error::new_spanned(
                    &call.func,
                    "unknown blot op() function",
                ))
            }
        }

        other => Err(syn::Error::new_spanned(other, "invalid blot op expression")),
    }
}

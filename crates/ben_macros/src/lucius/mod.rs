use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Ident, LitStr};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct LspecInput {
    ident: Ident,
    data: darling::ast::Data<(), LspecFieldAttr>,
}

#[derive(Debug, FromField)]
#[darling(attributes(lspec))]
struct LspecFieldAttr {
    ident: Option<Ident>,

    #[darling(default)]
    rule: Option<String>,

    #[darling(default)]
    level: Option<String>,

    #[darling(default)]
    expected_type: Option<String>,

    #[darling(default)]
    route: Option<String>,

    #[darling(default)]
    note: Option<String>,

    #[darling(default)]
    has_lspec: bool,
}

pub fn derive_lucius(input: DeriveInput) -> syn::Result<TokenStream2> {
    let spec = LspecInput::from_derive_input(&input)
        .map_err(|e| syn::Error::new_spanned(&input, e.to_string()))?;

    let ident = spec.ident;

    let mut specs_tokens = Vec::new();

    let fields = match spec.data {
        darling::ast::Data::Struct(fields) => fields.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "Lucius only supports structs with named fields",
            ));
        }
    };

    let syn_fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "Lucius only supports structs",
            ));
        }
    };

    for field in fields {
        let field_ident_ref = field.ident.as_ref();
        let had_lspec_attr = field_ident_ref
            .and_then(|ident| syn_fields.iter().find(|f| f.ident.as_ref() == Some(ident)))
            .map(|f| f.attrs.iter().any(|a| a.path().is_ident("lspec")))
            .unwrap_or(false);

        if !had_lspec_attr {
            continue;
        }

        let field_ident = field.ident.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Unnamed field not supported for Lucius",
            )
        })?;

        let field_name_str = field_ident.to_string();

        let rule_tokens = if let Some(ref rule) = field.rule {
            let lit = LitStr::new(rule, field_ident.span());
            quote! { Some(#lit) }
        } else {
            quote! { None }
        };

        let level_tokens = {
            let lvl = field.level.clone().unwrap_or_else(|| "light".to_string());
            match lvl.as_str() {
                "light" => quote! { ::ben_contracts::LuciusLevel::Light },
                "static" | "static_deep" => quote! { ::ben_contracts::LuciusLevel::StaticDeep },
                "dynamic" | "dynamic_sandbox" => {
                    quote! { ::ben_contracts::LuciusLevel::DynamicSandbox }
                }
                "hybrid" | "full" => quote! { ::ben_contracts::LuciusLevel::HybridFull },
                other => {
                    return Err(syn::Error::new(
                        field_ident.span(),
                        format!(
                            "Unknown lucius(level = \"{other}\"); expected one of: light|static|dynamic|hybrid"
                        ),
                    ));
                }
            }
        };

        let expected_lit = {
            let v = field
                .expected_type
                .clone()
                .unwrap_or_else(|| "bin".to_string());
            LitStr::new(&v, field_ident.span())
        };

        let route_lit = {
            let v = field
                .route
                .clone()
                .unwrap_or_else(|| field_name_str.clone());
            LitStr::new(&v, field_ident.span())
        };

        let note_tokens = if let Some(ref note) = field.note {
            let lit = LitStr::new(note, field_ident.span());
            quote! { Some(#lit) }
        } else {
            quote! { None }
        };

        specs_tokens.push(quote! {
            ::ben_contracts::LuciusFieldSpec {
                field: #field_name_str,
                rule: #rule_tokens,
                level: #level_tokens,
                expected_type: #expected_lit,
                route: #route_lit,
                note: #note_tokens,
            }
        });
    }

    let expanded = quote! {
        impl ::ben_contracts::LuciusSpec for #ident {
            const LUCIUS_SPECS: &'static [::ben_contracts::LuciusFieldSpec] = &[
                #( #specs_tokens ),*
            ];
        }
    };

    Ok(expanded)
}

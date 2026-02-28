use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use serde::Serialize;
use syn::{Data, DeriveInput, Fields, Lit, spanned::Spanned};

struct EnumVariant {
    label: String,

    value: i16,

    span: Span,

    ident: syn::Ident,
}

#[derive(Serialize)]
struct JsonEnumVariant {
    label: String,
    value: i16,
}

#[derive(Serialize)]
struct JsonEnumDescriptor {
    bits: u8,
    variants: Vec<JsonEnumVariant>,
}

pub fn derive_ben_enum(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &input.ident;

    let variants = match &input.data {
        Data::Enum(en) => &en.variants,
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "BenEnum may only be derived on enums",
            ));
        }
    };

    let mut collected: Vec<EnumVariant> = Vec::new();
    let mut last_value: Option<i16> = None;

    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "BenEnum only supports unit-like variants",
            ));
        }

        let mut label_override: Option<String> = None;
        let mut explicit_value: Option<i16> = None;

        for attr in &variant.attrs {
            if !attr.path().is_ident("benum") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("label") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    label_override = Some(lit.value());
                    return Ok(());
                }

                if meta.path.is_ident("value") {
                    let lit: Lit = meta.value()?.parse()?;
                    let lit_span = lit.span();

                    explicit_value = match lit {
                        Lit::Int(v) => Some(v.base10_parse::<i16>()?),
                        Lit::Str(s) => {
                            let raw = s.value();
                            if let Some(hex) =
                                raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"))
                            {
                                i16::from_str_radix(hex, 16).ok()
                            } else {
                                raw.parse::<i16>().ok()
                            }
                        }
                        other => {
                            return Err(syn::Error::new(
                                other.span(),
                                "value must be integer or string like \"0xFF\"",
                            ));
                        }
                    };

                    if explicit_value.is_none() {
                        return Err(syn::Error::new(
                            lit_span,
                            "invalid enum value; must fit i16",
                        ));
                    }

                    return Ok(());
                }

                Ok(())
            })?;
        }

        let label = label_override.unwrap_or_else(|| variant.ident.to_string());

        let numeric_value = match explicit_value {
            Some(v) => v,
            None => match last_value {
                Some(prev) => prev
                    .checked_add(1)
                    .ok_or_else(|| syn::Error::new(variant.span(), "i16 overflow"))?,
                None => 0,
            },
        };

        last_value = Some(numeric_value);

        collected.push(EnumVariant {
            label,
            value: numeric_value,
            span: variant.span(),
            ident: variant.ident.clone(),
        });
    }

    if collected.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "enum has no variants for BenEnum",
        ));
    }

    use std::collections::HashSet;

    let mut labels = HashSet::new();
    let mut values = HashSet::new();

    for v in &collected {
        if !labels.insert(&v.label) {
            return Err(syn::Error::new(
                v.span,
                format!("duplicate enum label '{}'", v.label),
            ));
        }
        if !values.insert(v.value) {
            return Err(syn::Error::new(
                v.span,
                format!("duplicate enum numeric value {}", v.value),
            ));
        }
    }

    let needs_16 = collected.iter().any(|v| v.value < -128 || v.value > 127);
    let bits: u8 = if needs_16 { 16 } else { 8 };

    let variant_idents: Vec<_> = collected.iter().map(|v| &v.ident).collect();
    let labels_lit: Vec<_> = collected
        .iter()
        .map(|v| syn::LitStr::new(&v.label, Span::call_site()))
        .collect();
    let values_lit: Vec<_> = collected
        .iter()
        .map(|v| syn::LitInt::new(&v.value.to_string(), Span::call_site()))
        .collect();

    let reg_ident = quote::format_ident!("{}_ENUM_META", ident);

    Ok(quote! {




        #[linkme::distributed_slice(::ben_contracts::enum_info::BEN_ENUM_REGISTRY)]
        pub static #reg_ident: fn() -> &'static ::ben_contracts::enum_info::EnumInfo = || {
            & #ident :: ENUM_INFO
        };


        impl #ident {

            pub const ENUM_INFO: ::ben_contracts::enum_info::EnumInfo =
                ::ben_contracts::enum_info::EnumInfo {
                    name: stringify!(#ident),
                    bits: #bits,
                    variants: &[
                        #(
                            ::ben_contracts::enum_info::EnumVariant {
                                label: #labels_lit,
                                value: #values_lit,
                            },
                        )*
                    ],
                };

            pub const __BEN_ENUM_BITS: u8 = #bits;
            pub const __BEN_ENUM_VARIANTS: &'static [::ben_contracts::enum_info::EnumVariant] = &[
                #(
                    ::ben_contracts::enum_info::EnumVariant {
                        label: #labels_lit,
                        value: #values_lit,
                    },
                )*
            ];

            #[inline]
            pub fn __ben_enum_to_i16(&self) -> i16 {
                match self {
                    #( Self::#variant_idents => #values_lit, )*
                }
            }

            #[inline]
            pub fn __ben_enum_from_i16(v: i16) -> Option<Self> {
                match v {
                    #( #values_lit => Some(Self::#variant_idents), )*
                    _ => None,
                }
            }
        }

        impl ::ben_contracts::enum_info::BenEnumInfo for #ident {
            fn variants() -> &'static [::ben_contracts::enum_info::EnumVariant] {
                &[
                    #(
                        ::ben_contracts::enum_info::EnumVariant {
                            label: #labels_lit,
                            value: #values_lit,
                        }
                    ),*
                ]
            }
        }

        impl ::core::convert::From<#ident> for i16 {
            #[inline]
            fn from(e: #ident) -> i16 { e.__ben_enum_to_i16() }
        }

        impl ::core::convert::TryFrom<i16> for #ident {
            type Error = ();
            #[inline]
            fn try_from(v: i16) -> Result<Self, ()> {
                Self::__ben_enum_from_i16(v).ok_or(())
            }
        }

        impl ::ben_contracts::BenEnumDesc for #ident {
            const BITS: u8 = #bits;
            const VARIANTS: &'static [::ben_contracts::enum_info::EnumVariant] =  &[
                #(
                    ::ben_contracts::enum_info::EnumVariant {
                        label: #labels_lit,
                        value: #values_lit,
                    },
                )*
            ];

            fn to_i16(self) -> i16 { self.__ben_enum_to_i16() }
            fn from_i16(v: i16) -> Option<Self> { Self::__ben_enum_from_i16(v) }
        }

        impl ::core::str::FromStr for #ident {
            type Err = ();
            #[inline]
            fn from_str(s: &str) -> Result<Self, ()> {
                match s {
                    #( #labels_lit => Ok(Self::#variant_idents), )*
                    _ => Err(()),
                }
            }
        }
    })
}

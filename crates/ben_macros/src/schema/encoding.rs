use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Type, spanned::Spanned};

use crate::schema::{
    helpers::{hashmap_key_value, is_ben_enum, option_inner_ty, vec_inner_ty},
    validate::SchemaField,
};

pub fn encode_inner_element(ty: &Type, var_name: &str) -> syn::Result<TokenStream2> {
    use syn::{TypeArray, TypePath};
    let var_ident = quote::format_ident!("{}", var_name);

    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            let name = seg.ident.to_string();
            // Note: `var_ident` is a reference (&T). We must dereference for Copy types.
            match name.as_str() {
                "u8" => return Ok(quote!( out.push(*#var_ident); )),
                "u64" | "i64" | "u32" | "u128" => {
                    return Ok(quote!( out.extend_from_slice(&(*#var_ident).to_le_bytes()); ));
                }
                "f64" => {
                    return Ok(
                        quote!( out.extend_from_slice(&(*#var_ident).to_bits().to_le_bytes()); ),
                    );
                }
                "bool" => return Ok(quote!( out.push(*#var_ident as u8); )),
                "String" | "&str" => {
                    return Ok(quote!({
                        let bytes = #var_ident.as_bytes();
                        let len = bytes.len() as u32;
                        out.extend_from_slice(&len.to_le_bytes());
                        out.extend_from_slice(bytes);
                    }));
                }

                _ => {}
            };
        }
    }

    if is_ben_enum(ty) {
        return Ok(quote! {
            out.extend_from_slice(&#var_ident.__ben_enum_to_i16().to_le_bytes());
        });
    }

    if let Type::Array(TypeArray { elem, .. }) = ty {
        if let Type::Path(TypePath { path, .. }) = elem.as_ref() {
            if path.is_ident("u8") {
                return Ok(quote!( out.extend_from_slice(#var_ident); ));
            }
        }
    }

    Err(syn::Error::new(
        ty.span(),
        "BenSchema inner: unsupported element type",
    ))
}

pub fn encode_stmt_for_field(
    field_ident: &syn::Ident,
    ty: &Type,
    f: &crate::schema::validate::SchemaField,
) -> syn::Result<TokenStream2> {
    use syn::{TypeArray, TypePath};

    if let Some(ref et) = f.enum_type {
        return match et.as_str() {
            "string" => Ok(quote! {
                {
                    let s = self.#field_ident.as_str();
                    let bytes = s.as_bytes();
                    let len = bytes.len() as u32;
                    out.extend_from_slice(&len.to_le_bytes());
                    out.extend_from_slice(bytes);
                }
            }),
            "u64" => Ok(quote! {
                {
                    let v = self.#field_ident.as_u64();
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }),
            "i64" => Ok(quote! {
                {
                    let v = self.#field_ident.as_i64();
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }),
            "f64" => Ok(quote! {
                {
                    let v = self.#field_ident.as_f64();
                    out.extend_from_slice(&v.to_bits().to_le_bytes());
                }
            }),
            other => Err(syn::Error::new(
                ty.span(),
                format!("BenSchema: unsupported enum underlying type '{}'", other),
            )),
        };
    }

    if let Some(inner_ty) = option_inner_ty(ty) {
        let inner_stmt = encode_inner_element(inner_ty, "elem")?;
        return Ok(quote! {
            if let Some(ref elem) = self.#field_ident {
                out.push(1); // Not Null
                #inner_stmt
            } else {
                out.push(0); // Null
            }
        });
    }

    if let Some(inner_ty) = vec_inner_ty(ty) {
        let encode_elem = encode_inner_element(inner_ty, "elem")?;
        return Ok(quote! {
            {
                let items = &self.#field_ident;
                let len = items.len() as u64;
                out.extend_from_slice(&len.to_le_bytes());
                for elem in items {
                    #encode_elem
                }
            }
        });
    }

    if let Some((key_ty, val_ty)) = hashmap_key_value(ty) {
        let key_enc = encode_inner_element(key_ty, "k")?;
        let val_enc = encode_inner_element(val_ty, "v")?;
        return Ok(quote! {
            {
                let items = &self.#field_ident;
                let len = items.len() as u64;
                out.extend_from_slice(&len.to_le_bytes());
                for (k, v) in items {
                    #key_enc
                    #val_enc
                }
            }
        });
    }

    if is_ben_enum(ty) {
        return Ok(quote! {
            out.extend_from_slice(&self.#field_ident.__ben_enum_to_i16().to_le_bytes());
        });
    }

    if let Type::Array(TypeArray { elem, .. }) = ty {
        if let Type::Path(TypePath { path, .. }) = elem.as_ref() {
            if path.is_ident("u8") {
                return Ok(quote!( out.extend_from_slice(&self.#field_ident); ));
            }
        }
    }

    let Type::Path(TypePath { path, .. }) = ty else {
        return Err(syn::Error::new(
            ty.span(),
            "BenSchema: unsupported type structure",
        ));
    };

    let Some(seg) = path.segments.last() else {
        return Err(syn::Error::new(ty.span(), "BenSchema: empty type path"));
    };

    let name = seg.ident.to_string();

    Ok(match name.as_str() {
        "u64" | "i64" | "u32" | "u128" => {
            quote! { out.extend_from_slice(&self.#field_ident.to_le_bytes()); }
        }
        "f64" => quote! { out.extend_from_slice(&self.#field_ident.to_bits().to_le_bytes()); },
        "bool" => quote! { out.push(self.#field_ident as u8); },
        "String" | "&str" => quote! {
            let bytes = self.#field_ident.as_bytes();
            let len: u32 = bytes.len() as u32;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(bytes);
        },
        _ => {
            return Err(syn::Error::new(
                ty.span(),
                format!("BenSchema: unsupported type '{}'", name),
            ));
        }
    })
}

pub fn decode_expr_for_type(ty: &Type, f: &SchemaField) -> syn::Result<TokenStream2> {
    if let Some(inner_ty) = option_inner_ty(ty) {
        let inner_expr = decode_expr_for_type(inner_ty, f)?;
        return Ok(quote! {
            {
                if cur.read_u8()? == 1 {
                    Some(#inner_expr)
                } else {
                    None
                }
            }
        });
    }

    if let Some(inner_ty) = vec_inner_ty(ty) {
        let inner_decode = decode_expr_for_type(inner_ty, f)?;
        return Ok(quote! {
            {
                let len = cur.read_u64()?;
                let mut vec = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let val = #inner_decode;
                    vec.push(val);
                }
                vec
            }
        });
    }

    if let Some((key_ty, val_ty)) = hashmap_key_value(ty) {
        let key_expr = decode_expr_for_type(key_ty, f)?;
        let val_expr = decode_expr_for_type(val_ty, f)?;
        return Ok(quote! {
            {
                let len = cur.read_u64()?;
                let mut map = ::std::collections::HashMap::with_capacity(len as usize);
                for _ in 0..len {
                    let k = #key_expr;
                    let v = #val_expr;
                    map.insert(k, v);
                }
                map
            }
        });
    }

    if let Some(ref et) = f.enum_type {
        return match et.as_str() {
            "string" => Ok(quote! {
                {
                    let raw = cur.read_string()?;
                    <#ty>::from_value_str(&raw)
                        .ok_or_else(|| ::anyhow::anyhow!("Invalid enum value '{}'", raw))?
                }
            }),
            "u64" => Ok(quote! {
                {
                    let raw = cur.read_u64()?;
                    <#ty>::from_value_u64(raw)
                        .ok_or_else(|| ::anyhow::anyhow!("Invalid enum value {}", raw))?
                }
            }),
            "i64" => Ok(quote! {
                {
                    let raw = cur.read_i64()?;
                    <#ty>::from_value_i64(raw)
                        .ok_or_else(|| ::anyhow::anyhow!("Invalid enum value {}", raw))?
                }
            }),
            "f64" => Ok(quote! {
                {
                    let raw = cur.read_f64()?;
                    <#ty>::from_value_f64(raw)
                        .ok_or_else(|| ::anyhow::anyhow!("Invalid enum value {}", raw))?
                }
            }),
            other => Err(syn::Error::new(
                ty.span(),
                format!("Unsupported enum backing type {}", other),
            )),
        };
    }

    if is_ben_enum(ty) {
        return Ok(quote! {
            {
                let val = cur.read_i16()?;
                <#ty>::__ben_enum_from_i16(val)
                    .ok_or_else(|| ::anyhow::anyhow!("Invalid enum value {}", val))?
            }
        });
    }

    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            let id = seg.ident.to_string();
            return match id.as_str() {
                "u64" => Ok(quote! { cur.read_u64()? }),
                "u32" => Ok(quote! { cur.read_u32()? }),
                "u128" => Ok(quote! { cur.read_u128()? }),
                "i64" => Ok(quote! { cur.read_i64()? }),
                "i32" => Ok(quote! { cur.read_i32()? }),
                "i16" => Ok(quote! { cur.read_i16()? }),
                "i8" => Ok(quote! { cur.read_i8()? }),
                "f64" => Ok(quote! { cur.read_f64()? }),
                "f32" => Ok(quote! { cur.read_f32()? }),
                "bool" => Ok(quote! { cur.read_bool()? }),
                "String" => Ok(quote! { cur.read_string()? }),
                "&str" => Err(syn::Error::new(
                    ty.span(),
                    "BenSchema: decode cannot produce borrowed &str fields, only owned String",
                )),
                _ => Err(syn::Error::new(
                    ty.span(),
                    format!("BenSchema: unsupported RowBinaryDecode type '{}'", id),
                )),
            };
        }
    }
    Err(syn::Error::new(
        ty.span(),
        "BenSchema: unsupported RowBinaryDecode type",
    ))
}

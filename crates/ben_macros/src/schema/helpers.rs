use syn::{Type, spanned::Spanned};

pub fn option_inner_ty(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, PathArguments, TypePath};
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };
    Some(inner)
}

pub fn vec_inner_ty(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, PathArguments, TypePath};
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "Vec" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };
    Some(inner)
}

pub fn hashmap_key_value(ty: &syn::Type) -> Option<(&syn::Type, &syn::Type)> {
    use syn::{GenericArgument, PathArguments, TypePath};
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "HashMap" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    if args.args.len() < 2 {
        return None;
    }
    let GenericArgument::Type(key_ty) = args.args.first()? else {
        return None;
    };
    let GenericArgument::Type(val_ty) = args.args.get(1)? else {
        return None;
    };
    Some((key_ty, val_ty))
}

pub fn is_primitive(ty: &Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            let id = seg.ident.to_string();
            return matches!(
                id.as_str(),
                "u8" | "u16"
                    | "u32"
                    | "u64"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "String"
                    | "&str"
            );
        }
    }
    false
}

pub fn is_ben_enum(ty: &Type) -> bool {
    // If it's a TypePath but NOT Option, Vec, HashMap, or Primitive, we assume it's a BenEnum.
    !option_inner_ty(ty).is_some()
        && !vec_inner_ty(ty).is_some()
        && !hashmap_key_value(ty).is_some()
        && !is_primitive(ty)
        && matches!(ty, Type::Path(_))
}

pub fn clickhouse_type_for(ty: &syn::Type) -> syn::Result<String> {
    use syn::TypePath;

    // Option<T> -> Nullable(T)
    if let Some(inner) = option_inner_ty(ty) {
        let inner_ty = clickhouse_type_for(inner)?;
        return Ok(format!("Nullable({inner_ty})"));
    }

    // Vec<T> -> Array(T)
    if let Some(inner) = vec_inner_ty(ty) {
        let inner_ty = clickhouse_type_for(inner)?;
        return Ok(format!("Array({inner_ty})"));
    }

    // HashMap<K,V> -> Map(K,V)
    if let Some((k, v)) = hashmap_key_value(ty) {
        let k_ty = clickhouse_type_for(k)?;
        let v_ty = clickhouse_type_for(v)?;
        return Ok(format!("Map({k_ty}, {v_ty})"));
    }

    // BenEnum -> Int16 (Implicit)
    if is_ben_enum(ty) {
        return Ok("Int16".to_string());
    }

    // Primitive types
    if let syn::Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            let id = seg.ident.to_string();
            return match id.as_str() {
                "u8" => Ok("UInt8".into()),
                "u16" => Ok("UInt16".into()),
                "u32" => Ok("UInt32".into()),
                "u64" => Ok("UInt64".into()),
                "i8" => Ok("Int8".into()),
                "i16" => Ok("Int16".into()),
                "i32" => Ok("Int32".into()),
                "i64" => Ok("Int64".into()),
                "f32" => Ok("Float32".into()),
                "f64" => Ok("Float64".into()),
                "bool" => Ok("Bool".into()),
                "String" => Ok("String".into()),
                _ => Err(syn::Error::new(
                    ty.span(),
                    format!("BenSchema: unsupported ClickHouse type '{id}'"),
                )),
            };
        }
    }

    Err(syn::Error::new(
        ty.span(),
        "BenSchema: unsupported ClickHouse type",
    ))
}

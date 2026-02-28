use super::clickhouse_type_for;
use super::validate::{SchemaField, SchemaSpec};
// use crate::schema::parse::parse_enum_pairs;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use syn::Type;

#[derive(Serialize)]
struct JsonField {
    name: String,
    rust_type: String,
    ch_type: String,
    key: bool,
    unique: bool,
    nullable: bool,
    tags: bool,
    ben_meta: bool,
    enum_map: Option<std::collections::BTreeMap<String, i16>>,
}

pub fn manifest_json(spec: &SchemaSpec) -> syn::Result<String> {
    let mut infra_cols: Vec<JsonField> = vec![];
    let mut user_cols: Vec<JsonField> = vec![];

    infra_cols.push(JsonField {
        name: "tenant_id".to_string(),
        rust_type: "Uuid".to_string(),
        ch_type: "UUID".to_string(),
        key: true,
        unique: false,
        tags: false,
        ben_meta: true,
        nullable: false,
        enum_map: None,
    });

    for f in fields(spec) {
        let mut enum_map = None;
        let ch_type = if let Some(ref et) = f.enum_type {
            map_enum_type_to_clickhouse(et)
        } else {
            clickhouse_type_for(&f.ty)?.to_string()
        };
        user_cols.push(JsonField {
            name: f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
            rust_type: type_to_string_lossy(&f.ty),
            key: f.key,
            unique: f.unique,
            tags: f.tags,
            ben_meta: f.ben_meta,
            nullable: f.nullable,
            ch_type: clickhouse_type_for(&f.ty)?.to_string(),
            enum_map,
        });
    }
    user_cols.sort_by(|a, b| a.name.cmp(&b.name)); // stable fingerprint for user columns

    let mut cols = Vec::new();
    cols.extend(infra_cols);
    cols.extend(user_cols);

    let obj = json!({
        "table": spec.table,
        "version": spec.version,
        "order_by": spec.order_by,
        "compression": spec.compression,
        "columns": cols,
        "description": spec.description,
    });

    Ok(serde_json::to_string_pretty(&obj).unwrap())
}

fn map_enum_type_to_clickhouse(et: &str) -> String {
    match et {
        "string" => "String".into(),
        "u64" => "UInt64".into(),
        "i64" => "Int64".into(),
        "u32" => "UInt32".into(),
        "i32" => "Int32".into(),
        "f64" => "Float64".into(),
        "f32" => "Float32".into(),
        "bool" => "Bool".into(),
        "datetime64" => "DateTime64".into(),
        unknown => panic!("Unknown enum underlying type: {}", unknown),
    }
}

pub fn fingerprint(json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let out = hasher.finalize();
    format!("sha256:{:x}", out)
}

/// Raw 32-byte SHA-256 of the manifest JSON, used as EVT_HASH on the wire.
pub fn evt_hash(json: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let digest = hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&digest[..]);
    out
}

pub fn ddl(spec: &SchemaSpec) -> syn::Result<String> {
    let mut cols = Vec::new();

    for f in fields(spec) {
        let name = f
            .ident
            .as_ref()
            .expect("Field in named struct must have an ident")
            .to_string();

        let ch_ty = if let Some(ref et) = f.enum_type {
            map_enum_type_to_clickhouse(et)
        } else {
            clickhouse_type_for(&f.ty)?
        };

        cols.push(format!("  `{}` {}", name, ch_ty));
    }

    cols.push("  `system_mask` UInt64".to_string());
    cols.push("  `struct_mask` UInt64".to_string());
    cols.push("  `field_mask` UInt64".to_string());

    let cols_joined = cols.join(",\n");

    let engine = spec.engine.as_deref().unwrap_or("MergeTree");

    let order = spec
        .order_by
        .clone()
        .unwrap_or_else(|| key_field_list(spec));

    let mut ddl_parts = vec![
        format!("CREATE TABLE IF NOT EXISTS {}", spec.table),
        "(".to_string(),
        cols_joined,
        ")".to_string(),
        format!("ENGINE = {}", engine),
        format!("ORDER BY ({})", order),
    ];

    if let Some(p) = &spec.partition_by {
        ddl_parts.push(format!("PARTITION BY {}", p));
    }

    ddl_parts.push(";".to_string());

    Ok(ddl_parts.join("\n"))
}

fn fields(spec: &SchemaSpec) -> Vec<&SchemaField> {
    match &spec.data {
        darling::ast::Data::Struct(fields) => fields.iter().collect(),
        _ => vec![],
    }
}

fn key_field_list(spec: &SchemaSpec) -> String {
    let mut out = vec![];
    for f in fields(spec) {
        if f.key {
            out.push(f.ident.as_ref().unwrap().to_string());
        }
    }
    out.join(", ")
}

fn type_to_string_lossy(ty: &Type) -> String {
    quote::quote!(#ty)
        .to_string()
        .replace(" ", "")
        .replace(" :: ", "::")
}

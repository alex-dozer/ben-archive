use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SchemaManifest {
    pub database: String,
    pub tables: Vec<TableEntry>,
    pub version: String,
    pub fingerprint: String,
}

#[derive(Debug, Deserialize)]
pub struct TableEntry {
    pub table: String,
    pub ddl: String,
    pub json: serde_json::Value,
    pub version: String,
    pub fingerprint: String,
}

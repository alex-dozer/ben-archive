#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Schema {
    pub event: String,
    pub version: u32,
    pub evt_hash: [u8; 32],
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
    pub nullable: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FieldType {
    UInt64,
    Int64,
    Float64,
    Bool,
    String,
    IPv4,
    IPv6,
    DateTime64 { scale: u32 },
    Uuid,
    Str,
}

impl Schema {
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.iter().map(|f| f.name.as_str()).collect()
    }
    pub fn fields_len(&self) -> usize {
        self.fields.len()
    }
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|n| *n.name == *name)
    }
}

use std::{borrow::Cow, marker::PhantomData};

pub trait BenSchema {
    const __BEN_SCHEMA_TABLE: &'static str;
    const __BEN_SCHEMA_VERSION: u32;
    const __BEN_SCHEMA_FINGERPRINT: &'static str;
    const __BEN_SCHEMA_DDL: &'static str;
    const __BEN_SCHEMA_JSON: &'static str;
}

pub trait BenInstallable {
    fn table(&self) -> Cow<'static, str>;
    fn version(&self) -> u32;
    fn fingerprint(&self) -> Cow<'static, str>;
    fn ddl(&self) -> Cow<'static, str>;
    fn json(&self) -> Cow<'static, str>;
}

// // Zero-sized wrapper that reads the __BEN_SCHEMA_* consts from T.
#[derive(Debug, Clone)]
pub struct Schema<T>(PhantomData<T>);
impl<T> Default for Schema<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: BenSchema> BenInstallable for Schema<T>
where
    Schema<T>: BenSchema,
{
    fn table(&self) -> Cow<'static, str> {
        Cow::Borrowed(T::__BEN_SCHEMA_TABLE)
    }
    fn version(&self) -> u32 {
        T::__BEN_SCHEMA_VERSION
    }
    fn fingerprint(&self) -> Cow<'static, str> {
        Cow::Borrowed(T::__BEN_SCHEMA_FINGERPRINT)
    }
    fn ddl(&self) -> Cow<'static, str> {
        Cow::Borrowed(T::__BEN_SCHEMA_DDL)
    }
    fn json(&self) -> Cow<'static, str> {
        Cow::Borrowed(T::__BEN_SCHEMA_JSON)
    }
}

#[macro_export]
macro_rules! ben_installable {
    ($ty:ty) => {
        impl $crate::schema::BenInstallable for $ty {
            fn table(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty>::__BEN_SCHEMA_TABLE)
            }
            fn version(&self) -> u32 {
                <$ty>::__BEN_SCHEMA_VERSION
            }
            fn fingerprint(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty>::__BEN_SCHEMA_FINGERPRINT)
            }
            fn ddl(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty>::__BEN_SCHEMA_DDL)
            }
            fn json(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(<$ty>::__BEN_SCHEMA_JSON)
            }
        }
    };
}

// Inventory entry = a pointer to a &'static dyn BenInstallable.
// #[derive(Debug, Clone)]
// pub struct SchemaEntry {
//     pub get: fn() -> &'static dyn BenInstallable,
// }

#[macro_export]
macro_rules! register_schema {
    ($ty:ty) => {
        // One static per type; zero-size, no init cost.
        static __SCHEMA_INST: $crate::schema::Schema<$ty> = $crate::schema::Schema::<$ty>(::core::marker::PhantomData);
        $crate::schema::inventory::submit! {
            $crate::schema::SchemaEntry { get: || { &__SCHEMA_INST as &'static dyn $crate::schema::BenInstallable } }
        }
    };
}

#[derive(Debug, Clone)]
pub struct SchemaManifest {
    pub tables: Vec<TableDef>,
}

#[derive(Debug, Clone)]
pub struct TableDef {
    pub name: String,
    pub ddl: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Default, Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub ch_type: String,
    pub nullable: bool,
    pub default: Option<String>,
}

pub fn merge_manifests(list: &[SchemaManifest]) -> SchemaManifest {
    let mut out = SchemaManifest { tables: vec![] };
    for m in list {
        out.tables.extend(m.tables.clone());
    }
    out
}

#[derive(Debug, Clone)]
pub struct AgentSchema {
    pub agent_id: String,
    pub agent_descriptors: Vec<AgentSchemaDescriptor>,
}

#[derive(Debug, Clone, Copy)]
pub struct AgentSchemaDescriptor {
    /// Stable schema identity (table id)
    pub fingerprint: [u8; 32],

    pub evt_hash: [u8; 32],

    /// Number of fields expected in the flattened payload
    pub field_count: u16,
}

use ben_contracts::BEN_ENUM_REGISTRY;
use ben_contracts::enum_info::EnumInfo;
use ben_contracts::schema::{Schema, SchemaManifest};
use ben_macros::{BenEnum, BenSchema};
use ben_wire::rowbinary::{EncodeQuic, RowBinaryEncode};

use std::collections::HashMap;

//
// First: define an enum using BenEnum
//
#[derive(Debug)]
pub enum DangerLevel {
    Low = 2,

    Medium = 3,

    High = 6, // auto = 6
}

//
// Next: define a schema-annotated struct using many features
//
#[derive(Debug, BenSchema)]
#[bschema(
    table = "super_event",
    version = 9,
    order_by = "id, ts",
    partition_by = "toYYYYMM(toDateTime(ts/1000))",
    description = "Maximal schema test"
)]
struct SuperEvent {
    #[bschema(key)]
    id: u64,

    #[bschema(key, cardinality = "low")]
    kind: String,

    #[bschema(nullable)]
    note: Option<String>,

    values: Vec<i64>,

    info: HashMap<String, String>,

    // #[bschema(enum_type = "u64")]
    // danger: DangerLevel,
    ts: u64,

    flag: bool,
}

//
// MAIN SUPER TEST
//
#[test]
fn super_schema_test() {
    //
    // --- 1. Validate schema consts ---
    //

    //assert_eq!(SuperEvent::__BEN_SCHEMA_TABLE, "super_event");
    assert_eq!(SuperEvent::__BEN_SCHEMA_VERSION, 9);
    assert!(SuperEvent::__BEN_SCHEMA_DDL.contains("CREATE TABLE IF NOT EXISTS super_event"));
    assert!(SuperEvent::__BEN_SCHEMA_FINGERPRINT.starts_with("sha256:"));
    assert_eq!(SuperEvent::__BEN_SCHEMA_EVT_HASH.len(), 32);

    // // We have 8 fields
    assert_eq!(SuperEvent::__BEN_SCHEMA_FIELD_COUNT, 7);

    //
    // --- 2. Validate JSON manifest content ---
    //
    let json = SuperEvent::__BEN_SCHEMA_JSON;

    for field in ["id", "kind", "note", "values", "info", "ts", "flag"] {
        assert!(json.contains(&format!("\"{field}\"")));
    }

    //Metadata
    assert!(json.contains("\"table\": \"super_event\""));
    assert!(json.contains("\"version\": 9"));
    assert!(json.contains("\"description\": \"Maximal schema test\""));

    //
    // --- 3. Validate DDL correctness ---
    //
    let ddl = SuperEvent::__BEN_SCHEMA_DDL;
    println!("{ddl}");
    assert!(ddl.contains("`id` UInt64"));
    assert!(ddl.contains("`kind` String"));
    assert!(ddl.contains("`note` Nullable(String)"));
    assert!(ddl.contains("`values` Array(Int64)"));
    assert!(ddl.contains("`info` Map(String, String)"));
    //assert!(ddl.contains("`danger` Int64"));
    assert!(ddl.contains("`ts` UInt64"));
    assert!(ddl.contains("`flag` Bool"));

    assert!(ddl.contains("ORDER BY (id, ts)"));
    assert!(ddl.contains("PARTITION BY toYYYYMM(toDateTime(ts/1000))"));

    //
    // --- 4. RowBinary encoding smoke test ---
    //
    let mut map = HashMap::new();
    map.insert("k".into(), "v".into());

    let evt = SuperEvent {
        id: 10,
        kind: "alpha".into(),
        note: Some("hello".into()),
        values: vec![1, 2, 3],
        info: map,
        //danger: DangerLevel::High,
        ts: 987654321,
        flag: true,
    };

    let mut buf = Vec::new();
    evt.encode_rowbinary(&mut buf).unwrap();
    assert!(buf.len() > 40);

    //
    // --- 5. QUIC framing correctness ---
    //
    let mut quic = Vec::new();
    evt.encode_quic(&mut quic);

    // header: 32 bytes hash + 2 bytes field count
    let expected_header_len = 32 + 2;
    assert!(quic.len() > expected_header_len);

    println!("QUIC len: {}, {}", quic.len(), expected_header_len);

    // hash is correct
    //assert_eq!(&quic[..32], &SuperEvent::__BEN_SCHEMA_EVT_HASH[..]);

    // field count matches
    let fc = u16::from_le_bytes([quic[32], quic[33]]);
    println!("Field count: {fc}");
    assert_eq!(fc, SuperEvent::__BEN_SCHEMA_FIELD_COUNT);
}

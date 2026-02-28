// use ben_contracts::schema::{Schema, SchemaManifest};
// use ben_macros::BenSchema;
// use ben_wire::rowbinary::{EncodeQuic, RowBinaryEncode, RowBinaryResult};

// //
// //
// // A simple schema struct we expect to expand cleanly
// //
// //
// #[derive(Debug, Default, BenSchema)]
// #[bschema(
//     table = "unit_test_event",
//     version = 42,
//     order_by = "id",
//     description = "Test struct for BenSchema macro"
// )]
// struct UnitTestEvent {
//     #[bschema(key)]
//     id: u64,

//     value: i64,

//     name: String,

//     flag: bool,
// }

// pub enum VesselState {
//     Moving = 10,
//     Walking = 12,
//     Hamsters = 30,
// }

// //
// //
// // Basic validation: did the macro generate all schema consts?
// //
// //
// #[test]
// fn test_schema_consts_exist() {
//     assert_eq!(UnitTestEvent::__BEN_SCHEMA_TABLE, "unit_test_event");
//     assert_eq!(UnitTestEvent::__BEN_SCHEMA_VERSION, 42);

//     assert!(UnitTestEvent::__BEN_SCHEMA_JSON.len() > 10);
//     assert!(UnitTestEvent::__BEN_SCHEMA_DDL.contains("CREATE TABLE"));
//     assert!(UnitTestEvent::__BEN_SCHEMA_FINGERPRINT.starts_with("sha256:"));

//     // EVT_HASH must be exactly 32 bytes
//     assert_eq!(UnitTestEvent::__BEN_SCHEMA_EVT_HASH.len(), 32);

//     // Our struct has four fields
//     assert_eq!(UnitTestEvent::__BEN_SCHEMA_FIELD_COUNT, 4);
// }

// //
// //
// // Validate JSON manifest shape
// //
// //
// #[test]
// fn test_manifest_json_contents() {
//     let json = UnitTestEvent::__BEN_SCHEMA_JSON;

//     // Make sure fields appear
//     assert!(json.contains("\"id\""));
//     assert!(json.contains("\"value\""));
//     assert!(json.contains("\"name\""));
//     assert!(json.contains("\"flag\""));

//     // Meta fields
//     assert!(json.contains("\"table\": \"unit_test_event\""));
//     assert!(json.contains("\"version\": 42"));
//     assert!(json.contains("\"description\": \"Test struct for BenSchema macro\""));
// }

// //
// //
// // Validate DDL output minimally
// //
// //
// #[test]
// fn test_generated_ddl() {
//     let ddl = UnitTestEvent::__BEN_SCHEMA_DDL;

//     println!("{:?}", ddl);

//     assert!(ddl.contains("CREATE TABLE IF NOT EXISTS unit_test_event"));
//     assert!(ddl.contains("`id` UInt64"));
//     assert!(ddl.contains("`value` Int64"));
//     assert!(ddl.contains("`name` String"));
//     assert!(ddl.contains("`flag` Bool")); // bool stored as UInt8 (0/1) in ClickHouse

//     assert!(ddl.contains("ORDER BY (id)"));
// }

// //
// //
// // Ensure RowBinary encoding compiles and produces bytes
// // (No decoding test here — just confirming expansion)
// //
// //
// #[test]
// fn test_rowbinary_encode_smoke() {
//     let evt = UnitTestEvent {
//         id: 555,
//         value: -123,
//         name: "hello".into(),
//         flag: true,
//     };

//     let mut buf = Vec::new();
//     evt.encode_rowbinary(&mut buf).unwrap();

//     assert!(buf.len() > 20); // definitely encoded something
// }

// //
// //
// // Ensure QUIC framing compiles and contains header + payload
// //
// //
// #[test]
// fn test_quic_encode() {
//     let evt = UnitTestEvent {
//         id: 1,
//         value: 2,
//         name: "abc".into(),
//         flag: false,
//     };

//     let mut out = Vec::new();
//     evt.encode_quic(&mut out);

//     let expected_header_len = 32 + // EVT_HASH
//         2; // FIELD_COUNT

//     assert!(out.len() > expected_header_len);

//     // Check header start = EVT_HASH
//     assert_eq!(&out[..32], &UnitTestEvent::__BEN_SCHEMA_EVT_HASH[..]);

//     // Check field count bytes
//     let fc = u16::from_le_bytes([out[32], out[33]]);
//     assert_eq!(fc, UnitTestEvent::__BEN_SCHEMA_FIELD_COUNT);
// }

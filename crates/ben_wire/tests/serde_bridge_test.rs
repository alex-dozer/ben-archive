// use ben_wire::{serde_bridge::RowpackStructDe, slot::SlotValue};
// use serde::Deserialize;

// #[derive(Debug, Deserialize, PartialEq)]
// struct LoginView {
//     user: String,
//     heat: u32,
//     ip: Option<String>, // keep it simple for now
// }

// #[test]
// fn serde_bridge_maps_row_to_struct() {
//     // Pretend schema.fields = ["user","heat","ip"]
//     let field_names = vec!["user", "heat", "ip"];

//     let row = vec![
//         SlotValue::Str("alice"),    // user
//         SlotValue::U64(200),        // heat
//         SlotValue::Str("10.1.2.3"), // ip
//     ];

//     let de = RowpackStructDe {
//         row: &row,
//         field_names: &field_names,
//     };

//     let view: LoginView = serde::Deserialize::deserialize(de).expect("deserialize LoginView");

//     assert_eq!(
//         view,
//         LoginView {
//             user: "alice".to_string(),
//             heat: 200,
//             ip: Some("10.1.2.3".to_string()),
//         }
//     );
// }

// #[test]
// fn serde_bridge_handles_missing_as_option_none() {
//     let field_names = vec!["user", "heat", "ip"];

//     let row = vec![
//         SlotValue::Str("bob"),
//         SlotValue::U64(50),
//         SlotValue::Missing, // no ip
//     ];

//     let de = RowpackStructDe {
//         row: &row,
//         field_names: &field_names,
//     };

//     let view: LoginView = serde::Deserialize::deserialize(de).expect("deserialize LoginView");

//     assert_eq!(
//         view,
//         LoginView {
//             user: "bob".to_string(),
//             heat: 50,
//             ip: None,
//         }
//     );
// }

// use ben_wire::serde_bridge::SlotValueDeserializer;
// use serde::de::DeserializeSeed;

// #[test]
// fn slotvalue_deserializer_numeric_coercions() {
//     // u64 -> u64
//     let d = SlotValueDeserializer(SlotValue::U64(42));
//     let v: u64 = serde::Deserialize::deserialize(d).expect("u64 from U64");
//     assert_eq!(v, 42);

//     // U64  u32 via Serde
//     let d = SlotValueDeserializer(SlotValue::U64(7));
//     let v: u32 = serde::Deserialize::deserialize(d).expect("u32 from U64");
//     assert_eq!(v, 7);
// }

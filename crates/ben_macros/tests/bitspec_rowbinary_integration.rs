// use ben_macros::Bitspec;
// use ben_wire::{
//     Envelope,
//     rowbinary::{RowBinCursor, RowBinaryDecode, RowBinaryEncode, RowBinaryResult},
// };
// use bitspec_engine::{RowAccess, context::BenContext, engine::BitspecEngine, policy::PolicyKey};
// use std::collections::HashMap;

// //
// // 1. The struct under test (wire + policy)
// //
// #[derive(Debug, Bitspec)]
// pub struct VesselPing {
//     #[bspec(thresholds(
//         rule = "HEAT",
//         values = "WARN=200,CRIT=600",
//         fact = "heat.level",
//         flag_if = "CRIT"
//     ))]
//     pub heat: f64,

//     #[bspec(brule(rule = "BIG_GROUP", op=eq(true)))]
//     pub big_group: bool,

//     #[bspec(brule(rule = "NO_SERVE", op = eq(0)))]
//     pub serve: f64,
// }

// // This hash must match what ben_wire will expect for this event.
// // In real BenSchema land this will be generated; for the test we hard-code it.
// impl VesselPing {
//     pub const EVT_HASH: [u8; 32] = *b"VesselPing__v1__________________";
// }

// impl RowBinaryEncode for VesselPing {
//     fn encode_rowbinary(&self, out: &mut Vec<u8>) -> RowBinaryResult {
//         out.extend_from_slice(&self.heat.to_bits().to_le_bytes());
//         out.push(self.big_group as u8);
//         out.extend_from_slice(&self.serve.to_bits().to_le_bytes());

//         Ok(())
//     }
// }

// impl RowBinaryDecode for VesselPing {
//     const EVT_HASH: [u8; 32] = Self::EVT_HASH;
//     const FIELD_COUNT: u16 = 3;

//     fn from_rowbinary(cur: &mut RowBinCursor<'_>) -> anyhow::Result<Self> {
//         let heat_bits = cur.read_u64()?;
//         let big_group = cur.read_bool()?;
//         let serve_bits = cur.read_u64()?;

//         Ok(Self {
//             heat: f64::from_bits(heat_bits),
//             big_group,
//             serve: f64::from_bits(serve_bits),
//         })
//     }
// }

// //
// // 2. DemoRow: ONLY the Bitspec view, never on the wire
// //
// #[derive(Default)]
// struct DemoRow {
//     nums: HashMap<&'static str, f64>,
//     bools: HashMap<&'static str, bool>,
// }

// impl DemoRow {
//     fn new() -> Self {
//         Self::default()
//     }
//     fn num(mut self, k: &'static str, v: f64) -> Self {
//         self.nums.insert(k, v);
//         self
//     }
//     fn b(mut self, k: &'static str, v: bool) -> Self {
//         self.bools.insert(k, v);
//         self
//     }
// }

// impl RowAccess for DemoRow {
//     fn get_f64(&self, id: &str) -> Option<f64> {
//         self.nums.get(id).copied()
//     }
//     fn get_bool(&self, id: &str) -> Option<bool> {
//         self.bools.get(id).copied()
//     }
//     fn get_str(&self, _: &str) -> Option<&str> {
//         None
//     }
//     fn get_u64(&self, field_id: &str) -> Option<u64> {
//         None
//     }

//     fn get_enum_i16(&self, field_id: &str) -> Option<i16> {
//         None
//     }
// }

// //
// // 3. Integration test: Bitspec eval + RowBinary encode/decode + envelope roundtrip
// //
// #[test]
// fn test_bitspec_rowbinary_envelope_roundtrip() {
//     // Step 1: Build pack + engine
//     let pack = VesselPing::bitspec_pack();
//     let mut engine = BitspecEngine::new();
//     let key = PolicyKey::new("gwf".into(), "vessel_ping".into(), 1);
//     engine.insert_pack(key.clone(), pack);

//     // Step 2: Row that should light up everything
//     let row = DemoRow::new()
//         .num("heat", 750.0) // triggers WARN + CRIT
//         .b("big_group", true) // flag
//         .num("serve", 0.0); // ns

//     let mut ctx = BenContext::new(&row);
//     engine.eval_ctx(&key, &mut ctx);

//     // Step 3: Build the actual wire event
//     let evt = VesselPing {
//         heat: 750.0,
//         big_group: true,
//         serve: 0.0,
//     };

//     let mut payload = Vec::new();
//     evt.encode_rowbinary(&mut payload);
//     assert!(!payload.is_empty());

//     // Step 4: Wrap in envelope + roundtrip parse
//     let mut buf = Vec::new();
//     Envelope::write_rowbinary_header(&mut buf, VesselPing::EVT_HASH, 12345, 3);
//     buf.extend_from_slice(&payload);

//     let env = Envelope::parse(&buf).expect("failed parsing envelope");
//     assert_eq!(env.evt_hash, VesselPing::EVT_HASH);
//     assert_eq!(env.epoch, 12345);

//     // Step 5: Decode back into VesselPing
//     let mut cur = RowBinCursor::new(env.payload);
//     let decoded = VesselPing::from_rowbinary(&mut cur).expect("decode failed");

//     assert!((decoded.heat - 750.0).abs() < 1e-6);
//     assert_eq!(decoded.big_group, true);
//     assert_eq!(decoded.serve, 0.0);

//     // Step 6: Confirm Bitspec mask + facts
//     // We expect 4 field bits:
//     //   - 2 from heat thresholds (WARN, CRIT)
//     //   - 1 from BIG_GROUP brule
//     //   - 1 from NO_SERVE brule
//     assert_eq!(ctx.field_mask.count_ones(), 4);

//     // At least one rule bit should also fire (from the CRIT flag_if mapping)
//     assert!(ctx.rule_mask != 0);

//     let fact = ctx.facts.get("heat.level").expect("missing fact");
//     match fact {
//         bitspec_engine::FactValue::Str(s) => assert_eq!(s, "CRIT"),
//         _ => panic!("Expected Str"),
//     }
//     assert!(ctx.facts.contains_key("bitspec.mask"));
// }

// //
// // 4. Sanity check: different values also roundtrip cleanly
// //
// #[test]
// fn test_bitspec_rowbinary_partial_trigger_roundtrip() {
//     let evt = VesselPing {
//         heat: 150.0,
//         big_group: false,
//         serve: 10.0,
//     };

//     let mut payload = Vec::new();
//     evt.encode_rowbinary(&mut payload);

//     let mut buf = Vec::new();
//     Envelope::write_rowbinary_header(&mut buf, VesselPing::EVT_HASH, 11111, 3);
//     buf.extend_from_slice(&payload);

//     let env = Envelope::parse(&buf).unwrap();
//     let mut cur = RowBinCursor::new(env.payload);
//     let decoded = VesselPing::from_rowbinary(&mut cur).unwrap();

//     assert!((decoded.heat - 150.0).abs() < 1e-6);
//     assert_eq!(decoded.big_group, false);
//     assert_eq!(decoded.serve, 10.0);
// }

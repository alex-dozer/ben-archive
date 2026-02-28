// use ben_macros::Bitspec;
// use bitspec_engine::{RowAccess, context::BenContext, engine::BitspecEngine, policy::PolicyKey};
// use std::collections::HashMap;

// #[derive(Debug, Bitspec)]
// pub struct VesselPing {
//     #[bspec(thresholds(
//         rule = "HEAT",
//         values = "WARN=200,CRIT=600",
//         fact = "heat.level",
//         flag_if = "CRIT"
//     ))]
//     heat: f64,

//     #[bspec(brule(rule = "BIG_GROUP", op = eq(true)))]
//     big_group: bool,

//     #[bspec(brule(rule = "NO_SERVE", op = eq(0)))]
//     serve: f64,
// }

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
// //
// //  End-to-end integration test
// //
// #[test]
// fn test_bitspec_threshold_flag_ns_and_rules() {
//     let pack = VesselPing::bitspec_pack();

//     let mut engine = BitspecEngine::new();
//     let key = PolicyKey::new("gwf".into(), "vessel_ping".into(), 1);
//     engine.insert_pack(key.clone(), pack);

//     let row = DemoRow::new()
//         .num("heat", 750.0) // triggers WARN + CRIT
//         .b("big_group", true) // brule set
//         .num("serve", 0.0); // brule triggers

//     let mut ctx = BenContext::new(&row);
//     engine.eval_ctx(&key, &mut ctx);

//     println!("seed_mask  {:b}", ctx.seed_mask);
//     println!("field_mask {:b}", ctx.field_mask);
//     println!("rule_mask  {:b}", ctx.rule_mask);
//     println!("final_mask {:b}", ctx.final_mask());

//     // --- BIT EXPECTATIONS ---
//     // We expect 4 field bits total:
//     //   - 2 from heat thresholds (WARN, CRIT)
//     //   - 1 from BIG_GROUP brule
//     //   - 1 from NO_SERVE brule
//     let field_bits = ctx.field_mask;
//     assert_eq!(field_bits.count_ones(), 4);

//     // --- FACT EXPECTATIONS ---
//     let fact = ctx.facts.get("heat.level").unwrap();
//     match fact {
//         bitspec_engine::FactValue::Str(s) => assert_eq!(s, "CRIT"),
//         _ => panic!("Expected Str fact value"),
//     }

//     assert!(ctx.facts.contains_key("bitspec.mask"));
// }

// //
// //
// //  Second test with different values
// //
// #[test]
// fn test_bitspec_no_triggers() {
//     let pack = VesselPing::bitspec_pack();
//     let mut engine = BitspecEngine::new();
//     let key = PolicyKey::new("gwf".into(), "vessel_ping".into(), 1);
//     engine.insert_pack(key.clone(), pack);

//     // Low heat, no brules
//     let row = DemoRow::new()
//         .num("heat", 50.0)
//         .b("big_group", false)
//         .num("serve", 10.0);

//     let mut ctx = BenContext::new(&row);
//     engine.eval_ctx(&key, &mut ctx);

//     // No thresholds should trigger
//     assert_eq!(ctx.field_mask, 0);
// }

// #[test]
// fn test_bitspec_only_flag_triggers() {
//     let pack = VesselPing::bitspec_pack();
//     let mut engine = BitspecEngine::new();
//     let key = PolicyKey::new("gwf".into(), "vessel_ping".into(), 1);
//     engine.insert_pack(key.clone(), pack);

//     let row = DemoRow::new()
//         .num("heat", 10.0) // no WARN/CRIT
//         .b("big_group", true) // brule triggers
//         .num("serve", 5.0); // brule does NOT trigger

//     let mut ctx = BenContext::new(&row);
//     engine.eval_ctx(&key, &mut ctx);

//     // Only the BIG_GROUP brule should fire
//     assert_eq!(ctx.field_mask.count_ones(), 1);
// }

// #[test]
// fn test_bitspec_only_ns_triggers() {
//     let pack = VesselPing::bitspec_pack();
//     let mut engine = BitspecEngine::new();
//     let key = PolicyKey::new("gwf".into(), "vessel_ping".into(), 1);
//     engine.insert_pack(key.clone(), pack);

//     let row = DemoRow::new()
//         .num("heat", 50.0) // below all thresholds
//         .b("big_group", false) // brule false
//         .num("serve", 0.0); // brule triggers

//     let mut ctx = BenContext::new(&row);
//     engine.eval_ctx(&key, &mut ctx);

//     // Only the NO_SERVE brule should fire
//     assert_eq!(ctx.field_mask.count_ones(), 1);
// }

// ///
// /// 3. Engine evaluation must match pack's bit layout
// ///
// #[test]
// fn test_bitspec_encoding_and_eval_consistency() {
//     let pack = VesselPing::bitspec_pack();
//     let mut engine = BitspecEngine::new();

//     let key = PolicyKey::new("gwf".into(), "vessel_ping".into(), 1);
//     engine.insert_pack(key.clone(), pack.clone());

//     let row = DemoRow::new()
//         .num("heat", 750.0)
//         .b("big_group", true)
//         .num("serve", 0.0);

//     let mut ctx = BenContext::new(&row);
//     engine.eval_ctx(&key, &mut ctx);

//     // Two thresholds + two brules => 4 field bits
//     assert_eq!(ctx.field_mask.count_ones(), 4);

//     // Final mask integrates seeds + fields + rules
//     assert!(ctx.final_mask() >= ctx.field_mask);
// }

// ///
// /// 4. Fact emission must store correct metadata from pack
// ///
// #[test]
// fn test_bitspec_fact_encoding() {
//     let pack = VesselPing::bitspec_pack();
//     let mut engine = BitspecEngine::new();

//     let key = PolicyKey::new("gwf".into(), "vessel_ping".into(), 1);
//     engine.insert_pack(key.clone(), pack);

//     let row = DemoRow::new()
//         .num("heat", 750.0)
//         .b("big_group", false)
//         .num("serve", 5.0);

//     let mut ctx = BenContext::new(&row);
//     engine.eval_ctx(&key, &mut ctx);

//     // High heat should trip at least one threshold bit
//     assert!(ctx.field_mask != 0);

//     let fact = ctx.facts.get("heat.level").expect("missing fact");
//     match fact {
//         bitspec_engine::FactValue::Str(s) => assert_eq!(s, "CRIT"),
//         _ => panic!("Expected Str fact"),
//     }

//     // Global mask fact must exist
//     assert!(ctx.facts.contains_key("bitspec.mask"));
// }

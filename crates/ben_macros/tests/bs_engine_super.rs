//! Full-system test: bitspec macro + engine evaluation.

use ben_macros::Bitspec;
use ben_wire::slot::SlotValue;
use bitspec_engine::{
    FactValue, RowAccess, pack::BitspecPack, predicate::PredicateSpec, threshold::ThresholdLevel,
    threshold::ThresholdOp, threshold::ThresholdSpec,
};
//
// ---------------------------------------------------------
// Local test Row implementation
// ---------------------------------------------------------
//

#[derive(Default)]
struct TestRow {
    map: std::collections::HashMap<String, SlotValue<'static>>,
}

impl TestRow {
    fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }

    fn with(mut self, key: &str, val: SlotValue<'static>) -> Self {
        self.map.insert(key.to_string(), val);
        self
    }
}

impl RowAccess for TestRow {
    fn get_str(&self, key: &str) -> Option<&str> {
        self.map.get(key)?.as_str().ok()
    }
    fn get_f64(&self, key: &str) -> Option<f64> {
        self.map.get(key)?.as_f64().ok().copied()
    }
    fn get_u64(&self, key: &str) -> Option<u64> {
        self.map.get(key)?.as_u64().ok().copied()
    }
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.map.get(key)?.as_i64().ok().copied()
    }
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.map.get(key)?.as_bool().ok().copied()
    }

    fn get_slot(&self, field_id: &str) -> Option<&SlotValue> {
        self.map.get(field_id)
    }
}

//
// ---------------------------------------------------------
// A bitspec-annotated test struct
// ---------------------------------------------------------
//

/*
This struct is a toy example of a Ben event type.
Bitspec is our macro that turns each field’s annotations into a packed rule-evaluation engine.
When you derive Bitspec, you get a compiled “rule pack” that can evaluate predicates and thresholds
at runtime with a tiny memory footprint.
*/
#[derive(Debug, Bitspec)]

struct ExampleEvent {
    /*
    A simple numeric rule.
    This generates a predicate called NUM_GT_10 and sets its bit when num > 10.
    The brule (Boolean Rule) syntax always yields a single bit in the mask.
    */
    #[bspec(brule(rule = "NUM_GT_10", op = gt(10)))]
    num: f64,

    /*
    A text predicate.
    Sets the TXT_START_HE bit when the field value begins with "he".
    Every string operator (eq, ne, starts_with, contains) is supported.
    */
    #[bspec(brule(rule = "TXT_START_HE", op = starts_with("he")))]
    name: &'static str,

    /*
    A Boolean equality rule.
    When flag == true, the FLAG_TRUE bit gets flipped on.
     */
    #[bspec(brule(rule = "FLAG_TRUE", op = eq(true)))]
    flag: bool,

    /*
    A composite rule built from multiple predicates.
    all(...) requires that all conditions pass.
    Here: comp must be > 5 and < 20.
    Macros like all, any, and not let you build arbitrarily expressive logical rules without writing manual code.
     */
    #[bspec(brule(
        rule = "COMPLEX_ALL",
        op = all(
            gt(5),
            lt(20),
        )
    ))]
    comp: f64,

    /*
    A full threshold rule pack.
    This is where things get spicy:
        -	op = "gte" means the comparison operation is ≥
        -	"WARM=100, HOT=200, SCORCH=500" defines named threshold levels
        -	The highest triggered level is written into a fact called "heat.level"
        -	The flags = "ALERT:HOT, CAUTION:WARM, PUKE:SCORCH" expression maps threshold names -> fact flags
        ALERT = Flag Set, HOT =fact inserted
        all arbitrary except op
    */
    #[bspec(thresholds(
        rule = "HEAT",
        op = "gte",
        values = "WARM=100, HOT=200, SCORCH=500",
        fact = "heat.level",
        flags = "ALERT:HOT, CAUTION:WARM, PUKE:SCORCH"
    ))]
    heat: f64,
}

//

#[test]
fn test_bitspec_macro_and_engine() {
    let pack: &BitspecPack = &ExampleEvent::BITSPEC_PACK;
    println!("bitcount = {}", pack.bit_count);

    let row = TestRow::new()
        .with("num", SlotValue::F64(42.0))
        .with("name", SlotValue::Str("hello"))
        .with("flag", SlotValue::Bool(true))
        .with("comp", SlotValue::F64(12.0))
        .with("heat", SlotValue::F64(350.0)); // >=200 => HOT; >=500 => SCORCH (not here)

    let (mask, facts) = pack.eval(&row);

    // Print the mask for debugging
    println!("mask bits: {:064b}", mask);

    //
    // ---------------------------------------------------------
    // VERIFY PREDICATE BITS
    // ---------------------------------------------------------
    //

    // Predicate ordering:
    // Bit 0: NUM_GT_10
    // Bit 1: TXT_START_HE
    // Bit 2: FLAG_TRUE
    // Bit 3: COMPLEX_ALL
    //
    // Threshold levels begin at Bit 4.

    assert!(mask & (1 << 0) != 0, "NUM_GT_10 should fire");
    assert!(mask & (1 << 1) != 0, "TXT_START_HE should fire");
    assert!(mask & (1 << 2) != 0, "FLAG_TRUE should fire");
    assert!(mask & (1 << 3) != 0, "COMPLEX_ALL should fire");

    //
    // ---------------------------------------------------------
    // VERIFY THRESHOLDS
    // levels = [WARM, HOT, SCORCH]
    // Bits:
    //   WARM   => bit 4
    //   HOT    => bit 5
    //   SCORCH => bit 6
    // ---------------------------------------------------------
    //

    // heat = 350 ≥ WARM (100)
    assert!(mask & (1 << 4) != 0, "WARM threshold should fire");

    // heat = 350 ≥ HOT (200)
    assert!(mask & (1 << 5) != 0, "HOT threshold should fire");

    // heat = 350 < SCORCH (500)
    assert!(mask & (1 << 6) == 0, "SCORCH threshold should NOT fire");

    //
    // ---------------------------------------------------------
    // VERIFY FACT
    // "heat.level" should contain best (highest) threshold matched: HOT
    // ---------------------------------------------------------
    //

    let level_val = facts.get("heat.level").expect("missing fact heat.level");

    match level_val {
        FactValue::Str(s) => assert_eq!(s, "HOT"),
        _ => panic!("heat.level should be a string"),
    }

    //
    // ---------------------------------------------------------
    // VERIFY FLAG MAPPING
    //
    // flags = {
    //   ALERT: [HOT, SCORCH]
    //   CAUTION: [WARM]
    // }
    //
    // heat = 350 => WARM + HOT fire -> ALERT + CAUTION
    // ---------------------------------------------------------
    //

    let alert = facts.get("ALERT").unwrap();
    let caution = facts.get("CAUTION").unwrap();

    match alert {
        FactValue::Str(s) => assert_eq!(s, "HOT"), // highest triggered in that set
        _ => panic!("ALERT fact should be str"),
    }

    match caution {
        FactValue::Str(s) => assert_eq!(s, "WARM"),
        _ => panic!("CAUTION fact should be str"),
    }

    //
    // ---------------------------------------------------------
    // MISSING FIELD BEHAVIOR
    // ---------------------------------------------------------
    //

    let row_missing = TestRow::new().with("num", SlotValue::F64(3.0)); // only num exists

    let (mask2, facts2) = pack.eval(&row_missing);

    assert_eq!(mask2 & (1 << 0), 0, "num=3 should NOT fire NUM_GT_10");
    assert!(
        facts2.is_empty(),
        "no facts should be recorded when heat missing"
    );
}

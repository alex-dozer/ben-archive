use ben_contracts::{BlotOp, BlotSpec};
use ben_macros::Blot;
use blot_engine::{RowGet, RowPut};

#[derive(Blot)]
pub struct TestBlot {
    #[blotspec(rule="PII_EMAIL", op = mask_all)]
    email: String,

    #[blotspec(op = mask_suffix(4))]
    card: String,
}

#[test]
fn blot_rules_are_generated() {
    let rules = TestBlot::BLOT_RULES;
    assert_eq!(rules.len(), 2);

    let r0 = &rules[0];
    assert_eq!(r0.field, "email");
    assert_eq!(r0.rule, Some("PII_EMAIL"));
    assert!(matches!(r0.op, BlotOp::MaskAll));

    let r1 = &rules[1];
    assert_eq!(r1.field, "card");
    assert_eq!(r1.rule, None);
    assert!(matches!(r1.op, BlotOp::MaskSuffix { keep: 4 }));
}

#[test]
fn test_blot_sanitization() {
    let rules = TestBlot::BLOT_RULES;
    let engine = blot_engine::BlotEngine::new(rules);

    struct In<'a> {
        email: &'a str,
        card: &'a str,
    }
    impl RowGet for In<'_> {
        fn get_str(&self, f: &str) -> Option<&str> {
            match f {
                "email" => Some(self.email),
                "card" => Some(self.card),
                _ => None,
            }
        }
    }

    #[derive(Default)]
    struct Out(std::collections::HashMap<String, String>);
    impl RowPut for Out {
        fn put_str(&mut self, f: &str, v: String) {
            self.0.insert(f.to_string(), v);
        }
    }

    let mut out = Out::default();
    engine.apply(
        &In {
            email: "john@example.com",
            card: "1234567890123456",
        },
        &mut out,
    );

    assert_eq!(out.0["email"], "****************");
    assert_eq!(&out.0["card"][12..], "3456");
}

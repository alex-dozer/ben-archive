// crates/ben_macros/tests/lucius_tests.rs

use ben_contracts::{LuciusFieldSpec, LuciusLevel, LuciusSpec};
use ben_macros::Lucius;

#[derive(Lucius)]
pub struct SingleField {
    #[lspec(
        level = "dynamic",
        expected_type = "bin",
        rule = "PAYLOAD_SCAN",
        route = "lucius-deep",
        note = "scan me"
    )]
    payload: String,
}

#[derive(Lucius)]
pub struct MultiFieldDefaults {
    // no lspec on this one  should not appear
    id: String,

    // uses defaults: level = "light", expected_type = "bin", route = field name, note = None
    #[lspec]
    attachment_key: String,

    #[lspec(level = "static", expected_type = "archive", rule = "ARCHIVE_SCAN")]
    archive_blob: Vec<u8>,
}

#[test]
fn lucius_single_field_spec_is_correct() {
    let specs = SingleField::LUCIUS_SPECS;
    assert_eq!(specs.len(), 1);

    let s = &specs[0];

    assert_eq!(s.field, "payload");
    assert_eq!(s.rule, Some("PAYLOAD_SCAN"));
    assert!(matches!(s.level, LuciusLevel::DynamicSandbox));
    assert_eq!(s.expected_type, "bin");
    assert_eq!(s.route, "lucius-deep");
    assert_eq!(s.note, Some("scan me"));
}

#[test]
fn lucius_multi_field_specs_and_defaults() {
    let specs = MultiFieldDefaults::LUCIUS_SPECS;
    assert_eq!(specs.len(), 2);

    let att = &specs[0];
    assert_eq!(att.field, "attachment_key");
    assert_eq!(att.rule, None);
    assert!(matches!(att.level, LuciusLevel::Light));
    assert_eq!(att.expected_type, "bin");
    assert_eq!(att.route, "attachment_key");
    assert_eq!(att.note, None);

    let arch = &specs[1];
    assert_eq!(arch.field, "archive_blob");
    assert_eq!(arch.rule, Some("ARCHIVE_SCAN"));
    assert!(matches!(arch.level, LuciusLevel::StaticDeep));
    assert_eq!(arch.expected_type, "archive");
    assert_eq!(arch.route, "archive_blob");
    assert_eq!(arch.note, None);
}

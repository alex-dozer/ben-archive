use ben_macros::BenEnum;
use linkme;

#[derive(Debug, PartialEq, BenEnum)]
enum VesselState {
    // explicit value
    #[benum(label = "Idle Human", value = 0)]
    Idle,

    // explicit value via attribute
    #[benum(value = 5)]
    Moving,

    // auto = 6
    Docked,
}

#[test]
fn test_enum_numeric_values() {
    // Ensure the to_i16 mapping is correct
    assert_eq!(VesselState::Idle.__ben_enum_to_i16(), 0);
    assert_eq!(VesselState::Moving.__ben_enum_to_i16(), 5);
    assert_eq!(VesselState::Docked.__ben_enum_to_i16(), 6);

    // Ensure TryFrom<i16> works
    assert_eq!(VesselState::try_from(0).unwrap(), VesselState::Idle);
    assert_eq!(VesselState::try_from(5).unwrap(), VesselState::Moving);
    assert_eq!(VesselState::try_from(6).unwrap(), VesselState::Docked);

    // Out-of-range values must fail
    assert!(VesselState::try_from(99).is_err());
}

#[test]
fn test_enum_static_metadata() {
    // Ensure the static table is correct
    // assert_eq!(
    //     VesselState::__BEN_ENUM_VARIANTS,
    //     &[("Idle Human", 0), ("Moving", 5), ("Docked", 6),]
    // );

    // Bits required should be 8 (fits signed byte)
    assert_eq!(VesselState::__BEN_ENUM_BITS, 8);
}

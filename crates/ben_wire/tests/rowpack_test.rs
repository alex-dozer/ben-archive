use ben_wire::{
    envelope::Envelope,
    rowpack::{decode_rowpack, BenEncodeRowpack, Tag},
    slot::SlotValue,
};

struct Login {
    pub heat: u32,
    pub user_id: u64,
    pub enabled: bool,
}

impl BenEncodeRowpack for Login {
    const EVT_HASH: [u8; 32] = [0xAB; 32]; // dummy hash for test
    const FIELD_COUNT: u16 = 3;

    fn encode_rowpack_payload(&self, out: &mut Vec<u8>) {
        // field 0: heat as U64
        out.push(Tag::U64 as u8);
        out.extend_from_slice(&(self.heat as u64).to_le_bytes());

        // field 1: user_id
        out.push(Tag::U64 as u8);
        out.extend_from_slice(&self.user_id.to_le_bytes());

        // field 2: enabled
        out.push(Tag::Bool as u8);
        out.push(if self.enabled { 1 } else { 0 });
    }
}

#[test]
fn rowpack_encode_decode_roundtrip() {
    let login = Login {
        heat: 420,
        user_id: 7,
        enabled: true,
    };

    let epoch = 1234_u64;
    let bytes = login.to_envelope_rowpack(epoch);

    // parse envelope
    let env = Envelope::parse(&bytes).expect("parse envelope");
    assert_eq!(env.evt_hash, Login::EVT_HASH);
    assert_eq!(env.epoch, epoch);

    // decode rowpack payload
    let row = decode_rowpack(env.payload, Login::FIELD_COUNT as usize).unwrap();
    assert_eq!(row.len(), 3);

    assert!(matches!(row[0], SlotValue::U64(420)));
    assert!(matches!(row[1], SlotValue::U64(7)));
    assert!(matches!(row[2], SlotValue::Bool(true)));
}

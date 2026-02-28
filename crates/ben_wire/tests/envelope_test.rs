// use ben_wire::envelope::{Encoding, Envelope};

// #[test]
// fn envelope_rejects_short_buffer() {
//     let buf = vec![0u8; 10];
//     let res = Envelope::parse(&buf);
//     assert!(res.is_err(), "short envelope should error");
// }

// #[test]
// fn envelope_parses_valid_header() {
//     let evt_hash = [0xCDu8; 32];
//     let epoch = 999_u64;
//     let encoding = Encoding::Rowpack as u8;
//     let field_count: u16 = 3;
//     let payload: &[u8] = &[0xFF, 0xEE];

//     let mut buf = Vec::new();
//     buf.extend_from_slice(&evt_hash);
//     buf.extend_from_slice(&epoch.to_le_bytes());
//     buf.push(encoding);
//     buf.extend_from_slice(&field_count.to_le_bytes());
//     buf.extend_from_slice(payload);

//     let env = Envelope::parse(&buf).expect("parse envelope");
//     assert_eq!(env.evt_hash, evt_hash);
//     assert_eq!(env.epoch, epoch);
//     assert_eq!(env.encoding as u8, encoding);
//     assert_eq!(env.payload, payload);
// }

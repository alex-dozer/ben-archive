use ben_wire::{
    Encoding,
    envelope::Envelope,
    rowbinary::{EncodeQuic, RowBinCursor, RowBinaryDecode, RowBinaryEncode, RowBinaryResult},
};

#[derive(Debug, PartialEq)]
struct TestEvent {
    a: u64,
    b: bool,
    c: String,
    d: i64,
    e: f64,
    ipv4: u32,
    ipv6: u128,
    dt_epoch: i64,
    dt_scale: u32,
    uuid: [u8; 16],
}

impl RowBinaryEncode for TestEvent {
    fn encode_rowbinary(&self, out: &mut Vec<u8>) -> RowBinaryResult {
        out.extend_from_slice(&self.a.to_le_bytes());
        out.push(self.b as u8);

        // string: len + data
        let bytes = self.c.as_bytes();
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(bytes);

        out.extend_from_slice(&self.d.to_le_bytes());
        out.extend_from_slice(&self.e.to_bits().to_le_bytes());
        out.extend_from_slice(&self.ipv4.to_le_bytes());
        out.extend_from_slice(&self.ipv6.to_le_bytes());
        out.extend_from_slice(&self.dt_epoch.to_le_bytes());
        out.extend_from_slice(&self.dt_scale.to_le_bytes());
        out.extend_from_slice(&self.uuid);

        Ok(())
    }
}

impl RowBinaryDecode for TestEvent {
    const EVT_HASH: [u8; 32] = *b"12345678901234567890123456789012";
    const FIELD_COUNT: u16 = 10;

    fn from_rowbinary(cur: &mut RowBinCursor<'_>) -> anyhow::Result<Self> {
        let a = cur.read_u64()?;
        let b = cur.read_bool()?;
        let c = cur.read_string()?;
        let d = cur.read_i64()?;
        let e = cur.read_f64()?;
        let ipv4 = cur.read_ipv4()?;
        let ipv6 = cur.read_ipv6()?;

        let (dt_epoch, dt_scale) = cur.read_datetime64()?;
        let uuid = cur.read_uuid()?;

        Ok(Self {
            a,
            b,
            c,
            d,
            e,
            ipv4,
            ipv6,
            dt_epoch,
            dt_scale,
            uuid,
        })
    }
}

fn make_event() -> TestEvent {
    TestEvent {
        a: 999,
        b: true,
        c: "hello-world".into(),
        d: -1234567,
        e: 3.14159,
        ipv4: 0x0A0B0C0D,
        ipv6: 0x11223344556677889900AABBCCDDEEFF_u128,
        dt_epoch: 1700000000,
        dt_scale: 3,
        uuid: [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ],
    }
}

#[test]
fn test_rowbinary_roundtrip() {
    let evt = make_event();

    let mut buf = Vec::new();
    evt.encode_rowbinary(&mut buf);

    let mut cur = RowBinCursor { buf: &buf, pos: 0 };
    let decoded = TestEvent::from_rowbinary(&mut cur).unwrap();

    assert_eq!(evt, decoded);
}

#[test]
fn test_truncated_decode_fails() {
    let evt = make_event();

    let mut buf = Vec::new();
    evt.encode_rowbinary(&mut buf);

    let truncated = &buf[..5];

    let mut cur = RowBinCursor {
        buf: truncated,
        pos: 0,
    };

    let err = TestEvent::from_rowbinary(&mut cur).unwrap_err();
    assert!(err.to_string().contains("buffer underflow"));
}

#[test]
fn test_string_encoding() {
    let evt = TestEvent {
        c: "🔥 unicode ok".into(),
        ..make_event()
    };

    let mut buf = Vec::new();
    evt.encode_rowbinary(&mut buf);

    let mut cur = RowBinCursor { buf: &buf, pos: 0 };
    let decoded = TestEvent::from_rowbinary(&mut cur).unwrap();

    assert_eq!(decoded.c, "🔥 unicode ok");
}

#[test]
fn test_envelope_roundtrip() {
    let evt = make_event();

    let mut payload = Vec::new();
    evt.encode_rowbinary(&mut payload);

    let epoch = 123456;

    let mut outer = Vec::new();
    ben_wire::envelope::Envelope::write_rowbinary_header(
        &mut outer,
        TestEvent::EVT_HASH,
        epoch,
        TestEvent::FIELD_COUNT,
    );
    outer.extend_from_slice(&payload);

    // parse envelope
    let env = Envelope::parse(&outer).unwrap();

    assert_eq!(env.evt_hash, TestEvent::EVT_HASH);
    assert_eq!(env.epoch, epoch);
    assert_eq!(env.encoding as u8, Encoding::RowBinary as u8);
    assert_eq!(env.payload, payload.as_slice());
}

#[test]
fn test_quic_encoding() {
    struct Q(TestEvent);
    impl RowBinaryEncode for Q {
        fn encode_rowbinary(&self, out: &mut Vec<u8>) -> RowBinaryResult {
            self.0.encode_rowbinary(out)
        }
    }

    impl ben_wire::rowbinary::EncodeQuic for Q {
        const EVT_HASH: [u8; 32] = TestEvent::EVT_HASH;
        const FIELD_COUNT: u16 = TestEvent::FIELD_COUNT;
    }

    let evt = Q(make_event());

    let mut regular = Vec::new();
    evt.0.encode_rowbinary(&mut regular);

    let mut quic = Vec::new();
    evt.encode_quic(&mut quic);

    assert_eq!(
        quic,
        [
            &TestEvent::EVT_HASH[..],
            &TestEvent::FIELD_COUNT.to_le_bytes(),
            &regular[..]
        ]
        .concat()
    );
}

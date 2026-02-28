use crate::{schema::Schema, slot::SlotValue};

#[repr(u8)]
pub enum Tag {
    Missing = 0,
    U64 = 1,
    I64 = 2,
    F64 = 3,
    Bool = 4,
    StrId = 5,
    IPv4 = 6,
    IPv6 = 7,
    DT64 = 8,
    Uuid = 9,
}

pub trait BenEncodeRowpack {
    const EVT_HASH: [u8; 32];
    const FIELD_COUNT: u16;

    fn encode_rowpack_payload(&self, out: &mut Vec<u8>);

    fn to_envelope_rowpack(&self, epoch: u64) -> Vec<u8> {
        let mut out = Vec::with_capacity(64);

        crate::envelope::Envelope::write_rowbinary_header(
            &mut out,
            Self::EVT_HASH,
            epoch,
            Self::FIELD_COUNT,
        );

        self.encode_rowpack_payload(&mut out);

        out
    }
}

pub fn decode_rowpack(mut p: &[u8], fields: usize) -> anyhow::Result<Vec<SlotValue>> {
    use Tag::*;
    let mut row = vec![SlotValue::Missing; fields];
    for id in 0..fields {
        let tag = p[0];
        p = &p[1..];
        row[id] = match tag {
            x if x == U64 as u8 => {
                let (n, r) = p.split_at(8);
                p = r;
                SlotValue::U64(u64::from_le_bytes(n.try_into()?))
            }
            x if x == I64 as u8 => {
                let (n, r) = p.split_at(8);
                p = r;
                SlotValue::I64(i64::from_le_bytes(n.try_into()?))
            }
            x if x == F64 as u8 => {
                let (n, r) = p.split_at(8);
                p = r;
                SlotValue::F64(f64::from_bits(u64::from_le_bytes(n.try_into()?)))
            }
            x if x == Bool as u8 => {
                let b = p[0];
                p = &p[1..];
                SlotValue::Bool(b != 0)
            }
            x if x == IPv4 as u8 => {
                let (n, r) = p.split_at(4);
                p = r;
                SlotValue::IPv4(u32::from_le_bytes(n.try_into()?))
            }
            x if x == IPv6 as u8 => {
                let (n, r) = p.split_at(16);
                p = r;
                SlotValue::IPv6(u128::from_le_bytes(n.try_into()?))
            }
            x if x == DT64 as u8 => {
                let (n, r) = p.split_at(12);
                p = r;
                let epoch = i64::from_le_bytes(n[0..8].try_into()?);
                let scale = u32::from_le_bytes(n[8..12].try_into()?);
                SlotValue::DateTime64 { epoch, scale }
            }
            x if x == Uuid as u8 => {
                let (n, r) = p.split_at(16);
                p = r;
                SlotValue::Uuid(n.try_into()?)
            }
            _ => SlotValue::Missing,
        };
    }
    Ok(row)
}

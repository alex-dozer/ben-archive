pub mod envelope;
pub mod error;
pub mod rowbinary;
pub mod rowpack;
pub mod schema;
pub mod serde_bridge;
pub mod slot;
pub mod view;

pub use crate::envelope::{Encoding, Envelope};
pub use crate::schema::Schema;

pub mod ch_binary {
    #[inline]
    pub fn write_uvarint(mut x: usize, buf: &mut Vec<u8>) {
        while x >= 0x80 {
            buf.push((x as u8) | 0x80);
            x >>= 7;
        }
        buf.push(x as u8);
    }

    #[inline]
    pub fn write_str(s: &str, out: &mut Vec<u8>) {
        write_uvarint(s.len(), out);
        out.extend_from_slice(s.as_bytes());
    }
}

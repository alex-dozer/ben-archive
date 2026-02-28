use anyhow::{Result, ensure};

pub type RowBinaryResult = anyhow::Result<()>;

pub struct RowBinCursor<'a> {
    pub buf: &'a [u8],
    pub pos: usize,
}

impl<'a> RowBinCursor<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    #[inline]
    pub fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.remaining() < n {
            anyhow::bail!("buffer underflow");
        }
        let start = self.pos;
        self.pos += n;
        Ok(&self.buf[start..self.pos])
    }

    #[inline]
    pub fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into()?))
    }
    #[inline]
    pub fn read_i64(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into()?))
    }

    #[inline]
    pub fn read_f64(&mut self) -> Result<f64> {
        let bytes: [u8; 8] = self.take(8)?.try_into()?;
        Ok(f64::from_le_bytes(bytes))
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    #[inline]
    pub fn read_ipv4(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }

    #[inline]
    pub fn read_ipv6(&mut self) -> Result<u128> {
        Ok(u128::from_le_bytes(self.take(16)?.try_into()?))
    }

    #[inline]
    pub fn read_datetime64(&mut self) -> Result<(i64, u32)> {
        let epoch = i64::from_le_bytes(self.take(8)?.try_into()?);
        let scale = u32::from_le_bytes(self.take(4)?.try_into()?);
        Ok((epoch, scale))
    }

    #[inline]
    pub fn read_uuid(&mut self) -> Result<[u8; 16]> {
        Ok(self.take(16)?.try_into()?)
    }

    #[inline]
    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.take(1)?[0] != 0)
    }
    #[inline]
    pub fn read_string(&mut self) -> Result<String> {
        let len = u32::from_le_bytes(self.take(4)?.try_into()?) as usize;
        let bytes = self.take(len)?;
        Ok(String::from_utf8(bytes.to_vec())?)
    }
}

pub trait RowBinaryEncode {
    fn encode_rowbinary(&self, out: &mut Vec<u8>) -> RowBinaryResult;
}

pub trait RowBinaryDecode: Sized {
    const EVT_HASH: [u8; 32];
    const FIELD_COUNT: u16;

    fn from_rowbinary(cur: &mut RowBinCursor<'_>) -> Result<Self>;
}

pub trait EncodeQuic: RowBinaryEncode {
    const EVT_HASH: [u8; 32];
    const FIELD_COUNT: u16;

    fn encode_quic(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&Self::EVT_HASH);

        out.extend_from_slice(&Self::FIELD_COUNT.to_le_bytes());

        self.encode_rowbinary(out);
    }
}

pub trait DecodeQuic: RowBinaryDecode {
    const EVT_HASH: [u8; 32];
    const FIELD_COUNT: u16;

    fn decode_quic(buf: &[u8]) -> Result<Self> {
        if buf.len() < 34 {
            anyhow::bail!("decode_quic: frame too small");
        }

        let (hash_bytes, rest) = buf.split_at(32);
        if hash_bytes != <Self as DecodeQuic>::EVT_HASH {
            anyhow::bail!("decode_quic: wrong event hash");
        }

        let (fc_bytes, payload) = rest.split_at(2);
        let fc = u16::from_le_bytes(fc_bytes.try_into()?);

        if fc != <Self as DecodeQuic>::FIELD_COUNT {
            anyhow::bail!(
                "decode_quic: field count mismatch (got {}, expected {})",
                fc,
                <Self as DecodeQuic>::FIELD_COUNT
            );
        }

        let mut cur = RowBinCursor::new(payload);
        Self::from_rowbinary(&mut cur)
    }
}

impl<T: RowBinaryEncode> RowBinaryEncode for Option<T> {
    fn encode_rowbinary(&self, buf: &mut Vec<u8>) -> Result<(), anyhow::Error> {
        match self {
            None => {
                buf.push(1u8); // null marker
                Ok(())
            }
            Some(value) => {
                buf.push(0u8); // not null
                value.encode_rowbinary(buf)
            }
        }
    }
}
impl<T: RowBinaryDecode> RowBinaryDecode for Option<T> {
    const EVT_HASH: [u8; 32] = T::EVT_HASH;
    const FIELD_COUNT: u16 = T::FIELD_COUNT;

    fn from_rowbinary(cur: &mut RowBinCursor<'_>) -> Result<Self, anyhow::Error> {
        let is_null = cur.take(1)?[0] != 0;
        if is_null {
            Ok(None)
        } else {
            Ok(Some(T::from_rowbinary(cur)?))
        }
    }
}

impl<T: RowBinaryEncode> RowBinaryEncode for Vec<T> {
    fn encode_rowbinary(&self, buf: &mut Vec<u8>) -> Result<(), anyhow::Error> {
        let len = self.len() as u64;
        buf.extend_from_slice(&len.to_le_bytes());

        for item in self {
            item.encode_rowbinary(buf)?;
        }

        Ok(())
    }
}
impl<T: RowBinaryDecode> RowBinaryDecode for Vec<T> {
    const EVT_HASH: [u8; 32] = T::EVT_HASH;
    const FIELD_COUNT: u16 = T::FIELD_COUNT;

    fn from_rowbinary(cur: &mut RowBinCursor<'_>) -> Result<Self, anyhow::Error> {
        let len = cur.read_u64()? as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(T::from_rowbinary(cur)?);
        }
        Ok(out)
    }
}

impl RowBinaryEncode for u64 {
    fn encode_rowbinary(&self, buf: &mut Vec<u8>) -> Result<(), anyhow::Error> {
        buf.extend_from_slice(&self.to_le_bytes());
        Ok(())
    }
}
impl RowBinaryDecode for u64 {
    const EVT_HASH: [u8; 32] = [0u8; 32];
    const FIELD_COUNT: u16 = 1;

    fn from_rowbinary(cur: &mut RowBinCursor<'_>) -> Result<Self, anyhow::Error> {
        Ok(cur.read_u64()?)
    }
}

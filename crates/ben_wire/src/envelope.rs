#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Encoding {
    RowBinary = 2,
}

#[derive(Clone, Debug)]
pub struct Envelope<'a> {
    pub evt_hash: [u8; 32],
    pub epoch: u64,
    pub encoding: Encoding,
    pub payload: &'a [u8],
}

impl<'a> Envelope<'a> {
    pub fn parse(buf: &'a [u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(buf.len() >= 32 + 8 + 1 + 2, "short envelope");

        let (hash_bytes, rest) = buf.split_at(32);
        let mut evt_hash = [0u8; 32];
        evt_hash.copy_from_slice(hash_bytes);

        let epoch = u64::from_le_bytes(rest[0..8].try_into()?);

        let encoding = match rest[8] {
            2 => Encoding::RowBinary,
            _ => anyhow::bail!("unknown encoding"),
        };

        let _field_count = u16::from_le_bytes(rest[9..11].try_into()?);
        let payload = &rest[11..];

        Ok(Self {
            evt_hash,
            epoch,
            encoding,
            payload,
        })
    }

    pub fn write_rowbinary_header(
        dst: &mut Vec<u8>,
        evt_hash: [u8; 32],
        epoch: u64,
        field_count: u16,
    ) {
        dst.extend_from_slice(&evt_hash);
        dst.extend_from_slice(&epoch.to_le_bytes());
        dst.push(Encoding::RowBinary as u8);
        dst.extend_from_slice(&field_count.to_le_bytes());
    }
}

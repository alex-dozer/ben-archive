#[derive(Clone, Copy, Debug)]
pub enum SlotValue<'a> {
    Missing,
    U64(u64),
    I64(i64),
    F64(f64),
    Bool(bool),
    Str(&'a str),
    IPv4(u32),
    IPv6(u128),
    DateTime64 { epoch: i64, scale: u32 },
    Uuid([u8; 16]),
}

impl<'a> SlotValue<'a> {
    #[inline]
    pub fn as_u64(&self) -> Result<&u64, String> {
        match &self {
            SlotValue::U64(x) => Ok(&x),
            _ => Err("u64".into()),
        }
    }
    #[inline]
    pub fn as_i64(&self) -> Result<&i64, String> {
        match &self {
            SlotValue::I64(x) => Ok(&x),
            _ => Err("i64".into()),
        }
    }
    #[inline]
    pub fn as_f64(&self) -> Result<&f64, String> {
        match &self {
            SlotValue::F64(x) => Ok(&x),
            _ => Err("f64".into()),
        }
    }

    #[inline]
    pub fn as_str(&self) -> Result<&'a str, String> {
        match &self {
            SlotValue::Str(x) => Ok(x),
            _ => Err("str".into()),
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Result<&bool, String> {
        match &self {
            SlotValue::Bool(x) => Ok(&x),
            _ => Err("bool".into()),
        }
    }
}

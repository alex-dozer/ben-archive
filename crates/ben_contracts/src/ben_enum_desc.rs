use crate::enum_info::EnumVariant;

pub trait BenEnumDesc: Sized + 'static {
    const BITS: u8;

    const VARIANTS: &'static [EnumVariant];

    fn to_i16(self) -> i16;

    fn from_i16(v: i16) -> Option<Self>;
}

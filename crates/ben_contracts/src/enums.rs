pub use linkme::distributed_slice;

use crate::EnumInfo;

pub trait BenEnum: Sized + 'static {
    fn ben_enum_info() -> &'static EnumInfo;

    fn to_ben_discriminant(self) -> i32;

    fn from_ben_discriminant(v: i32) -> Option<Self>;
}

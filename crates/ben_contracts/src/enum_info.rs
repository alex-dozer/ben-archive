use linkme::distributed_slice;

#[derive(Debug)]
pub struct EnumInfo {
    pub name: &'static str,
    pub bits: u8,
    pub variants: &'static [EnumVariant],
}

impl EnumInfo {
    /// Convenience helper: compute min/max over the concrete variants.
    pub fn bounds(&self) -> (i16, i16) {
        let v = self.variants;
        let min = v.iter().map(|x| x.value).min().unwrap_or(0);
        let max = v.iter().map(|x| x.value).max().unwrap_or(0);
        (min, max)
    }

    pub fn variants(&self) -> &'static [EnumVariant] {
        self.variants
    }
}

#[distributed_slice]
pub static BEN_ENUM_REGISTRY: [fn() -> &'static EnumInfo] = [..];

pub fn lookup_enum(name: &str) -> Option<&'static EnumInfo> {
    for f in BEN_ENUM_REGISTRY {
        let info = f();
        if info.name == name {
            return Some(info);
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub label: &'static str,
    pub value: i16,
}

pub trait BenEnumInfo {
    fn variants() -> &'static [EnumVariant];

    fn bounds() -> (i16, i16) {
        let v = Self::variants();
        let min = v.iter().map(|x| x.value).min().unwrap_or(0);
        let max = v.iter().map(|x| x.value).max().unwrap_or(0);
        (min, max)
    }
}

// pub fn clickhouse_enum_type<T>() -> ::std::string::String
// where
//     T: BenEnumInfo,
// {
//     let (min_v, max_v) = T::bounds();
//     let fits_enum8 = min_v >= i16::from(i8::MIN) && max_v <= i16::from(i8::MAX);

//     let variants = T::variants();
//     let tags = variants
//         .iter()
//         .map(|v| ::std::format!("'{}' = {}", v.label, v.value))
//         .collect::<::std::vec::Vec<_>>()
//         .join(", ");

//     if fits_enum8 {
//         ::std::format!("Enum8({})", tags)
//     } else {
//         ::std::format!("Enum16({})", tags)
//     }
// }

// use crate::slot::{SlotValue, conv};
// use serde::de::{self, IntoDeserializer};

// macro_rules! visit_num {
//     ($meth:ident, $ty:ty, $visit:ident, $conv:ident) => {
//         fn $meth<V>(self, v: V) -> Result<V::Value, Self::Error>
//         where
//             V: serde::de::Visitor<'de>,
//         {
//             let x: $ty = conv::$conv(self.0)?;
//             v.$visit(x)
//         }
//     };
// }

// pub struct SlotValueDeserializer(pub SlotValue);
// impl<'de> serde::de::Deserializer<'de> for SlotValueDeserializer {
//     type Error = de::value::Error;
//     visit_num!(deserialize_u64, u64, visit_u64, to_u64);
//     visit_num!(deserialize_i64, i64, visit_i64, to_i64);
//     visit_num!(deserialize_f64, f64, visit_f64, to_f64);

//     fn deserialize_bool<V>(self, v: V) -> Result<V::Value, Self::Error>
//     where
//         V: serde::de::Visitor<'de>,
//     {
//         match self.0 {
//             SlotValue::Bool(b) => v.visit_bool(b),
//             _ => Err(de::Error::custom("bool")),
//         }
//     }

//     fn deserialize_enum<V>(
//         self,
//         _name: &str,
//         _variants: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: serde::de::Visitor<'de>,
//     {
//         use SlotValue::*;
//         match self.0 {
//             Str(s) => visitor.visit_enum(s.into_deserializer()),
//             U64(x) => {
//                 let v: u32 = x
//                     .try_into()
//                     .map_err(|_| de::Error::custom("enum index out of range (u64->u32)"))?;
//                 visitor.visit_enum(v.into_deserializer())
//             }
//             I64(x) => {
//                 let v: i32 = x
//                     .try_into()
//                     .map_err(|_| de::Error::custom("enum index out of range (i64->i32)"))?;
//                 let as_u32: u32 = v
//                     .try_into()
//                     .map_err(|_| de::Error::custom("enum index out of range (i32u32)"))?;
//                 visitor.visit_enum(as_u32.into_deserializer())
//             }
//             _ => Err(de::Error::custom("enum from non-scalar SlotValue")),
//         }
//     }

//     fn deserialize_string<V>(self, v: V) -> Result<V::Value, Self::Error>
//     where
//         V: serde::de::Visitor<'de>,
//     {
//         match self.0 {
//             SlotValue::Str(s) => v.visit_string(s.to_owned()),
//             SlotValue::StrId(_) => Err(de::Error::custom("interner not wired")),
//             _ => Err(de::Error::custom("string")),
//         }
//     }
//     fn deserialize_str<V>(self, v: V) -> Result<V::Value, Self::Error>
//     where
//         V: serde::de::Visitor<'de>,
//     {
//         match self.0 {
//             SlotValue::Str(s) => v.visit_borrowed_str(s),
//             _ => Err(de::Error::custom("&str")),
//         }
//     }
//     fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: serde::de::Visitor<'de>,
//     {
//         match self.0 {
//             SlotValue::Missing => visitor.visit_none(),
//             _ => visitor.visit_some(SlotValueDeserializer(self.0)),
//         }
//     }

//     serde::forward_to_deserialize_any! {
//         i8 i16 i32 u8 u16 u32 f32 char bytes byte_buf
//         unit unit_struct newtype_struct seq tuple tuple_struct map struct identifier ignored_any
//     }

//     fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         let _ = visitor;
//         Err(de::Error::custom("i128 is not supported"))
//     }

//     fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         let _ = visitor;
//         Err(de::Error::custom("u128 is not supported"))
//     }

//     fn is_human_readable(&self) -> bool {
//         true
//     }

//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         use SlotValue::*;
//         match self.0 {
//             U64(x) => visitor.visit_u64(x),
//             I64(x) => visitor.visit_i64(x),
//             F64(x) => visitor.visit_f64(x),
//             Bool(b) => visitor.visit_bool(b),
//             Str(s) => visitor.visit_str(s),
//             Missing => visitor.visit_unit(),
//             StrId(_) => Err(de::Error::custom("StrId requires interner")),
//             IPv4(ip) => visitor.visit_u32(ip),
//             IPv6(ip) => visitor.visit_u128(ip),
//             DateTime64 { epoch, .. } => visitor.visit_i64(epoch),
//             Uuid(bytes) => visitor.visit_bytes(&bytes),
//         }
//     }
// }

// impl<'de> IntoDeserializer<'de, de::value::Error> for SlotValueDeserializer {
//     type Deserializer = Self;
//     fn into_deserializer(self) -> Self::Deserializer {
//         self
//     }
// }

// pub struct RowpackStructDe<'a> {
//     pub row: &'a [SlotValue],
//     pub field_names: &'a [&'a str],
// }

// impl<'de, 'a> serde::de::Deserializer<'de> for RowpackStructDe<'a> {
//     type Error = de::value::Error;
//     fn deserialize_struct<V>(
//         self,
//         _: &'static str,
//         _: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: serde::de::Visitor<'de>,
//     {
//         let iter = (0..self.field_names.len()).map(|i| {
//             (
//                 self.field_names[i].into_deserializer(),
//                 SlotValueDeserializer(self.row[i]),
//             )
//         });
//         serde::de::value::MapDeserializer::new(iter).deserialize_any(visitor)
//     }
//     fn deserialize_map<V>(self, v: V) -> Result<V::Value, Self::Error>
//     where
//         V: serde::de::Visitor<'de>,
//     {
//         self.deserialize_struct("", &[], v)
//     }

//     serde::forward_to_deserialize_any! {
//        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
//         option unit unit_struct newtype_struct seq tuple tuple_struct enum identifier ignored_any
//     }

//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         let _ = visitor;
//         Err(de::Error::custom("any is not supported"))
//     }
// }

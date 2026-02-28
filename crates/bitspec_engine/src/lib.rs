pub mod pack;
pub mod policy;
pub mod predicate;
pub mod threshold;

use std::collections::HashMap;

use crate::predicate::PredOp;
use ben_wire::slot::SlotValue;

pub type Bit = u16;

pub type BitMask = u64;

pub type FactMap = HashMap<&'static str, FactValue>;

#[derive(Debug, Clone)]
pub enum FactValue {
    Bool(bool),
    Num(f64),
    Str(String),
}

#[derive(Debug)]
pub struct Rule<'a> {
    pub name: &'static str,
    pub pred: PredOp<'a>,
    pub field_idx: usize,
}

pub trait RowAccess {
    fn get_slot(&self, field_id: &str) -> Option<&SlotValue>;

    #[inline]
    fn get_f64(&self, field_id: &str) -> Option<f64> {
        let s = self.get_slot(field_id)?;
        s.as_f64().ok().copied()
    }

    #[inline]
    fn get_u64(&self, field_id: &str) -> Option<u64> {
        let s = self.get_slot(field_id)?;
        s.as_u64().ok().copied()
    }

    #[inline]
    fn get_i64(&self, field_id: &str) -> Option<i64> {
        let s = self.get_slot(field_id)?;
        s.as_i64().ok().copied()
    }

    #[inline]
    fn get_str(&self, field_id: &str) -> Option<&str> {
        let s = self.get_slot(field_id)?;
        s.as_str().ok()
    }

    #[inline]
    fn get_bool(&self, field_id: &str) -> Option<bool> {
        let s = self.get_slot(field_id)?;
        s.as_bool().ok().copied()
    }
}

use ben_wire::slot::SlotValue;

#[derive(Debug, Clone)]
pub enum PredOp<'a> {
    EqF64(f64),
    GtF64(f64),
    LtF64(f64),
    BetweenF64 { lo: f64, hi: f64 },

    EqBool(bool),

    EqStr(&'static str),
    NeStr(&'static str),
    StartsWith(&'static str),
    Contains(&'static str),

    ModEq { m: u64, r: u64 },
    ModNe { m: u64, r: u64 },

    All(&'a [PredOp<'a>]),
    Any(&'a [PredOp<'a>]),
    Not(&'a PredOp<'a>),
}

impl<'a> PredOp<'a> {
    pub fn eval(&self, slot: &SlotValue) -> bool {
        match self {
            PredOp::EqF64(v) => slot.as_f64().map(|x| x == v).unwrap_or(false),
            PredOp::GtF64(v) => slot.as_f64().map(|x| x > v).unwrap_or(false),
            PredOp::LtF64(v) => slot.as_f64().map(|x| x < v).unwrap_or(false),
            PredOp::BetweenF64 { lo, hi } => {
                slot.as_f64().map(|x| x >= lo && x <= hi).unwrap_or(false)
            }

            PredOp::EqBool(v) => slot.as_bool().map(|x| *x == *v).unwrap_or(false),

            PredOp::EqStr(s) => slot.as_str().map(|x| x == *s).unwrap_or(false),
            PredOp::NeStr(s) => slot.as_str().map(|x| x != *s).unwrap_or(false),
            PredOp::StartsWith(s) => slot.as_str().map(|x| x.starts_with(s)).unwrap_or(false),
            PredOp::Contains(s) => slot.as_str().map(|x| x.contains(s)).unwrap_or(false),

            PredOp::ModEq { m, r } => slot.as_u64().map(|x| x % *m == *r).unwrap_or(false),
            PredOp::ModNe { m, r } => slot.as_u64().map(|x| x % *m != *r).unwrap_or(false),

            PredOp::All(list) => list.iter().all(|p| p.eval(slot)),
            PredOp::Any(list) => list.iter().any(|p| p.eval(slot)),
            PredOp::Not(inner) => !inner.eval(slot),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PredicateSpec<'a> {
    pub field_id: &'static str,
    pub bit: u64,
    pub op: PredOp<'a>,
}

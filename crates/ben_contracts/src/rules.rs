pub type Bits = u128;

#[inline]
pub const fn bit(i: u8) -> Bits {
    1u128 << i
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Pass,
    Sample(u8),
    Quarantine(&'static str),
    RouteC,
    RouteD,
    Pause,
}

#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub all: Bits,
    pub any: Bits,
    pub none: Bits,
    pub action: Action,
    pub priority: u8,
    pub id: u32,
}

#[inline]
pub fn eval(bits: Bits, rules: &[Rule]) -> Action {
    for r in rules {
        if (bits & r.all) == r.all && (r.any == 0 || (bits & r.any) != 0) && (bits & r.none) == 0 {
            return r.action;
        }
    }
    Action::Pass
}

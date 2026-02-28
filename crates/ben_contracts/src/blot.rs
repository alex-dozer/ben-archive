#[derive(Debug, Clone)]
pub enum BlotOp {
    MaskAll,

    MaskSuffix { keep: usize },

    MaskPrefix { keep: usize },

    HashSha256,

    Drop,

    Truncate { len: usize },
}

#[derive(Debug, Clone)]
pub struct BlotFieldRule {
    pub field: &'static str,

    pub rule: Option<&'static str>,

    pub op: BlotOp,

    pub note: Option<&'static str>,
}

pub trait BlotSpec {
    const BLOT_RULES: &'static [BlotFieldRule];
}

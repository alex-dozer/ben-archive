use crate::{BitMask, FactMap, FactValue, RowAccess};

#[derive(Debug, Clone)]
pub enum ThresholdOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
}

#[derive(Debug, Clone)]
pub struct ThresholdLevel {
    pub name: &'static str,
    pub value: f64,
    pub bit: u16, // bit index
}

#[derive(Debug, Clone)]
pub struct ThresholdSpec {
    pub field_id: &'static str,

    pub fact_key: Option<&'static str>,

    pub flags: &'static [(&'static str, &'static [&'static str])],

    pub threshold_op: ThresholdOp,

    /// Names + thresholds
    pub levels: &'static [ThresholdLevel],
}

impl ThresholdSpec {
    pub fn eval<R: RowAccess + ?Sized>(
        &self,
        row: &R,
        out_mask: &mut BitMask,
        out_facts: &mut FactMap,
    ) {
        let Some(val) = row.get_f64(self.field_id) else {
            return;
        };

        let mut triggered: Vec<&'static str> = Vec::new();

        for lvl in self.levels {
            let pass = match self.threshold_op {
                ThresholdOp::Gt => val > lvl.value,
                ThresholdOp::Gte => val >= lvl.value,
                ThresholdOp::Lt => val < lvl.value,
                ThresholdOp::Lte => val <= lvl.value,
                ThresholdOp::Eq => (val - lvl.value).abs() < f64::EPSILON,
            };

            if pass {
                *out_mask |= (1u64 << lvl.bit);
                triggered.push(lvl.name);
            }
        }

        // Fact Key  best level
        if let Some(key) = self.fact_key {
            if let Some(best) = best_level(self.levels, &triggered) {
                out_facts.insert(key, FactValue::Str(best.to_string()));
            }
        }

        // Flags
        for (flag, lvl_list) in self.flags {
            // collect triggered levels for this flag only
            let mut relevant_triggered = Vec::new();
            for lvl_name in lvl_list.iter() {
                if triggered.contains(lvl_name) {
                    relevant_triggered.push(*lvl_name);
                }
            }

            if let Some(best) = best_level(self.levels, &relevant_triggered) {
                out_facts.insert(*flag, FactValue::Str(best.to_string()));
            }
        }
    }
}

/// Highest-value matching threshold
fn best_level<'a>(levels: &'a [ThresholdLevel], triggered: &[&'static str]) -> Option<&'a str> {
    let mut best: Option<(&'a str, f64)> = None;

    for lvl in levels {
        if triggered.contains(&lvl.name) {
            match best {
                None => best = Some((lvl.name, lvl.value)),
                Some((_, cur)) if lvl.value > cur => best = Some((lvl.name, lvl.value)),
                _ => {}
            }
        }
    }

    best.map(|(name, _)| name)
}

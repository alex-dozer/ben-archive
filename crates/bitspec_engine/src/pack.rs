use crate::{BitMask, FactMap, RowAccess, predicate::PredicateSpec, threshold::ThresholdSpec};

#[derive(Debug)]
pub struct BitspecPack {
    pub predicates: &'static [PredicateSpec<'static>],
    pub thresholds: &'static [ThresholdSpec],
    pub bit_count: u16,
}

impl BitspecPack {
    pub fn eval<R: RowAccess + ?Sized>(&self, row: &R) -> (BitMask, FactMap) {
        let mut mask: BitMask = 0;
        let mut facts = FactMap::default();

        self.eval_preds(row, &mut mask);
        self.eval_thresh(row, &mut mask, &mut facts);

        (mask, facts)
    }

    fn eval_preds<R: RowAccess + ?Sized>(&self, row: &R, mask: &mut BitMask) {
        for pred in self.predicates {
            if let Some(slot) = row.get_slot(pred.field_id) {
                if pred.op.eval(slot) {
                    *mask |= pred.bit;
                }
            }
        }
    }

    fn eval_thresh<R: RowAccess + ?Sized>(&self, row: &R, mask: &mut BitMask, facts: &mut FactMap) {
        for th in self.thresholds {
            th.eval(row, mask, facts);
        }
    }
}

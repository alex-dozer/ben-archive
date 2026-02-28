use ben_contracts::{BlotFieldRule, BlotOp};

pub trait RowGet {
    fn get_str(&self, field: &str) -> Option<&str>;
    fn get_owned(&self, field: &str) -> Option<String> {
        self.get_str(field).map(|s| s.to_owned())
    }
}

pub trait RowPut {
    fn put_str(&mut self, field: &str, value: String);
}

pub struct BlotEngine<'a> {
    rules: &'a [BlotFieldRule],
}

impl<'a> BlotEngine<'a> {
    pub fn new(rules: &'a [BlotFieldRule]) -> Self {
        Self { rules }
    }

    pub fn apply<R, W>(&self, input: &R, output: &mut W)
    where
        R: RowGet + ?Sized,
        W: RowPut + ?Sized,
    {
        for rule in self.rules {
            if let Some(raw) = input.get_str(rule.field) {
                let sanitized = apply_op(raw, &rule.op);
                output.put_str(rule.field, sanitized);
            }
        }
    }
}

fn apply_op(raw: &str, op: &BlotOp) -> String {
    match op {
        BlotOp::MaskAll => "*".repeat(raw.len()),

        BlotOp::MaskSuffix { keep } => {
            let keep = *keep as usize;
            if raw.len() <= keep {
                raw.to_owned()
            } else {
                "*".repeat(raw.len() - keep) + &raw[raw.len() - keep..]
            }
        }

        BlotOp::MaskPrefix { keep } => {
            let keep = *keep as usize;
            if raw.len() <= keep {
                raw.to_owned()
            } else {
                let prefix = &raw[..keep];
                let masked = "*".repeat(raw.len() - keep);
                format!("{prefix}{masked}")
            }
        }

        BlotOp::Truncate { len } => {
            let len = *len as usize;
            raw.chars().take(len).collect()
        }

        BlotOp::HashSha256 => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(raw.as_bytes());
            let bytes = hasher.finalize();
            hex::encode(bytes)
        }

        BlotOp::Drop => String::new(),
    }
}

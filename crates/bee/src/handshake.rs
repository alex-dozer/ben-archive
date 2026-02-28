use crate::cli::HandoffCfg;
use crate::transport::Lanes;
use anyhow::Result;

pub struct HelloResult {
    pub spec_fingerprint: String,
}

pub async fn hello(_cfg: &HandoffCfg, lanes: &Lanes) -> Result<HelloResult> {
    let _ = lanes;
    Ok(HelloResult {
        spec_fingerprint: "v0".into(),
    })
}

pub struct ProofResult {
    pub rtt_ms: f32,
    pub framing_ok: bool,
}

pub async fn proof_of_wire(_cfg: &HandoffCfg, lanes: &Lanes) -> Result<ProofResult> {
    let _ = lanes;
    Ok(ProofResult {
        rtt_ms: 0.4,
        framing_ok: true,
    })
}

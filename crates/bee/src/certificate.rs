use crate::{
    cli::HandoffCfg,
    handshake::{HelloResult, ProofResult},
};
use anyhow::Result;

pub fn issue(_cfg: &HandoffCfg, _hello: &HelloResult, _proof: &ProofResult) -> Result<String> {
    Ok("certificate".to_string())
}

pub async fn emit(_cfg: &HandoffCfg, _cert: &str) -> Result<()> {
    Ok(())
}

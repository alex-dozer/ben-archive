mod certificate;
mod cli;
mod diagnostics;
mod handoff;
mod handshake;
mod narrative;
mod policy;
mod shutdown;
mod transport;

use clap::Parser;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = cli::Args::parse();
    match args.command {
        cli::Command::Handoff(cfg) => run_handoff(cfg).await?,
        cli::Command::DiagSelftest => run_selftest().await?,
        cli::Command::PrintPolicy => print_policy()?,
    }
    Ok(())
}

async fn run_handoff(cfg: cli::HandoffCfg) -> anyhow::Result<()> {
    let id = narrative::alloc_id();
    narrative::greet(&id, cfg.pid, &cfg.mode, &cfg.os);
    let lanes = transport::open_lanes(&cfg).await?;
    let hello = handshake::hello(&cfg, &lanes).await?;
    let proof = handshake::proof_of_wire(&cfg, &lanes).await?;
    let cert = certificate::issue(&cfg, &hello, &proof)?;
    certificate::emit(&cfg, &cert).await?;
    narrative::retire(&id, &cert);
    Ok(())
}

async fn run_selftest() -> anyhow::Result<()> {
    Ok(())
}

fn print_policy() -> anyhow::Result<()> {
    Ok(())
}

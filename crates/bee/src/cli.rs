use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "bee", version, about = "Ben Escrow Emissary")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Handoff(HandoffCfg),
    DiagSelftest,
    PrintPolicy,
}

#[derive(clap::Args, Debug, Clone)]
pub struct HandoffCfg {
    #[arg(long, value_enum)]
    pub mode: Mode,
    #[arg(long, value_enum)]
    pub os: OsType,
    #[arg(long)]
    pub pid: u32,
    #[arg(long)]
    pub policy: String,
    #[arg(long)]
    pub clickhouse_url: Option<String>,

    // UDS/FD
    #[arg(long)]
    pub fd_diag: Option<i32>,
    #[arg(long)]
    pub fd_event: Option<i32>,
    #[arg(long)]
    pub fd_ctrl: Option<i32>,

    // TCP
    #[arg(long)]
    pub host: Option<String>,
    #[arg(long)]
    pub port_diag: Option<u16>,
    #[arg(long)]
    pub port_event: Option<u16>,
    #[arg(long)]
    pub port_ctrl: Option<u16>,

    // future mux
    #[arg(long, value_enum, default_value_t = Mux::None)]
    pub mux: Mux,
    #[arg(long)]
    pub port: Option<u16>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Mode {
    Uds,
    Tcp,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OsType {
    Linux,
    Macos,
    Windows,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Mux {
    None,
    Yamux,
    Quic,
}

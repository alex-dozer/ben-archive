use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => cmd_init(path),
        Commands::Build { path } => cmd_build(path),
        Commands::Inspect { path, kind } => cmd_inspect(path, kind),
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "benctl",
    version,
    about = "Ben control utility — scaffolding, inspection, and build tooling"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init {
        #[arg(default_value = "ben_crate")]
        path: PathBuf,
    },

    Build {
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    Inspect {
        #[arg(default_value = ".")]
        path: PathBuf,

        #[arg(default_value = "summary")]
        kind: InspectKind,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum InspectKind {
    Summary,
    Schemas,
    Signals,
    Caps,
}

fn cmd_init(path: PathBuf) -> Result<()> {
    println!("Initializing Ben crate at {}", path.display());
    Ok(())
}

fn cmd_build(path: PathBuf) -> Result<()> {
    println!("Building Ben artifacts from {}", path.display());
    Ok(())
}

fn cmd_inspect(path: PathBuf, kind: InspectKind) -> Result<()> {
    println!("Inspecting {:?} artifacts in {}", kind, path.display());

    match kind {
        InspectKind::Summary => {
            println!("(summary not implemented yet)");
        }
        InspectKind::Schemas => {
            println!("(schemas not implemented yet)");
        }
        InspectKind::Signals => {
            println!("(signals not implemented yet)");
        }
        InspectKind::Caps => {
            println!("(caps not implemented yet)");
        }
    }

    Ok(())
}

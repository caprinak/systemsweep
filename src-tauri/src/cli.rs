// src-tauri/src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "systemsweep-cli")]
#[command(about = "SystemSweep Command Line Interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan {
        #[arg(short, long)]
        path: String,
    },
    Clean {
        #[arg(short, long)]
        id: String,
    },
}

fn main() {
    let _cli = Cli::parse();
    println!("SystemSweep CLI - Coming soon");
}

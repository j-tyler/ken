use clap::{Parser, Subcommand};

mod commands;
mod storage;
mod session;
mod error;

use error::Result;

#[derive(Parser)]
#[command(name = "ken")]
#[command(about = "Durable workflow system for AI agent self-orchestration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new ken project (creates .ken/ken.db)
    Init,

    /// Wake a new session with a kenning and task
    Wake {
        /// Path to the ken (e.g., "core/cli")
        ken: String,

        /// Task description
        #[arg(short, long)]
        task: String,
    },

    /// Process agent request (used by agents to communicate with ken)
    Request {
        /// JSON request body
        json: String,
    },

    /// Evaluate triggers and spawn one pending session
    Process,

    /// Show current session status
    Status,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Wake { ken, task } => commands::wake::run(&ken, &task),
        Commands::Request { json } => commands::request::run(&json),
        Commands::Process => commands::process::run(),
        Commands::Status => commands::status::run(),
    }
}

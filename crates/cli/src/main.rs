use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use tracing::error;

mod commands;
mod utils;
mod config;

use commands::*;

#[derive(Parser)]
#[command(name = "tiffiny")]
#[command(about = "Tiffiny Studio - Media Databending and Processing Tool")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Tiffiny Studio Team")]
pub struct Cli {
    /Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long, default_value = "~/.config/tiffiny/config.toml")]
    pub config: PathBuf,

    #[arg(short, long, default_value = ".")]
    pub work_dir: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    Process {
        #[command(subcommand)]
        command: ProcessCommands,
    },
    Effects {
        #[command(subcommand)]
        command: EffectsCommands,
    },
    Convert {
        #[command(subcommand)]
        command: ConvertCommands,
    },
    Gpu {
        #[command(subcommand)]
        command: GpuCommands,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Utils {
        #[command(subcommand)]
        command: UtilsCommands,
    },
    Interactive {
        #[arg(short, long)]
        project: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    utils::init_logging(cli.verbose);

    let config = config::load_config(&cli.config).await?;

    std::env::set_current_dir(&cli.work_dir)?;

    match cli.command {
        Commands::Project { command } => project::handle(command, &config).await,
        Commands::Process { command } => process::handle(command, &config).await,
        Commands::Effects { command } => effects::handle(command, &config).await,
        Commands::Convert { command } => convert::handle(command, &config).await,
        Commands::Gpu { command } => gpu::handle(command, &config).await,
        Commands::Config { command } => config::handle(command, &config).await,
        Commands::Utils { command } => utils::handle(command, &config).await,
        Commands::Interactive { project } => interactive::handle(project, &config).await,
    }
}

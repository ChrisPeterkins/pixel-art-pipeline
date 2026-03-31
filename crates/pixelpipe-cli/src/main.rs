mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pixelpipe", version, about = "Pixel art asset pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file
    #[arg(short, long, default_value = "pixelpipe.yaml")]
    config: PathBuf,

    /// Increase log verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new pixelpipe.yaml config
    Init {
        /// Starter template to use
        #[arg(long, default_value = "minimal")]
        template: String,
    },
    /// Check config and inputs without producing output
    Validate,
    /// Run the full asset pipeline
    Build {
        /// Run only a specific phase
        #[arg(long)]
        only: Option<String>,
        /// Show what would be built without writing files
        #[arg(long)]
        dry_run: bool,
    },
    /// Watch input files and rebuild on changes
    Watch {
        /// Debounce interval in milliseconds
        #[arg(long, default_value = "300")]
        debounce: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.quiet {
        "error"
    } else {
        match cli.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    let result = match cli.command {
        Commands::Init { template } => commands::init::run(&template),
        Commands::Validate => commands::validate::run(&cli.config),
        Commands::Build { only, dry_run } => commands::build::run(&cli.config, only, dry_run),
        Commands::Watch { debounce } => commands::watch::run(&cli.config, debounce),
    };

    if let Err(e) = result {
        eprintln!("{} {}", colored::Colorize::red("error:"), e);
        std::process::exit(1);
    }
}

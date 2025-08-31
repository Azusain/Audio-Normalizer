mod audio_processor;
mod fade;
mod normalizer;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info, debug};
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

#[derive(Parser)]
#[command(
    name = "audio_normalizer",
    about = "A command-line tool for audio normalization with fade effects",
    version = "2.0.0"
)]
struct Cli {
    /// Enable verbose output (debug level logging)
    #[arg(short, long)]
    verbose: bool,

    /// Enable quiet mode (error level logging only)
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Input audio file
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output audio file
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Target peak level in dB (default: -12.0)
    #[arg(short = 'm', long = "max-peak", default_value = "-12.0")]
    max_peak: f64,

    /// Target LUFS level for loudness normalization
    #[arg(short = 'l', long = "lufs")]
    lufs: Option<f64>,

    /// Fade in duration in seconds
    #[arg(long = "fade-in", default_value = "0.0")]
    fade_in: f64,

    /// Fade out duration in seconds
    #[arg(long = "fade-out", default_value = "0.0")]
    fade_out: f64,

    /// Fade curve type (linear, exponential, logarithmic)
    #[arg(long = "fade-curve", default_value = "linear")]
    fade_curve: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Only show peak level of input file (no normalization)
    Peak {
        /// Input audio file to analyze
        input: PathBuf,
    },
    /// Only show LUFS level of input file (no normalization)
    MeasureLufs {
        /// Input audio file to analyze
        input: PathBuf,
    },
    /// Normalize audio with optional fade effects
    Normalize(NormalizeArgs),
}

#[derive(Args)]
struct NormalizeArgs {
    /// Input audio file
    input: PathBuf,
    
    /// Output audio file
    output: PathBuf,
    
    /// Target peak level in dB (default: -12.0)
    #[arg(short = 'm', long = "max-peak", default_value = "-12.0")]
    max_peak: f64,

    /// Target LUFS level for loudness normalization
    #[arg(short = 'l', long = "lufs")]
    lufs: Option<f64>,

    /// Fade in duration in seconds
    #[arg(long = "fade-in", default_value = "0.0")]
    fade_in: f64,

    /// Fade out duration in seconds
    #[arg(long = "fade-out", default_value = "0.0")]
    fade_out: f64,

    /// Fade curve type (linear, exponential, logarithmic)
    #[arg(long = "fade-curve", default_value = "linear")]
    fade_curve: String,
}

fn setup_logging(verbose: bool, quiet: bool) {
    let level = if verbose {
        LevelFilter::DEBUG
    } else if quiet {
        LevelFilter::ERROR
    } else {
        LevelFilter::INFO
    };

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_level(true)
                .compact(),
        )
        .with(level)
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    setup_logging(cli.verbose, cli.quiet);

    debug!("Audio Normalizer v2.0.0");

    match cli.command {
        Some(Commands::Peak { input }) => {
            info!("Analyzing peak level of: {}", input.display());
            let peak_db = audio_processor::get_peak_level(&input)?;
            info!("Peak level: {:.2} dB", peak_db);
        }
        Some(Commands::MeasureLufs { input }) => {
            info!("Analyzing LUFS level of: {}", input.display());
            let lufs_level = audio_processor::get_lufs_level(&input)?;
            info!("LUFS level: {:.2} LUFS", lufs_level);
        }
        Some(Commands::Normalize(args)) => {
            process_normalization(
                &args.input,
                &args.output,
                args.max_peak,
                args.lufs,
                args.fade_in,
                args.fade_out,
                &args.fade_curve,
            )?;
        }
        None => {
            // Handle legacy command-line format
            match (cli.input, cli.output) {
                (Some(input), Some(output)) => {
                    process_normalization(
                        &input,
                        &output,
                        cli.max_peak,
                        cli.lufs,
                        cli.fade_in,
                        cli.fade_out,
                        &cli.fade_curve,
                    )?;
                }
                (Some(input), None) => {
                    // Default to peak analysis
                    info!("Analyzing peak level of: {}", input.display());
                    let peak_db = audio_processor::get_peak_level(&input)?;
                    info!("Peak level: {:.2} dB", peak_db);
                }
                _ => {
                    error!("Input file is required");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn process_normalization(
    input: &PathBuf,
    output: &PathBuf,
    max_peak: f64,
    lufs: Option<f64>,
    fade_in: f64,
    fade_out: f64,
    fade_curve: &str,
) -> Result<()> {
    debug!("Input file: {}", input.display());
    debug!("Output file: {}", output.display());

    if let Some(target_lufs) = lufs {
        debug!("Target LUFS level: {:.2} LUFS", target_lufs);
        normalizer::normalize_lufs(input, output, target_lufs, fade_in, fade_out, fade_curve)?;
        info!("LUFS normalization completed: {} -> {} (target: {:.2} LUFS)", 
              input.display(), output.display(), target_lufs);
    } else {
        debug!("Target peak level: {:.2} dB", max_peak);
        normalizer::normalize_peak(input, output, max_peak, fade_in, fade_out, fade_curve)?;
        info!("Peak normalization completed: {} -> {} (target: {:.2} dB)", 
              input.display(), output.display(), max_peak);
    }

    if fade_in > 0.0 || fade_out > 0.0 {
        info!("Applied fades: in={:.2}s, out={:.2}s, curve={}", fade_in, fade_out, fade_curve);
    }

    Ok(())
}

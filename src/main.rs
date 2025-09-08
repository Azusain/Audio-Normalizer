mod audio_processor;
mod fade;
mod normalizer;
mod multi_format_processor;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, debug};
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

#[derive(Parser)]
#[command(
    name = "audio_normalizer",
    about = "A command-line tool for audio normalization with fade effects",
    version = "2.0.0"
)]
struct Cli {
    /// Input audio file
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output audio file (optional - if not provided, will only analyze)
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Enable verbose output (debug level logging)
    #[arg(short, long)]
    verbose: bool,

    /// Enable quiet mode (error level logging only)
    #[arg(short, long)]
    quiet: bool,

    /// Only analyze peak level (no normalization)
    #[arg(long = "peak-only")]
    peak_only: bool,

    /// Only analyze LUFS level (no normalization)
    #[arg(long = "lufs-only")]
    lufs_only: bool,

    /// Target peak level in dB
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

    /// Force clipping if necessary to reach target LUFS (default: auto-adjust to prevent clipping)
    #[arg(long = "force-clip")]
    force_clip: bool,
}

fn setup_logging(verbose: bool, quiet: bool) {
    let level = if verbose {
        LevelFilter::DEBUG
    } else if quiet {
        LevelFilter::ERROR
    } else {
        LevelFilter::INFO // Change back to INFO so we can see processing messages
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

    // Peak analysis only
    if cli.peak_only {
        info!("Analyzing peak level of: {}", cli.input.display());
        let peak_db = multi_format_processor::MultiFormatProcessor::get_peak_level(&cli.input)?;
        println!("Peak level: {:.2} dB", peak_db);
        return Ok(());
    }

    // LUFS analysis only
    if cli.lufs_only {
        info!("Analyzing LUFS level of: {}", cli.input.display());
        let lufs_level = multi_format_processor::MultiFormatProcessor::get_lufs_level(&cli.input)?;
        println!("LUFS level: {:.2} LUFS", lufs_level);
        return Ok(());
    }

    // Normalization or simple analysis
    match cli.output {
        Some(output) => {
            // Normalize audio
            process_normalization(
                &cli.input,
                &output,
                cli.max_peak,
                cli.lufs,
                cli.fade_in,
                cli.fade_out,
                &cli.fade_curve,
                cli.force_clip,
            )?;
        }
        None => {
            // Just analyze peak level
            info!("Analyzing peak level of: {}", cli.input.display());
            let peak_db = multi_format_processor::MultiFormatProcessor::get_peak_level(&cli.input)?;
            println!("Peak level: {:.2} dB", peak_db);
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
    force_clip: bool,
) -> Result<()> {
    debug!("Input file: {}", input.display());
    debug!("Output file: {}", output.display());

    if let Some(target_lufs) = lufs {
        debug!("Target LUFS level: {:.2} LUFS", target_lufs);
        normalizer::normalize_lufs(input, output, target_lufs, fade_in, fade_out, fade_curve, force_clip)?;
        println!("LUFS normalization completed: {} -> {} (target: {:.2} LUFS)", 
              input.display(), output.display(), target_lufs);
    } else {
        debug!("Target peak level: {:.2} dB", max_peak);
        normalizer::normalize_peak(input, output, max_peak, fade_in, fade_out, fade_curve)?;
        println!("Peak normalization completed: {} -> {} (target: {:.2} dB)", 
              input.display(), output.display(), max_peak);
    }

    if fade_in > 0.0 || fade_out > 0.0 {
        println!("Applied fades: in={:.2}s, out={:.2}s, curve={}", fade_in, fade_out, fade_curve);
    }

    Ok(())
}

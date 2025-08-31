use std::path::Path;
use anyhow::Result;
use crate::multi_format_processor::MultiFormatProcessor;
use crate::fade::{apply_fades, FadeCurve};

fn linear_to_db(x: f32) -> f32 { if x <= 0.0 { f32::NEG_INFINITY } else { 20.0 * x.log10() } }
fn db_to_linear(db: f32) -> f32 { (10.0f32).powf(db / 20.0) }

pub fn normalize_peak(input: &Path, output: &Path, target_peak_db: f64, fade_in: f64, fade_out: f64, curve: &str) -> Result<()> {
    // Decode input file using multi-format processor
    let mut audio_data = MultiFormatProcessor::decode_audio_to_f32(input)?;

    // Find current peak
    let mut peak = 0.0f32;
    for v in &audio_data.samples { peak = peak.max(v.abs()); }

    // Compute gain
    let current_peak_db = linear_to_db(peak);
    let gain_db = target_peak_db as f32 - current_peak_db;
    let gain = db_to_linear(gain_db);

    for v in &mut audio_data.samples { *v = (*v * gain).clamp(-1.0, 1.0); }

    // Apply fades
    let curve = FadeCurve::from_str(curve);
    apply_fades(&mut audio_data.samples, audio_data.channels, audio_data.sample_rate, fade_in, fade_out, curve);

    // Write output in the format determined by file extension
    MultiFormatProcessor::write_audio_data(output, &audio_data, 16)?;
    Ok(())
}

pub fn normalize_lufs(input: &Path, output: &Path, target_lufs: f64, fade_in: f64, fade_out: f64, curve: &str) -> Result<()> {
    // Decode input file using multi-format processor
    let mut audio_data = MultiFormatProcessor::decode_audio_to_f32(input)?;

    // Measure LUFS using multi-format processor
    let current_lufs = MultiFormatProcessor::get_lufs_level(input)? as f32;
    let gain_db = (target_lufs as f32) - current_lufs;
    let gain = db_to_linear(gain_db);

    for v in &mut audio_data.samples { *v = (*v * gain).clamp(-1.0, 1.0); }

    // Apply fades
    let curve = FadeCurve::from_str(curve);
    apply_fades(&mut audio_data.samples, audio_data.channels, audio_data.sample_rate, fade_in, fade_out, curve);

    // Write output in the format determined by file extension
    MultiFormatProcessor::write_audio_data(output, &audio_data, 16)?;
    Ok(())
}


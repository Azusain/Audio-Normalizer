use anyhow::Result;
use hound::{SampleFormat};
use std::path::Path;

use crate::audio_processor::{read_wav_as_f32, write_wav, get_lufs_level};
use crate::fade::{apply_fades, FadeCurve};

fn linear_to_db(x: f32) -> f32 { if x <= 0.0 { f32::NEG_INFINITY } else { 20.0 * x.log10() } }
fn db_to_linear(db: f32) -> f32 { (10.0f32).powf(db / 20.0) }

pub fn normalize_peak(input: &Path, output: &Path, target_peak_db: f64, fade_in: f64, fade_out: f64, curve: &str) -> Result<()> {
    let (mut spec, mut data) = read_wav_as_f32(input)?;

    // Find current peak
    let mut peak = 0.0f32;
    for v in &data { peak = peak.max(v.abs()); }

    // Compute gain
    let current_peak_db = linear_to_db(peak);
    let gain_db = target_peak_db as f32 - current_peak_db;
    let gain = db_to_linear(gain_db);

    for v in &mut data { *v = (*v * gain).clamp(-1.0, 1.0); }

    // Apply fades
    let curve = FadeCurve::from_str(curve);
    apply_fades(&mut data, spec.channels as usize, spec.sample_rate as usize, fade_in, fade_out, curve);

    // Ensure format is valid; default to 16-bit PCM if int
    if spec.sample_format == SampleFormat::Int && spec.bits_per_sample == 0 {
        spec.bits_per_sample = 16;
    }

    write_wav(output, &spec, &data)?;
    Ok(())
}

pub fn normalize_lufs(input: &Path, output: &Path, target_lufs: f64, fade_in: f64, fade_out: f64, curve: &str) -> Result<()> {
    let (mut spec, mut data) = read_wav_as_f32(input)?;

    // Measure LUFS
    let current_lufs = get_lufs_level(input)? as f32;
    let gain_db = (target_lufs as f32) - current_lufs;
    let gain = db_to_linear(gain_db);

    for v in &mut data { *v = (*v * gain).clamp(-1.0, 1.0); }

    // Apply fades
    let curve = FadeCurve::from_str(curve);
    apply_fades(&mut data, spec.channels as usize, spec.sample_rate as usize, fade_in, fade_out, curve);

    // Ensure format is valid; default to 16-bit PCM if int
    if spec.sample_format == SampleFormat::Int && spec.bits_per_sample == 0 {
        spec.bits_per_sample = 16;
    }

    write_wav(output, &spec, &data)?;
    Ok(())
}


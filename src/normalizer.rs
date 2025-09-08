use std::path::Path;
use anyhow::Result;
use crate::multi_format_processor::MultiFormatProcessor;
use crate::fade::{apply_fades, FadeCurve};
use tracing::{info, warn};

fn linear_to_db(x: f32) -> f32 { if x <= 0.0 { f32::NEG_INFINITY } else { 20.0 * x.log10() } }
fn db_to_linear(db: f32) -> f32 { (10.0f32).powf(db / 20.0) }

/// Calculate the maximum safe LUFS target that won't cause clipping
fn calculate_max_safe_lufs(audio_data: &[f32], current_lufs: f32) -> f32 {
    // Find current peak
    let mut peak = 0.0f32;
    for v in audio_data {
        peak = peak.max(v.abs());
    }
    
    if peak <= 0.0 {
        return current_lufs; // No audio signal, no adjustment needed
    }
    
    // Calculate how much headroom we have before clipping (in dB)
    let current_peak_db = linear_to_db(peak);
    let headroom_db = 0.0 - current_peak_db; // headroom to 0 dBFS
    
    // Apply a small safety margin (0.5 dB) to avoid accidental clipping
    let safe_headroom_db = headroom_db - 0.5;
    
    // Maximum safe LUFS is current LUFS plus available headroom
    current_lufs + safe_headroom_db
}

/// Check if applying LUFS gain would cause clipping
fn would_clip_with_lufs_gain(audio_data: &[f32], lufs_gain_db: f32) -> bool {
    let gain = db_to_linear(lufs_gain_db);
    
    for v in audio_data {
        if (v * gain).abs() > 1.0 {
            return true;
        }
    }
    false
}

/// Results of clipping analysis
#[derive(Debug)]
pub struct ClippingAnalysis {
    pub would_clip: bool,
    pub max_safe_lufs: f32,
    pub current_peak_db: f32,
    pub headroom_db: f32,
}

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

pub fn normalize_lufs(input: &Path, output: &Path, target_lufs: f64, fade_in: f64, fade_out: f64, curve: &str, force_clip: bool) -> Result<()> {
    // Decode input file using multi-format processor
    let mut audio_data = MultiFormatProcessor::decode_audio_to_f32(input)?;

    // Measure LUFS using multi-format processor
    let current_lufs = MultiFormatProcessor::get_lufs_level(input)? as f32;
    let target_lufs_f32 = target_lufs as f32;
    let requested_gain_db = target_lufs_f32 - current_lufs;
    
    // Perform clipping analysis
    let analysis = analyze_clipping_risk(&audio_data.samples, current_lufs, target_lufs_f32);
    
    let (final_target_lufs, actual_gain_db) = if analysis.would_clip && !force_clip {
        // Clipping would occur and user didn't force it - adjust to safe level
        let safe_lufs = analysis.max_safe_lufs;
        let safe_gain_db = safe_lufs - current_lufs;
        
        warn!("target LUFS {:.2} would cause clipping (current peak: {:.2} dB, headroom: {:.2} dB)", 
              target_lufs_f32, analysis.current_peak_db, analysis.headroom_db);
        info!("automatically adjusted to maximum safe LUFS: {:.2} (gain: {:.2} dB)", 
              safe_lufs, safe_gain_db);
        info!("use --force-clip to override this safety check");
        
        (safe_lufs, safe_gain_db)
    } else if analysis.would_clip && force_clip {
        // User explicitly requested clipping
        warn!("forcing clipping: target LUFS {:.2} will cause clipping (current peak: {:.2} dB, headroom: {:.2} dB)", 
              target_lufs_f32, analysis.current_peak_db, analysis.headroom_db);
        warn!("audio quality will be degraded due to clipping artifacts");
        
        (target_lufs_f32, requested_gain_db)
    } else {
        // No clipping risk, proceed normally
        info!("target LUFS {:.2} is safe (current peak: {:.2} dB, headroom: {:.2} dB)", 
              target_lufs_f32, analysis.current_peak_db, analysis.headroom_db);
        
        (target_lufs_f32, requested_gain_db)
    };
    
    // Apply the calculated gain
    let gain = db_to_linear(actual_gain_db);
    for v in &mut audio_data.samples { 
        *v = (*v * gain).clamp(-1.0, 1.0); 
    }

    // Apply fades
    let curve = FadeCurve::from_str(curve);
    apply_fades(&mut audio_data.samples, audio_data.channels, audio_data.sample_rate, fade_in, fade_out, curve);

    // Write output in the format determined by file extension
    MultiFormatProcessor::write_audio_data(output, &audio_data, 16)?;
    
    // Report final results
    if analysis.would_clip && !force_clip {
        println!("LUFS normalization completed with safety adjustment:");
        println!("  requested: {:.2} LUFS -> actual: {:.2} LUFS (gain: {:.2} dB)", 
                target_lufs, final_target_lufs, actual_gain_db);
    } else {
        println!("LUFS normalization completed: {:.2} LUFS (gain: {:.2} dB)", 
                final_target_lufs, actual_gain_db);
    }
    
    Ok(())
}

/// Analyze clipping risk for LUFS normalization
fn analyze_clipping_risk(audio_data: &[f32], current_lufs: f32, target_lufs: f32) -> ClippingAnalysis {
    // Find current peak
    let mut peak = 0.0f32;
    for v in audio_data {
        peak = peak.max(v.abs());
    }
    
    let current_peak_db = if peak > 0.0 { linear_to_db(peak) } else { f32::NEG_INFINITY };
    let headroom_db = 0.0 - current_peak_db;
    
    // Calculate required gain and check for clipping
    let required_gain_db = target_lufs - current_lufs;
    let would_clip = would_clip_with_lufs_gain(audio_data, required_gain_db);
    
    // Calculate maximum safe LUFS
    let max_safe_lufs = calculate_max_safe_lufs(audio_data, current_lufs);
    
    ClippingAnalysis {
        would_clip,
        max_safe_lufs,
        current_peak_db,
        headroom_db,
    }
}


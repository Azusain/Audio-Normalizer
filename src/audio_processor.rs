use anyhow::{anyhow, Result};
use ebur128::EbuR128;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[allow(dead_code)]
pub fn get_peak_level(input: &Path) -> Result<f64> {
    let mut reader = WavReader::open(input)
        .map_err(|e| anyhow!("Failed to open input file: {}", e))?;

    let spec = reader.spec();

    // Read samples into f32
    let mut peak: f32 = 0.0;

    match spec.sample_format {
        SampleFormat::Float => {
            for s in reader.samples::<f32>() {
                let v = s?;
                let a = v.abs();
                if a > peak { peak = a; }
            }
        }
        SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            for s in reader.samples::<i32>() {
                let v = s? as f32 / max;
                let a = v.abs();
                if a > peak { peak = a; }
            }
        }
    }

    if peak <= 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    Ok(20.0 * (peak as f64).log10())
}

#[allow(dead_code)]
pub fn get_lufs_level(input: &Path) -> Result<f64> {
    let mut reader = WavReader::open(input)?;
    let spec = reader.spec();

    let rate = spec.sample_rate as u32;
    let ch = spec.channels as usize;

    let mut meter = EbuR128::new(ch as u32, rate, ebur128::Mode::I)?;

    match spec.sample_format {
        SampleFormat::Float => {
            let mut buf: Vec<f32> = Vec::new();
            for s in reader.samples::<f32>() { buf.push(s?); }
            meter.add_frames_f32(&buf)?;
        }
        SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            let mut buf: Vec<f32> = Vec::new();
            for s in reader.samples::<i32>() { buf.push(s? as f32 / max); }
            meter.add_frames_f32(&buf)?;
        }
    }

    let lufs = meter.loudness_global()?;
    Ok(lufs as f64)
}

pub fn write_wav(output: &Path, spec: &WavSpec, data: &[f32]) -> Result<()> {
    let file = File::create(output)?;
    let buf = BufWriter::new(file);
    let mut writer = WavWriter::new(buf, *spec)?;

    match spec.sample_format {
        SampleFormat::Float => {
            for v in data { 
                let clamped = v.clamp(-1.0, 1.0);
                writer.write_sample(clamped)?; 
            }
        }
        SampleFormat::Int => {
            // Calculate the proper scaling factor based on bit depth
            let scale_factor = match spec.bits_per_sample {
                8 => 127.0,
                16 => 32767.0,
                24 => 8388607.0,
                32 => 2147483647.0,
                _ => (1i64 << (spec.bits_per_sample - 1)) as f32 - 1.0,
            };
            
            for v in data { 
                let clamped = v.clamp(-1.0, 1.0);
                let scaled = (clamped * scale_factor).round();
                
                // Write the sample with proper type based on bit depth
                match spec.bits_per_sample {
                    8 => writer.write_sample(scaled as i8)?,
                    16 => writer.write_sample(scaled as i16)?,
                    24 => writer.write_sample(scaled as i32)?, // hound handles 24-bit as i32
                    32 => writer.write_sample(scaled as i32)?,
                    _ => writer.write_sample(scaled as i32)?, // fallback
                }
            }
        }
    }

    writer.finalize()?;
    Ok(())
}

#[allow(dead_code)]
pub fn read_wav_as_f32(path: &Path) -> Result<(WavSpec, Vec<f32>)> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    let mut data: Vec<f32> = Vec::new();
    match spec.sample_format {
        SampleFormat::Float => {
            for s in reader.samples::<f32>() { data.push(s?); }
        }
        SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            for s in reader.samples::<i32>() { data.push(s? as f32 / max); }
        }
    }
    Ok((spec, data))
}


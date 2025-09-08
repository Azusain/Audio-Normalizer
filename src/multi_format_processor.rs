use anyhow::{anyhow, Result};
use ebur128::EbuR128;
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tracing::warn;

/// Audio format detection and conversion
pub struct MultiFormatProcessor;

impl MultiFormatProcessor {
    /// Get peak level from any supported audio format
    pub fn get_peak_level(input: &Path) -> Result<f64> {
        let audio_data = Self::decode_audio_to_f32(input)?;
        let mut peak: f32 = 0.0;
        
        for sample in &audio_data.samples {
            let abs_sample = sample.abs();
            if abs_sample > peak {
                peak = abs_sample;
            }
        }
        
        if peak <= 0.0 {
            return Ok(f64::NEG_INFINITY);
        }
        Ok(20.0 * (peak as f64).log10())
    }
    
    /// Get LUFS level from any supported audio format
    pub fn get_lufs_level(input: &Path) -> Result<f64> {
        let audio_data = Self::decode_audio_to_f32(input)?;
        
        let mut meter = EbuR128::new(
            audio_data.channels as u32,
            audio_data.sample_rate as u32,
            ebur128::Mode::I,
        )?;
        
        meter.add_frames_f32(&audio_data.samples)?;
        let lufs = meter.loudness_global()?;
        Ok(lufs as f64)
    }
    
    /// Decode any supported audio format to f32 samples
    pub fn decode_audio_to_f32(input: &Path) -> Result<AudioData> {
        // First try WAV with hound (faster for WAV files)
        if let Some(ext) = input.extension() {
            if ext.to_string_lossy().to_lowercase() == "wav" {
                return Self::decode_wav_with_hound(input);
            }
        }
        
        // Use Symphonia for other formats
        Self::decode_with_symphonia(input)
    }
    
    /// Decode WAV files using hound (faster)
    fn decode_wav_with_hound(input: &Path) -> Result<AudioData> {
        let mut reader = hound::WavReader::open(input)
            .map_err(|e| anyhow!("Failed to open WAV file: {}", e))?;
        
        let spec = reader.spec();
        let mut samples = Vec::new();
        
        match spec.sample_format {
            hound::SampleFormat::Float => {
                for sample in reader.samples::<f32>() {
                    samples.push(sample?);
                }
            }
            hound::SampleFormat::Int => {
                let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
                for sample in reader.samples::<i32>() {
                    samples.push(sample? as f32 / max);
                }
            }
        }
        
        Ok(AudioData {
            samples,
            channels: spec.channels as usize,
            sample_rate: spec.sample_rate as usize,
        })
    }
    
    /// Decode using Symphonia (supports MP3, FLAC, OGG, etc.)
    fn decode_with_symphonia(input: &Path) -> Result<AudioData> {
        let file = std::fs::File::open(input)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        
        let mut hint = Hint::new();
        if let Some(extension) = input.extension() {
            hint.with_extension(&extension.to_string_lossy());
        }
        
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| anyhow!("Failed to probe format: {}", e))?;
        
        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("No supported audio tracks found"))?;
        
        let track_id = track.id;
        // Create decoder options with maximum tolerance for errors
        let mut dec_opts = DecoderOptions::default();
        // Disable all verification and error checking where possible
        dec_opts.verify = false;
        
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(|e| anyhow!("Failed to create decoder: {}", e))?;
        
        let mut samples = Vec::new();
        let mut sample_rate = 0;
        let mut channels = 0;
        
        // Decode all packets
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(_) => break,
            };
            
            if packet.track_id() != track_id {
                continue;
            }
            
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    if sample_rate == 0 {
                        let spec = decoded.spec();
                        sample_rate = spec.rate as usize;
                        channels = spec.channels.count();
                    }
                    
                    // Convert to f32 and interleave channels
                    let frame_count = decoded.frames();
                    let channel_count = decoded.spec().channels.count();
                    
                    match decoded {
                        AudioBufferRef::F32(buf) => {
                            // Interleave samples: [L,R,L,R,...] instead of [L,L,L...,R,R,R...]
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    samples.push(channel_data[frame]);
                                }
                            }
                        }
                        AudioBufferRef::U8(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    samples.push((sample as f32 - 128.0) / 127.0);
                                }
                            }
                        }
                        AudioBufferRef::U16(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    samples.push((sample as f32 - 32768.0) / 32767.0);
                                }
                            }
                        }
                        AudioBufferRef::U24(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    let val = sample.inner() as f32;
                                    samples.push((val - 8388608.0) / 8388607.0);
                                }
                            }
                        }
                        AudioBufferRef::U32(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    samples.push((sample as f32 - 2147483648.0) / 2147483647.0);
                                }
                            }
                        }
                        AudioBufferRef::S8(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    samples.push(sample as f32 / 127.0);
                                }
                            }
                        }
                        AudioBufferRef::S16(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    samples.push(sample as f32 / 32767.0);
                                }
                            }
                        }
                        AudioBufferRef::S24(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    let val = sample.inner() as f32;
                                    samples.push(val / 8388607.0);
                                }
                            }
                        }
                        AudioBufferRef::S32(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    samples.push(sample as f32 / 2147483647.0);
                                }
                            }
                        }
                        AudioBufferRef::F64(buf) => {
                            for frame in 0..frame_count {
                                for ch in 0..channel_count {
                                    let channel_data = buf.chan(ch);
                                    let sample = channel_data[frame];
                                    samples.push(sample as f32);
                                }
                            }
                        }
                    }
                }
                Err(symphonia::core::errors::Error::IoError(_)) => {
                    // End of stream
                    break;
                }
                Err(symphonia::core::errors::Error::DecodeError(err)) => {
                    // Log decode error but continue processing to preserve maximum audio data
                    warn!("decode error in packet (continuing): {}", err);
                    // Try to insert silence for the expected frame duration to maintain timing
                    if sample_rate > 0 && channels > 0 {
                        // Estimate typical packet duration (conservative estimate: 1024 samples per channel)
                        let estimated_samples_per_channel = 1024;
                        for _ in 0..(estimated_samples_per_channel * channels) {
                            samples.push(0.0); // Insert silence to maintain timing
                        }
                    }
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!("Decode error: {}", e));
                }
            }
        }
        
        if samples.is_empty() {
            return Err(anyhow!("No audio data decoded"));
        }
        
        Ok(AudioData {
            samples,
            channels,
            sample_rate,
        })
    }
    
    /// Write audio data to the appropriate format based on file extension
    pub fn write_audio_data(
        output: &Path,
        audio_data: &AudioData,
        bit_depth: u16,
    ) -> Result<()> {
        // Determine output format from file extension
        let extension = output.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        
        match extension.as_str() {
            "mp3" => Self::write_mp3(output, audio_data),
            "flac" => Self::write_flac(output, audio_data, bit_depth),
            "wav" | _ => Self::write_wav(output, audio_data, bit_depth), // Default to WAV
        }
    }
    
    
    /// Write audio data to WAV format
    fn write_wav(
        output: &Path,
        audio_data: &AudioData,
        bit_depth: u16,
    ) -> Result<()> {
        let spec = hound::WavSpec {
            channels: audio_data.channels as u16,
            sample_rate: audio_data.sample_rate as u32,
            bits_per_sample: bit_depth,
            sample_format: if bit_depth == 32 {
                hound::SampleFormat::Float
            } else {
                hound::SampleFormat::Int
            },
        };
        
        crate::audio_processor::write_wav(output, &spec, &audio_data.samples)
    }
    
    /// Write audio data to MP3 format
    fn write_mp3(output: &Path, _audio_data: &AudioData) -> Result<()> {
        // For now, convert MP3 output to WAV format
        // TODO: Implement proper MP3 encoding once lame API is figured out
        let wav_path = output.with_extension("wav");
        Self::write_wav(&wav_path, _audio_data, 16)?;
        
        // Could use external tool here if needed
        warn!("MP3 output requested but not yet implemented. Wrote WAV instead: {}", wav_path.display());
        Ok(())
    }
    
    /// Write audio data to FLAC format (currently outputs WAV)
    fn write_flac(
        output: &Path,
        audio_data: &AudioData,
        bit_depth: u16,
    ) -> Result<()> {
        // For now, output as high-quality WAV instead of FLAC
        // Use 24-bit for better quality than 16-bit
        let actual_bit_depth = if bit_depth < 24 { 24 } else { bit_depth };
        let wav_path = output.with_extension("wav");
        Self::write_wav(&wav_path, audio_data, actual_bit_depth)?;
        Ok(())
    }
}

/// Audio data structure
#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub sample_rate: usize,
}

impl AudioData {
    #[allow(dead_code)]
    pub fn duration_seconds(&self) -> f64 {
        self.samples.len() as f64 / (self.channels * self.sample_rate) as f64
    }
}

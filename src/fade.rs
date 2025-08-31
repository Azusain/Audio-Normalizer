#[derive(Clone, Copy, Debug)]
pub enum FadeCurve {
    Linear,
    Exponential,
    Logarithmic,
}

impl FadeCurve {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "exponential" | "exp" => FadeCurve::Exponential,
            "logarithmic" | "log" => FadeCurve::Logarithmic,
            _ => FadeCurve::Linear,
        }
    }

    pub fn in_gain(self, t: f32) -> f32 {
        match self {
            FadeCurve::Linear => t,
            FadeCurve::Exponential => t.powf(2.0),
            FadeCurve::Logarithmic => (t * 0.9 + 0.1).ln() / 0.1f32.ln(),
        }
    }

    pub fn out_gain(self, t: f32) -> f32 {
        match self {
            FadeCurve::Linear => 1.0 - t,
            FadeCurve::Exponential => (1.0 - t).powf(2.0),
            FadeCurve::Logarithmic => (1.0 - t * 0.9).ln() / 0.1f32.ln(),
        }
    }
}

pub fn apply_fades(samples: &mut [f32], channels: usize, sample_rate: usize, fade_in_s: f64, fade_out_s: f64, curve: FadeCurve) {
    let total_frames = samples.len() / channels;
    let fade_in_frames = (fade_in_s * sample_rate as f64).round() as usize;
    let fade_out_frames = (fade_out_s * sample_rate as f64).round() as usize;

    // Fade in
    if fade_in_frames > 0 {
        let n = fade_in_frames.min(total_frames);
        for f in 0..n {
            let t = f as f32 / n as f32; // 0..1
            let g = curve.in_gain(t);
            for c in 0..channels {
                let i = f * channels + c;
                samples[i] *= g;
            }
        }
    }

    // Fade out
    if fade_out_frames > 0 {
        let n = fade_out_frames.min(total_frames);
        for off in 0..n {
            let f = total_frames - n + off; // frame index
            let t = off as f32 / n as f32; // 0..1
            let g = curve.out_gain(t);
            for c in 0..channels {
                let i = f * channels + c;
                samples[i] *= g;
            }
        }
    }
}


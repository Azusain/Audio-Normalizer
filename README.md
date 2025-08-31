# Audio Normalizer (Rust)

A command-line tool for audio normalization rewritten in Rust. Supports both peak level normalization and LUFS (Loudness Units relative to Full Scale) normalization, plus fade-in and fade-out effects.

## Build

1. Install Rust and Cargo (https://rustup.rs/)
2. Build:
   ```bash
   cargo build --release
   ```

## Usage

```bash
# Peak normalization to -12 dB (default) with legacy style
audio_normalizer input.wav output.wav

# Peak normalization to -6 dB
audio_normalizer -m -6 input.wav output.wav

# LUFS normalization to -23 LUFS (broadcast standard)
audio_normalizer -l -23 input.wav output.wav

# Apply fades (1s fade-in, 2s fade-out)
audio_normalizer --fade-in 1.0 --fade-out 2.0 input.wav output.wav

# Choose fade curve (linear|exponential|logarithmic)
audio_normalizer --fade-in 1.0 --fade-out 1.0 --fade-curve exponential input.wav output.wav

# Show peak level only (subcommand)
audio_normalizer peak input.wav

# Show LUFS level only (subcommand)
audio_normalizer measure-lufs input.wav

# Verbose output
audio_normalizer -v input.wav output.wav
```

## Options

- `-m, --max-peak <dB>` - Target peak level (default: -12)
- `-l, --lufs <LUFS>` - Target LUFS level for loudness normalization
- `--fade-in <seconds>` - Fade in duration in seconds (default: 0)
- `--fade-out <seconds>` - Fade out duration in seconds (default: 0)
- `--fade-curve <curve>` - Fade curve type: `linear`, `exponential`, `logarithmic` (default: `linear`)
- `-v, --verbose` - Detailed output
- `-q, --quiet` - Error messages only
- `-h, --help` - Show help

## Notes

- Current implementation uses WAV I/O (via `hound`). If you need broad format support (FLAC/OGG/AIFF), we can integrate additional crates or an FFmpeg-based reader.
- LUFS measurement is powered by the `ebur128` crate.

## License

MIT License

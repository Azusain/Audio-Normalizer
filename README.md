# Audio Normalizer

A command-line tool for audio normalization. Supports both peak level normalization and LUFS (Loudness Units relative to Full Scale) normalization.

## Build

1. Install dependencies with vcpkg:
   ```bash
   vcpkg install libsndfile spdlog cxxopts libebur128 pkgconf
   ```

2. Build:
   ```bash
   mkdir build && cd build
   cmake .. -DCMAKE_TOOLCHAIN_FILE=/path/to/vcpkg/scripts/buildsystems/vcpkg.cmake
   cmake --build . --config Release
   ```
   
   Or run `build.bat` on Windows.

## Usage

```bash
# Peak normalization to -12 dB (default)
audio_normalizer input.wav output.wav

# Peak normalization to -6 dB
audio_normalizer -m -6 input.wav output.wav

# LUFS normalization to -23 LUFS (broadcast standard)
audio_normalizer -l -23 input.wav output.wav

# LUFS normalization to -16 LUFS (streaming standard)
audio_normalizer -l -16 input.wav output.wav

# Show peak level only
audio_normalizer --peak input.wav

# Show LUFS level only
audio_normalizer --measure-lufs input.wav

# Verbose output
audio_normalizer -v input.wav output.wav
```

## Options

- `-m, --max-peak <dB>` - Target peak level (default: -12)
- `-l, --lufs <LUFS>` - Target LUFS level for loudness normalization
- `-v, --verbose` - Detailed output
- `-q, --quiet` - Error messages only
- `--peak` - Analyze peak level without processing
- `--measure-lufs` - Analyze LUFS level without processing
- `-h, --help` - Show help

## Normalization Types

### Peak Normalization
Adjusts audio so the highest peak reaches the target dB level. This is the traditional approach but doesn't account for perceived loudness.

### LUFS Normalization
Adjusts audio based on perceptual loudness using the EBU R128 standard. This is the modern approach used by streaming platforms and broadcasters.

Common LUFS targets:
- **-23 LUFS**: EBU R128 broadcast standard
- **-16 LUFS**: Spotify, YouTube Music
- **-18 LUFS**: Apple Music
- **-14 LUFS**: Tidal, Deezer

## Supported Formats

WAV, FLAC, OGG, AU, AIFF and other libsndfile formats.

## License

MIT License

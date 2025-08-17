# Audio Normalizer

A command-line tool for audio peak normalization. Adjusts audio files to a target peak level.

## Build

1. Install dependencies with vcpkg:
   ```bash
   vcpkg install libsndfile spdlog cxxopts pkgconf
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
# Normalize to -12 dB (default)
audio_normalizer input.wav output.wav

# Normalize to -6 dB
audio_normalizer -m -6 input.wav output.wav

# Show peak level only
audio_normalizer --peak input.wav

# Verbose output
audio_normalizer -v input.wav output.wav
```

## Options

- `-m, --max-peak <dB>` - Target peak level (default: -12)
- `-v, --verbose` - Detailed output
- `-q, --quiet` - Error messages only
- `--peak` - Analyze peak level without processing
- `-h, --help` - Show help

## Supported Formats

WAV, FLAC, OGG, AU, AIFF and other libsndfile formats.

## License

MIT License

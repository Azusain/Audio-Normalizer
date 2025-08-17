#include "audio_normalizer.h"
#include <cmath>
#include <algorithm>
#include <limits>

AudioNormalizer::AudioNormalizer() {
}

AudioNormalizer::~AudioNormalizer() {
}

double AudioNormalizer::linearToDb(double linear) const {
    if (linear <= 0.0) {
        return -std::numeric_limits<double>::infinity();
    }
    return 20.0 * std::log10(linear);
}

double AudioNormalizer::dbToLinear(double db) const {
    return std::pow(10.0, db / 20.0);
}

double AudioNormalizer::findPeak(const double* data, sf_count_t frames, int channels) const {
    double peak = 0.0;
    sf_count_t totalSamples = frames * channels;
    
    for (sf_count_t i = 0; i < totalSamples; ++i) {
        double abs_sample = std::abs(data[i]);
        if (abs_sample > peak) {
            peak = abs_sample;
        }
    }
    
    return peak;
}

void AudioNormalizer::applyGain(double* data, sf_count_t frames, int channels, double gain) const {
    sf_count_t totalSamples = frames * channels;
    
    for (sf_count_t i = 0; i < totalSamples; ++i) {
        data[i] *= gain;
        // Prevent clipping
        if (data[i] > 1.0) data[i] = 1.0;
        if (data[i] < -1.0) data[i] = -1.0;
    }
}

double AudioNormalizer::getPeakLevel(const std::string& filePath) {
    SF_INFO info;
    info.format = 0;
    
    SNDFILE* file = sf_open(filePath.c_str(), SFM_READ, &info);
    if (!file) {
        SPDLOG_ERROR("Cannot open input file: {}", filePath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return -999.0;
    }
    
    const sf_count_t bufferSize = 4096;
    std::vector<double> buffer(bufferSize * info.channels);
    double globalPeak = 0.0;
    
    sf_count_t framesRead;
    while ((framesRead = sf_readf_double(file, buffer.data(), bufferSize)) > 0) {
        double bufferPeak = findPeak(buffer.data(), framesRead, info.channels);
        if (bufferPeak > globalPeak) {
            globalPeak = bufferPeak;
        }
    }
    
    sf_close(file);
    
    if (globalPeak == 0.0) {
        return -std::numeric_limits<double>::infinity();
    }
    
    return linearToDb(globalPeak);
}

double AudioNormalizer::getLufsLevel(const std::string& filePath) {
    SF_INFO info;
    info.format = 0;
    
    SNDFILE* file = sf_open(filePath.c_str(), SFM_READ, &info);
    if (!file) {
        SPDLOG_ERROR("Cannot open input file for LUFS analysis: {}", filePath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return -999.0;
    }
    
    // Create ebur128 state
    ebur128_state* state = ebur128_init(info.channels, info.samplerate, EBUR128_MODE_I);
    if (!state) {
        SPDLOG_ERROR("Cannot initialize ebur128 state");
        sf_close(file);
        return -999.0;
    }
    
    const sf_count_t bufferSize = 4096;
    std::vector<double> buffer(bufferSize * info.channels);
    
    sf_count_t framesRead;
    while ((framesRead = sf_readf_double(file, buffer.data(), bufferSize)) > 0) {
        // Add frames to ebur128 for analysis
        if (ebur128_add_frames_double(state, buffer.data(), framesRead) != EBUR128_SUCCESS) {
            SPDLOG_ERROR("Failed to add frames to ebur128 analyzer");
            ebur128_destroy(&state);
            sf_close(file);
            return -999.0;
        }
    }
    
    sf_close(file);
    
    // Get integrated loudness (LUFS)
    double loudness;
    if (ebur128_loudness_global(state, &loudness) != EBUR128_SUCCESS) {
        SPDLOG_ERROR("Failed to calculate LUFS loudness");
        ebur128_destroy(&state);
        return -999.0;
    }
    
    ebur128_destroy(&state);
    
    return loudness;
}

bool AudioNormalizer::normalizeLufs(const std::string& inputPath,
                                   const std::string& outputPath,
                                   double targetLufs) {
    SF_INFO inputInfo;
    inputInfo.format = 0;
    
    // Open input file
    SNDFILE* inputFile = sf_open(inputPath.c_str(), SFM_READ, &inputInfo);
    if (!inputFile) {
        SPDLOG_ERROR("Cannot open input file: {}", inputPath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return false;
    }
    
    SPDLOG_DEBUG("Input file info:");
    SPDLOG_DEBUG("  Sample rate: {} Hz", inputInfo.samplerate);
    SPDLOG_DEBUG("  Channels: {}", inputInfo.channels);
    SPDLOG_DEBUG("  Frames: {}", inputInfo.frames);
    SPDLOG_DEBUG("  Duration: {:.2f} seconds", (double)inputInfo.frames / inputInfo.samplerate);
    
    // Read all audio data
    std::vector<double> audioData(inputInfo.frames * inputInfo.channels);
    sf_count_t framesRead = sf_readf_double(inputFile, audioData.data(), inputInfo.frames);
    
    if (framesRead != inputInfo.frames) {
        SPDLOG_WARN("Read {} frames, expected {}", framesRead, inputInfo.frames);
    }
    
    sf_close(inputFile);
    
    // Measure current LUFS level
    double currentLufs = getLufsLevel(inputPath);
    if (currentLufs == -999.0) {
        SPDLOG_ERROR("Failed to measure current LUFS level");
        return false;
    }
    
    SPDLOG_DEBUG("Current LUFS level: {:.2f} LUFS", currentLufs);
    SPDLOG_DEBUG("Target LUFS level: {:.2f} LUFS", targetLufs);
    
    // Calculate required gain
    double gainDB = targetLufs - currentLufs;
    double gainLinear = dbToLinear(gainDB);
    
    SPDLOG_DEBUG("Required gain: {:.2f} dB ({:.3f}x)", gainDB, gainLinear);
    
    // Apply gain
    applyGain(audioData.data(), framesRead, inputInfo.channels, gainLinear);
    
    // Create output file - keep original format unless specifically converting
    SF_INFO outputInfo = inputInfo;  // Copy input file info
    
    // Only override format if we're explicitly changing extension (MP3 -> WAV)
    std::string inputPathStr = inputPath;
    std::string outputPathStr = outputPath;
    bool isMp3Input = (inputPathStr.find(".mp3") != std::string::npos || inputPathStr.find(".MP3") != std::string::npos);
    bool isWavOutput = (outputPathStr.find(".wav") != std::string::npos || outputPathStr.find(".WAV") != std::string::npos);
    
    if (isMp3Input && isWavOutput) {
        // Converting MP3 to WAV - use standard 16-bit PCM
        outputInfo.format = SF_FORMAT_WAV | SF_FORMAT_PCM_16;
        SPDLOG_DEBUG("Converting MP3 to standard 16-bit WAV");
    }
    // Otherwise keep the original format
    
    SNDFILE* outputFile = sf_open(outputPath.c_str(), SFM_WRITE, &outputInfo);
    if (!outputFile) {
        SPDLOG_ERROR("Cannot create output file: {}", outputPath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return false;
    }
    
    // Write audio data using appropriate format
    sf_count_t framesWritten;
    if ((outputInfo.format & SF_FORMAT_SUBMASK) == SF_FORMAT_PCM_16) {
        // For 24-bit output, still use double precision internally but libsndfile handles conversion
        framesWritten = sf_writef_double(outputFile, audioData.data(), framesRead);
    } else {
        framesWritten = sf_writef_double(outputFile, audioData.data(), framesRead);
    }
    if (framesWritten != framesRead) {
        SPDLOG_WARN("Wrote {} frames, expected {}", framesWritten, framesRead);
    }
    
    sf_close(outputFile);
    
    // Verify output LUFS level
    double outputLufs = getLufsLevel(outputPath);
    SPDLOG_DEBUG("Output LUFS level: {:.2f} LUFS", outputLufs);
    SPDLOG_DEBUG("LUFS normalization completed successfully!");
    
    return true;
}

bool AudioNormalizer::normalizeAudio(const std::string& inputPath, 
                                    const std::string& outputPath, 
                                    double targetPeakDB) {
    SF_INFO inputInfo;
    inputInfo.format = 0;
    
    // Open input file
    SNDFILE* inputFile = sf_open(inputPath.c_str(), SFM_READ, &inputInfo);
    if (!inputFile) {
        SPDLOG_ERROR("Cannot open input file: {}", inputPath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return false;
    }
    
    SPDLOG_DEBUG("Input file info:");
    SPDLOG_DEBUG("  Sample rate: {} Hz", inputInfo.samplerate);
    SPDLOG_DEBUG("  Channels: {}", inputInfo.channels);
    SPDLOG_DEBUG("  Frames: {}", inputInfo.frames);
    SPDLOG_DEBUG("  Duration: {:.2f} seconds", (double)inputInfo.frames / inputInfo.samplerate);
    
    // Read all audio data
    std::vector<double> audioData(inputInfo.frames * inputInfo.channels);
    sf_count_t framesRead = sf_readf_double(inputFile, audioData.data(), inputInfo.frames);
    
    if (framesRead != inputInfo.frames) {
        SPDLOG_WARN("Read {} frames, expected {}", framesRead, inputInfo.frames);
    }
    
    sf_close(inputFile);
    
    // Find current peak level
    double currentPeak = findPeak(audioData.data(), framesRead, inputInfo.channels);
    double currentPeakDB = linearToDb(currentPeak);
    
    SPDLOG_DEBUG("Current peak level: {:.2f} dB", currentPeakDB);
    SPDLOG_DEBUG("Target peak level: {:.2f} dB", targetPeakDB);
    
    // Calculate required gain
    double gainDB = targetPeakDB - currentPeakDB;
    double gainLinear = dbToLinear(gainDB);
    
    SPDLOG_DEBUG("Required gain: {:.2f} dB ({:.3f}x)", gainDB, gainLinear);
    
    // Apply gain
    applyGain(audioData.data(), framesRead, inputInfo.channels, gainLinear);
    
    // Create output file - keep original format unless specifically converting
    SF_INFO outputInfo = inputInfo;  // Copy input file info
    
    // Only override format if we're explicitly changing extension (MP3 -> WAV)
    std::string inputPathStr = inputPath;
    std::string outputPathStr = outputPath;
    bool isMp3Input = (inputPathStr.find(".mp3") != std::string::npos || inputPathStr.find(".MP3") != std::string::npos);
    bool isWavOutput = (outputPathStr.find(".wav") != std::string::npos || outputPathStr.find(".WAV") != std::string::npos);
    
    if (isMp3Input && isWavOutput) {
        // Converting MP3 to WAV - use standard 16-bit PCM
        outputInfo.format = SF_FORMAT_WAV | SF_FORMAT_PCM_16;
        SPDLOG_DEBUG("Converting MP3 to standard 16-bit WAV");
    }
    // Otherwise keep the original format
    
    SNDFILE* outputFile = sf_open(outputPath.c_str(), SFM_WRITE, &outputInfo);
    if (!outputFile) {
        SPDLOG_ERROR("Cannot create output file: {}", outputPath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return false;
    }
    
    // Write audio data using appropriate format
    sf_count_t framesWritten;
    if ((outputInfo.format & SF_FORMAT_SUBMASK) == SF_FORMAT_PCM_16) {
        // For 24-bit output, still use double precision internally but libsndfile handles conversion
        framesWritten = sf_writef_double(outputFile, audioData.data(), framesRead);
    } else {
        framesWritten = sf_writef_double(outputFile, audioData.data(), framesRead);
    }
    if (framesWritten != framesRead) {
        SPDLOG_WARN("Wrote {} frames, expected {}", framesWritten, framesRead);
    }
    
    sf_close(outputFile);
    
    // Verify output peak level
    double outputPeakDB = getPeakLevel(outputPath);
    SPDLOG_DEBUG("Output peak level: {:.2f} dB", outputPeakDB);
    SPDLOG_DEBUG("Normalization completed successfully!");
    
    return true;
}

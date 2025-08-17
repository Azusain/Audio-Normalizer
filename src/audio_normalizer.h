#pragma once

#include <string>
#include <vector>
#include <memory>
#include <sndfile.h>
#include <spdlog/spdlog.h>
#include <ebur128.h>

/**
 * Audio normalizer class
 * Implements audio peak detection and level normalization functionality
 */
class AudioNormalizer {
public:
    /**
     * Constructor
     */
    AudioNormalizer();
    
    /**
     * Destructor
     */
    ~AudioNormalizer();
    
    /**
     * Normalize audio file using peak level
     * @param inputPath Input audio file path
     * @param outputPath Output audio file path
     * @param targetPeakDB Target peak level (dB)
     * @return true if successful, false otherwise
     */
    bool normalizeAudio(const std::string& inputPath, 
                       const std::string& outputPath, 
                       double targetPeakDB);
    
    /**
     * Get peak level of audio file
     * @param filePath Audio file path
     * @return Peak level (dB), returns -999.0 on failure
     */
    double getPeakLevel(const std::string& filePath);
    
    /**
     * Get LUFS level of audio file
     * @param filePath Audio file path
     * @return LUFS level (LUFS), returns -999.0 on failure
     */
    double getLufsLevel(const std::string& filePath);
    
    /**
     * Normalize audio file using LUFS level
     * @param inputPath Input audio file path
     * @param outputPath Output audio file path
     * @param targetLufs Target LUFS level
     * @return true if successful, false otherwise
     */
    bool normalizeLufs(const std::string& inputPath,
                      const std::string& outputPath,
                      double targetLufs);
    
private:
    
    /**
     * Convert linear value to dB
     * @param linear Linear value
     * @return dB value
     */
    double linearToDb(double linear) const;
    
    /**
     * Convert dB to linear value
     * @param db dB value
     * @return Linear value
     */
    double dbToLinear(double db) const;
    
    /**
     * Find peak value in audio data
     * @param data Audio data
     * @param frames Number of frames
     * @param channels Number of channels
     * @return Peak value (linear)
     */
    double findPeak(const double* data, sf_count_t frames, int channels) const;
    
    /**
     * Apply gain to audio data
     * @param data Audio data
     * @param frames Number of frames
     * @param channels Number of channels
     * @param gain Gain value (linear)
     */
    void applyGain(double* data, sf_count_t frames, int channels, double gain) const;
};

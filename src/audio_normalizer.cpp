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
        // 防止削波
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

bool AudioNormalizer::normalizeAudio(const std::string& inputPath, 
                                    const std::string& outputPath, 
                                    double targetPeakDB) {
    SF_INFO inputInfo;
    inputInfo.format = 0;
    
    // 打开输入文件
    SNDFILE* inputFile = sf_open(inputPath.c_str(), SFM_READ, &inputInfo);
    if (!inputFile) {
        SPDLOG_ERROR("Cannot open input file: {}", inputPath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return false;
    }
    
    SPDLOG_INFO("Input file info:");
    SPDLOG_INFO("  Sample rate: {} Hz", inputInfo.samplerate);
    SPDLOG_INFO("  Channels: {}", inputInfo.channels);
    SPDLOG_INFO("  Frames: {}", inputInfo.frames);
    SPDLOG_INFO("  Duration: {:.2f} seconds", (double)inputInfo.frames / inputInfo.samplerate);
    
    // 读取所有音频数据
    std::vector<double> audioData(inputInfo.frames * inputInfo.channels);
    sf_count_t framesRead = sf_readf_double(inputFile, audioData.data(), inputInfo.frames);
    
    if (framesRead != inputInfo.frames) {
        SPDLOG_WARN("Read {} frames, expected {}", framesRead, inputInfo.frames);
    }
    
    sf_close(inputFile);
    
    // 查找当前峰值
    double currentPeak = findPeak(audioData.data(), framesRead, inputInfo.channels);
    double currentPeakDB = linearToDb(currentPeak);
    
    SPDLOG_INFO("Current peak level: {:.2f} dB", currentPeakDB);
    SPDLOG_INFO("Target peak level: {:.2f} dB", targetPeakDB);
    
    // 计算所需的增益
    double gainDB = targetPeakDB - currentPeakDB;
    double gainLinear = dbToLinear(gainDB);
    
    SPDLOG_INFO("Required gain: {:.2f} dB ({:.3f}x)", gainDB, gainLinear);
    
    // 应用增益
    applyGain(audioData.data(), framesRead, inputInfo.channels, gainLinear);
    
    // 创建输出文件
    SF_INFO outputInfo = inputInfo;  // 复制输入文件信息
    SNDFILE* outputFile = sf_open(outputPath.c_str(), SFM_WRITE, &outputInfo);
    if (!outputFile) {
        SPDLOG_ERROR("Cannot create output file: {}", outputPath);
        SPDLOG_ERROR("libsndfile error: {}", sf_strerror(nullptr));
        return false;
    }
    
    // 写入音频数据
    sf_count_t framesWritten = sf_writef_double(outputFile, audioData.data(), framesRead);
    if (framesWritten != framesRead) {
        SPDLOG_WARN("Wrote {} frames, expected {}", framesWritten, framesRead);
    }
    
    sf_close(outputFile);
    
    // 验证输出文件的峰值
    double outputPeakDB = getPeakLevel(outputPath);
    SPDLOG_INFO("Output peak level: {:.2f} dB", outputPeakDB);
    SPDLOG_INFO("Normalization completed successfully!");
    
    return true;
}

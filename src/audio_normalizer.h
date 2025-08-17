#pragma once

#include <string>
#include <vector>
#include <memory>
#include <sndfile.h>
#include <spdlog/spdlog.h>

/**
 * 音频标准化器类
 * 实现音频峰值检测和电平标准化功能
 */
class AudioNormalizer {
public:
    /**
     * 构造函数
     */
    AudioNormalizer();
    
    /**
     * 析构函数
     */
    ~AudioNormalizer();
    
    /**
     * 标准化音频文件
     * @param inputPath 输入音频文件路径
     * @param outputPath 输出音频文件路径
     * @param targetPeakDB 目标峰值电平 (dB)
     * @return true if successful, false otherwise
     */
    bool normalizeAudio(const std::string& inputPath, 
                       const std::string& outputPath, 
                       double targetPeakDB);
    
    /**
     * 获取音频文件的峰值电平
     * @param filePath 音频文件路径
     * @return 峰值电平 (dB)，失败时返回 -999.0
     */
    double getPeakLevel(const std::string& filePath);
    
private:
    
    /**
     * 将线性值转换为dB
     * @param linear 线性值
     * @return dB值
     */
    double linearToDb(double linear) const;
    
    /**
     * 将dB转换为线性值
     * @param db dB值
     * @return 线性值
     */
    double dbToLinear(double db) const;
    
    /**
     * 查找音频数据的峰值
     * @param data 音频数据
     * @param frames 帧数
     * @param channels 声道数
     * @return 峰值 (线性值)
     */
    double findPeak(const double* data, sf_count_t frames, int channels) const;
    
    /**
     * 应用增益到音频数据
     * @param data 音频数据
     * @param frames 帧数
     * @param channels 声道数
     * @param gain 增益 (线性值)
     */
    void applyGain(double* data, sf_count_t frames, int channels, double gain) const;
};

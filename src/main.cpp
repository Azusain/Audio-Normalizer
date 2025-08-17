#include "audio_normalizer.h"
#include <cxxopts.hpp>
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <iostream>

void initializeLogging(spdlog::level::level_enum level = spdlog::level::info) {
    // 设置日志格式和级别
    spdlog::set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%^%l%$] %v");
    spdlog::set_level(level);
    
    // 创建带颜色的控制台输出
    auto console = spdlog::stdout_color_mt("console");
    spdlog::set_default_logger(console);
}

int main(int argc, char* argv[]) {
    try {
        // 初始化命令行解析器
        cxxopts::Options options("audio_normalizer", "Audio Normalizer - Peak Level Control Tool");
        
        options.add_options()
            ("m,max-peak", "Target peak level in dB (e.g., -12)", 
             cxxopts::value<double>()->default_value("-12.0"))
            ("v,verbose", "Enable verbose output (debug level logging)")
            ("q,quiet", "Enable quiet mode (error level logging only)")
            ("peak", "Only show peak level of input file (no normalization)")
            ("h,help", "Show this help message")
            ("input", "Input audio file", cxxopts::value<std::string>())
            ("output", "Output audio file", cxxopts::value<std::string>())
            ;
        
        // 位置参数
        options.parse_positional({"input", "output"});
        options.positional_help("input_file [output_file]");
        
        // 解析命令行参数
        auto result = options.parse(argc, argv);
        
        // 处理帮助信息
        if (result.count("help") || argc == 1) {
            std::cout << options.help() << std::endl;
            std::cout << std::endl;
            std::cout << "Examples:" << std::endl;
            std::cout << "  audio_normalizer -m -12 input.wav output.wav" << std::endl;
            std::cout << "  audio_normalizer -m -6 -v input.flac output.flac" << std::endl;
            std::cout << "  audio_normalizer --peak input.mp3" << std::endl;
            std::cout << std::endl;
            std::cout << "Supported formats: WAV, FLAC, OGG, AU, AIFF, and others supported by libsndfile" << std::endl;
            return result.count("help") ? 0 : 1;
        }
        
        // 初始化日志系统
        spdlog::level::level_enum log_level = spdlog::level::info;
        if (result.count("verbose")) {
            log_level = spdlog::level::debug;
        } else if (result.count("quiet")) {
            log_level = spdlog::level::err;
        }
        initializeLogging(log_level);
        
        // 检查输入文件
        if (!result.count("input")) {
            SPDLOG_ERROR("Input file is required");
            std::cout << options.help() << std::endl;
            return 1;
        }
        
        std::string inputFile = result["input"].as<std::string>();
        double targetPeakDB = result["max-peak"].as<double>();
        bool peakOnly = result.count("peak") > 0;
        
        SPDLOG_INFO("Audio Normalizer v1.0.0");
        SPDLOG_DEBUG("Input file: {}", inputFile);
        SPDLOG_DEBUG("Target peak level: {:.2f} dB", targetPeakDB);
        SPDLOG_DEBUG("Peak only mode: {}", peakOnly);
        
        AudioNormalizer normalizer;
        
        // 如果只是查看峰值
        if (peakOnly) {
            SPDLOG_INFO("Analyzing peak level of: {}", inputFile);
            
            double peakDB = normalizer.getPeakLevel(inputFile);
            if (peakDB == -999.0) {
                SPDLOG_ERROR("Cannot analyze file: {}", inputFile);
                return 1;
            }
            
            // 输出结果（不使用日志格式）
            std::cout << "Peak level: " << peakDB << " dB" << std::endl;
            return 0;
        }
        
        // 标准化处理模式
        if (!result.count("output")) {
            SPDLOG_ERROR("Output file is required for normalization");
            std::cout << options.help() << std::endl;
            return 1;
        }
        
        std::string outputFile = result["output"].as<std::string>();
        SPDLOG_DEBUG("Output file: {}", outputFile);
        
        SPDLOG_INFO("Starting audio normalization...");
        SPDLOG_INFO("Input: {} -> Output: {}", inputFile, outputFile);
        SPDLOG_INFO("Target peak level: {:.2f} dB", targetPeakDB);
        
        bool success = normalizer.normalizeAudio(inputFile, outputFile, targetPeakDB);
        
        if (!success) {
            SPDLOG_ERROR("Normalization failed");
            return 1;
        }
        
        // 简洁的成功消息（非 debug 模式）
        if (log_level > spdlog::level::debug) {
            std::cout << "Normalization completed: " << inputFile << " -> " << outputFile 
                      << " (target: " << targetPeakDB << " dB)" << std::endl;
        }
        
        return 0;
        
    } catch (const cxxopts::exceptions::exception& e) {
        SPDLOG_ERROR("Command line parsing error: {}", e.what());
        return 1;
    } catch (const std::exception& e) {
        SPDLOG_ERROR("Unexpected error: {}", e.what());
        return 1;
    }
}

#include "audio_normalizer.h"
#include <cxxopts.hpp>
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <iostream>

void configureDetailedLogging(spdlog::level::level_enum level) {
    // Set detailed log format for normal operation
    spdlog::set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%^%l%$] %v");
    spdlog::set_level(level);
}


int main(int argc, char* argv[]) {
    try {
        // Initialize simple logger for help output (no timestamps)
        auto console = spdlog::stdout_color_mt("console");
        spdlog::set_default_logger(console);
        spdlog::set_pattern("%v");
        
        // Initialize command line parser
        cxxopts::Options options("audio_normalizer", "Audio Normalizer - Peak Level Control Tool");
        
        options.add_options()
            ("m,max-peak", "Target peak level in dB (e.g., -12)", 
             cxxopts::value<double>()->default_value("-12.0"))
            ("l,lufs", "Target LUFS level for loudness normalization (e.g., -23)", 
             cxxopts::value<double>())
            ("v,verbose", "Enable verbose output (debug level logging)")
            ("q,quiet", "Enable quiet mode (error level logging only)")
            ("peak", "Only show peak level of input file (no normalization)")
            ("measure-lufs", "Only show LUFS level of input file (no normalization)")
            ("h,help", "Show this help message")
            ("input", "Input audio file", cxxopts::value<std::string>())
            ("output", "Output audio file", cxxopts::value<std::string>())
            ;
        
        // Positional arguments
        options.parse_positional({"input", "output"});
        options.positional_help("input_file [output_file]");
        
        // Parse command line arguments
        auto result = options.parse(argc, argv);
        
        // Handle help information
        if (result.count("help") || argc == 1) {
            SPDLOG_INFO(options.help() + "\n");
            SPDLOG_INFO("Examples:");
            SPDLOG_INFO("  audio_normalizer -m -12 input.wav output.wav");
            SPDLOG_INFO("  audio_normalizer -l -23 input.wav output.wav");
            SPDLOG_INFO("  audio_normalizer -m -6 -v input.flac output.flac");
            SPDLOG_INFO("  audio_normalizer --peak input.mp3");
            SPDLOG_INFO("  audio_normalizer --measure-lufs input.wav");
            SPDLOG_INFO("");
            SPDLOG_INFO("Supported formats: WAV, FLAC, OGG, AU, AIFF, and others supported by libsndfile");
            return result.count("help") ? 0 : 1;
        }
        
        // Configure detailed logging for normal operation
        spdlog::level::level_enum log_level = spdlog::level::info;
        if (result.count("verbose")) {
            log_level = spdlog::level::debug;
        } else if (result.count("quiet")) {
            log_level = spdlog::level::err;
        }
        configureDetailedLogging(log_level);
        
        // Check input file
        if (!result.count("input")) {
            SPDLOG_ERROR("Input file is required");
            // Temporarily switch back to plain format for help
            spdlog::set_pattern("%v");
            SPDLOG_INFO(options.help());
            return 1;
        }
        
        std::string inputFile = result["input"].as<std::string>();
        double targetPeakDB = result["max-peak"].as<double>();
        bool peakOnly = result.count("peak") > 0;
        bool measureLufsOnly = result.count("measure-lufs") > 0;
        bool useLufs = result.count("lufs") > 0;
        double targetLufs = useLufs ? result["lufs"].as<double>() : -23.0;
        
        SPDLOG_DEBUG("Audio Normalizer v1.0.0");
        SPDLOG_DEBUG("Input file: {}", inputFile);
        SPDLOG_DEBUG("Target peak level: {:.2f} dB", targetPeakDB);
        SPDLOG_DEBUG("Target LUFS level: {:.2f} LUFS", targetLufs);
        SPDLOG_DEBUG("Peak only mode: {}", peakOnly);
        SPDLOG_DEBUG("LUFS measurement mode: {}", measureLufsOnly);
        SPDLOG_DEBUG("Use LUFS normalization: {}", useLufs);
        
        AudioNormalizer normalizer;
        
        // If only checking peak level
        if (peakOnly) {
            SPDLOG_INFO("Analyzing peak level of: {}", inputFile);
            
            double peakDB = normalizer.getPeakLevel(inputFile);
            if (peakDB == -999.0) {
                SPDLOG_ERROR("Cannot analyze file: {}", inputFile);
                return 1;
            }
            
            // Output result
            SPDLOG_INFO("Peak level: {:.2f} dB", peakDB);
            return 0;
        }
        
        // If only checking LUFS level
        if (measureLufsOnly) {
            SPDLOG_INFO("Analyzing LUFS level of: {}", inputFile);
            
            double lufsLevel = normalizer.getLufsLevel(inputFile);
            if (lufsLevel == -999.0) {
                SPDLOG_ERROR("Cannot analyze LUFS level of file: {}", inputFile);
                return 1;
            }
            
            // Output result
            SPDLOG_INFO("LUFS level: {:.2f} LUFS", lufsLevel);
            return 0;
        }
        
        // Normalization processing mode
        if (!result.count("output")) {
            SPDLOG_ERROR("Output file is required for normalization");
            // Temporarily switch back to plain format for help
            spdlog::set_pattern("%v");
            SPDLOG_INFO(options.help());
            return 1;
        }
        
        std::string outputFile = result["output"].as<std::string>();
        SPDLOG_DEBUG("Output file: {}", outputFile);
        
        SPDLOG_DEBUG("Starting audio normalization...");
        SPDLOG_DEBUG("Input: {} -> Output: {}", inputFile, outputFile);
        
        bool success;
        if (useLufs) {
            SPDLOG_DEBUG("Target LUFS level: {:.2f} LUFS", targetLufs);
            success = normalizer.normalizeLufs(inputFile, outputFile, targetLufs);
        } else {
            SPDLOG_DEBUG("Target peak level: {:.2f} dB", targetPeakDB);
            success = normalizer.normalizeAudio(inputFile, outputFile, targetPeakDB);
        }
        
        if (!success) {
            SPDLOG_ERROR("Normalization failed");
            return 1;
        }
        
        // Success message for non-debug mode
        if (log_level > spdlog::level::debug) {
            if (useLufs) {
                SPDLOG_INFO("LUFS normalization completed: {} -> {} (target: {:.2f} LUFS)", 
                           inputFile, outputFile, targetLufs);
            } else {
                SPDLOG_INFO("Peak normalization completed: {} -> {} (target: {:.2f} dB)", 
                           inputFile, outputFile, targetPeakDB);
            }
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

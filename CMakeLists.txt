cmake_minimum_required(VERSION 3.20)
project(audio_normalizer VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 使用vcpkg工具链
if(DEFINED CMAKE_TOOLCHAIN_FILE)
    message(STATUS "Using vcpkg toolchain: ${CMAKE_TOOLCHAIN_FILE}")
endif()

# 查找所需的包
# 尝试使用CMake targets
find_package(SndFile CONFIG)
if(NOT SndFile_FOUND)
    # 如果没有CMake targets，尝试pkg-config
    find_package(PkgConfig)
    if(PkgConfig_FOUND)
        pkg_check_modules(SNDFILE REQUIRED sndfile)
    else()
        message(FATAL_ERROR "libsndfile not found. Install with: vcpkg install libsndfile")
    endif()
endif()

find_package(spdlog CONFIG REQUIRED)
find_package(cxxopts CONFIG REQUIRED)

# 创建可执行文件
add_executable(audio_normalizer
    src/main.cpp
    src/audio_normalizer.cpp
    src/audio_normalizer.h
)

# 链接库
if(SndFile_FOUND)
    target_link_libraries(audio_normalizer PRIVATE 
        SndFile::sndfile
        spdlog::spdlog
        cxxopts::cxxopts
    )
else()
    target_link_libraries(audio_normalizer PRIVATE 
        ${SNDFILE_LIBRARIES}
        spdlog::spdlog
        cxxopts::cxxopts
    )
    target_include_directories(audio_normalizer PRIVATE 
        ${SNDFILE_INCLUDE_DIRS}
    )
endif()

# 设置包含目录
target_include_directories(audio_normalizer PRIVATE src)

# 编译选项
if(MSVC)
    target_compile_options(audio_normalizer PRIVATE /W4)
else()
    target_compile_options(audio_normalizer PRIVATE -Wall -Wextra -Wpedantic)
endif()

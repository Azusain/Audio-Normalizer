cmake_minimum_required(VERSION 3.20)
project(audio_normalizer VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 使用vcpkg工具链
if(DEFINED CMAKE_TOOLCHAIN_FILE)
    message(STATUS "Using vcpkg toolchain: ${CMAKE_TOOLCHAIN_FILE}")
endif()

# 查找所需的包
find_package(PkgConfig REQUIRED)
find_package(SndFile REQUIRED)
find_package(spdlog CONFIG REQUIRED)
find_package(cxxopts CONFIG REQUIRED)

# 创建可执行文件
add_executable(audio_normalizer
    src/main.cpp
    src/audio_normalizer.cpp
    src/audio_normalizer.h
)

# 链接库
target_link_libraries(audio_normalizer PRIVATE 
    SndFile::sndfile
    spdlog::spdlog
    cxxopts::cxxopts
)

# 设置包含目录
target_include_directories(audio_normalizer PRIVATE src)

# 编译选项
if(MSVC)
    target_compile_options(audio_normalizer PRIVATE /W4)
else()
    target_compile_options(audio_normalizer PRIVATE -Wall -Wextra -Wpedantic)
endif()

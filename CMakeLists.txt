cmake_minimum_required(VERSION 3.20)
project(audio_normalizer VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(DEFINED CMAKE_TOOLCHAIN_FILE)
    message(STATUS "Using vcpkg toolchain: ${CMAKE_TOOLCHAIN_FILE}")
endif()

find_package(SndFile CONFIG)
if(NOT SndFile_FOUND)
    find_package(PkgConfig)
    if(PkgConfig_FOUND)
        pkg_check_modules(SNDFILE REQUIRED sndfile)
    else()
        message(FATAL_ERROR "libsndfile not found. Install with: vcpkg install libsndfile")
    endif()
endif()

find_package(spdlog CONFIG REQUIRED)
find_package(cxxopts CONFIG REQUIRED)

# Try to find libebur128
find_package(PkgConfig QUIET)
if(PkgConfig_FOUND)
    pkg_check_modules(EBUR128 QUIET libebur128)
endif()

# If pkg-config didn't work, try to find manually
if(NOT EBUR128_FOUND)
    find_path(EBUR128_INCLUDE_DIR ebur128.h)
    find_library(EBUR128_LIBRARY NAMES ebur128)
    if(EBUR128_INCLUDE_DIR AND EBUR128_LIBRARY)
        set(EBUR128_FOUND TRUE)
        set(EBUR128_LIBRARIES ${EBUR128_LIBRARY})
        set(EBUR128_INCLUDE_DIRS ${EBUR128_INCLUDE_DIR})
    endif()
endif()

if(NOT EBUR128_FOUND)
    message(FATAL_ERROR "libebur128 not found. Install with: vcpkg install libebur128")
endif()

add_executable(audio_normalizer
    src/main.cpp
    src/audio_normalizer.cpp
    src/audio_normalizer.h
)

# Link libraries
if(SndFile_FOUND)
    target_link_libraries(audio_normalizer PRIVATE 
        SndFile::sndfile
        spdlog::spdlog
        cxxopts::cxxopts
        ${EBUR128_LIBRARIES}
    )
    target_include_directories(audio_normalizer PRIVATE 
        ${EBUR128_INCLUDE_DIRS}
    )
else()
    target_link_libraries(audio_normalizer PRIVATE 
        ${SNDFILE_LIBRARIES}
        spdlog::spdlog
        cxxopts::cxxopts
        ${EBUR128_LIBRARIES}
    )
    target_include_directories(audio_normalizer PRIVATE 
        ${SNDFILE_INCLUDE_DIRS}
        ${EBUR128_INCLUDE_DIRS}
    )
endif()

target_include_directories(audio_normalizer PRIVATE src)

if(MSVC)
    target_compile_options(audio_normalizer PRIVATE /W4)
else()
    target_compile_options(audio_normalizer PRIVATE -Wall -Wextra -Wpedantic)
endif()

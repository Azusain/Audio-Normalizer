@echo off
if not exist build mkdir build
cd build

REM Find vcpkg
set TOOLCHAIN=
if defined VCPKG_ROOT set TOOLCHAIN=%VCPKG_ROOT%\scripts\buildsystems\vcpkg.cmake
if not defined TOOLCHAIN if exist C:\vcpkg\scripts\buildsystems\vcpkg.cmake set TOOLCHAIN=C:\vcpkg\scripts\buildsystems\vcpkg.cmake

if not defined TOOLCHAIN (
    echo Error: vcpkg not found
    echo Install vcpkg and run: vcpkg install libsndfile spdlog cxxopts
    exit /b 1
)

echo Configuring...
cmake .. -DCMAKE_TOOLCHAIN_FILE="%TOOLCHAIN%"
if errorlevel 1 exit /b 1

echo Building...
cmake --build . --config Release
if errorlevel 1 exit /b 1

echo Done!

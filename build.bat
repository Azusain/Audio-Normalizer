@echo off
echo Building Audio Normalizer...

REM 创建构建目录
if not exist build mkdir build
cd build

REM 检查是否设置了 VCPKG_ROOT 环境变量
if "%VCPKG_ROOT%"=="" (
    echo Warning: VCPKG_ROOT environment variable is not set.
    echo Please set it to your vcpkg installation directory.
    echo Example: set VCPKG_ROOT=C:\vcpkg
    echo.
    echo Trying with common vcpkg locations...
    
    REM 尝试常见的 vcpkg 位置
    if exist "C:\vcpkg\scripts\buildsystems\vcpkg.cmake" (
        set "VCPKG_TOOLCHAIN=C:\vcpkg\scripts\buildsystems\vcpkg.cmake"
        echo Found vcpkg at C:\vcpkg
    ) else if exist "C:\tools\vcpkg\scripts\buildsystems\vcpkg.cmake" (
        set "VCPKG_TOOLCHAIN=C:\tools\vcpkg\scripts\buildsystems\vcpkg.cmake"
        echo Found vcpkg at C:\tools\vcpkg
    ) else (
        echo Error: Could not find vcpkg installation.
        echo Please install vcpkg and set the VCPKG_ROOT environment variable.
        pause
        exit /b 1
    )
) else (
    set "VCPKG_TOOLCHAIN=%VCPKG_ROOT%\scripts\buildsystems\vcpkg.cmake"
    echo Using vcpkg from: %VCPKG_ROOT%
)

REM 配置项目
echo Configuring project...
cmake .. -DCMAKE_TOOLCHAIN_FILE="%VCPKG_TOOLCHAIN%" -DCMAKE_BUILD_TYPE=Release

if %ERRORLEVEL% neq 0 (
    echo Error: CMake configuration failed.
    echo Make sure you have:
    echo 1. CMake installed and in PATH
    echo 2. vcpkg installed with libsndfile package
    echo 3. A C++ compiler (Visual Studio or similar)
    echo.
echo To install required dependencies with vcpkg:
echo   vcpkg install libsndfile spdlog cxxopts
    pause
    exit /b 1
)

REM 构建项目
echo Building project...
cmake --build . --config Release

if %ERRORLEVEL% neq 0 (
    echo Error: Build failed.
    pause
    exit /b 1
)

echo.
echo Build completed successfully!
echo Executable location: build\Release\audio_normalizer.exe (or build\audio_normalizer.exe)
echo.
echo To test the program, try:
echo   audio_normalizer.exe --help
echo.
pause

@echo off
if not exist build mkdir build
cd build

REM Try common vcpkg locations
if exist "C:\vcpkg\scripts\buildsystems\vcpkg.cmake" (
    set "VCPKG_TOOLCHAIN=C:\vcpkg\scripts\buildsystems\vcpkg.cmake"
) else if exist "C:\tools\vcpkg\scripts\buildsystems\vcpkg.cmake" (
    set "VCPKG_TOOLCHAIN=C:\tools\vcpkg\scripts\buildsystems\vcpkg.cmake"
) else if defined VCPKG_ROOT (
    set "VCPKG_TOOLCHAIN=%VCPKG_ROOT%\scripts\buildsystems\vcpkg.cmake"
) else (
    echo Error: vcpkg not found. Install vcpkg and run:
    echo   vcpkg install libsndfile spdlog cxxopts
    pause
    exit /b 1
)

cmake .. -DCMAKE_TOOLCHAIN_FILE="%VCPKG_TOOLCHAIN%" -DCMAKE_BUILD_TYPE=Release
if %ERRORLEVEL% neq 0 (
    echo Build failed. Make sure dependencies are installed:
    echo   vcpkg install libsndfile spdlog cxxopts
    pause
    exit /b 1
)

cmake --build . --config Release
if %ERRORLEVEL% neq 0 (
    pause
    exit /b 1
)

echo Build complete!

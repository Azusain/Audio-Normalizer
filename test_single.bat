@echo off
setlocal enabledelayedexpansion

echo Testing single file normalization...
echo.

set "NORMALIZER=C:\Users\azusaing\Desktop\Code\Audio-Normalizer\audio_normalizer.exe"
set "INPUT=C:\Users\azusaing\Desktop\Code\Audio-Normalizer\test_audio\test_song.mp3"
set "OUTPUT=C:\Users\azusaing\Desktop\Code\Audio-Normalizer\test_audio\normalized_test_song.wav"

echo Input: %INPUT%
echo Output: %OUTPUT%
echo.

echo Running normalization...
"%NORMALIZER%" "%INPUT%" "%OUTPUT%" --max-peak=-12.0

if %errorlevel% equ 0 (
    if exist "%OUTPUT%" (
        echo ✓ SUCCESS: File normalized successfully!
        echo.
        dir "%OUTPUT%"
    ) else (
        echo ✗ ERROR: Output file not created
    )
) else (
    echo ✗ ERROR: Normalization failed with error code: %errorlevel%
)

echo.
echo Press any key to continue...
pause >nul

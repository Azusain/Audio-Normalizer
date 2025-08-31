@echo off
setlocal enabledelayedexpansion

echo ==========================================
echo Safe Audio Normalizer Batch Script
echo ==========================================
echo.

set "AUDIO_DIR=C:\Users\azusaing\Desktop\Code\Dulcets\frontend\public\audio"
set "NORMALIZER=C:\Users\azusaing\Desktop\Code\Audio-Normalizer\audio_normalizer.exe"
set "TARGET_PEAK=-12.0"

echo Audio directory: %AUDIO_DIR%
echo Target peak level: %TARGET_PEAK% dB
echo.

set /a processed=0
set /a success=0
set /a failed=0

echo Starting normalization...
echo.

rem Process all MP3, FLAC, and WAV files recursively
for /r "%AUDIO_DIR%" %%F in (*.mp3 *.flac *.wav) do (
    set "input_file=%%F"
    set "filename=%%~nF"
    set "extension=%%~xF"
    
    rem Skip files that are already normalized
    echo !filename! | findstr /I "normalized" >nul
    if !errorlevel! neq 0 (
        echo Processing: !filename!!extension!
        
        rem Create temporary output file
        set "temp_output=%%~dpFtemp_normalized_!filename!.wav"
        
        rem Run normalization
        "%NORMALIZER%" "!input_file!" "!temp_output!" --max-peak=%TARGET_PEAK% --quiet
        
        if !errorlevel! equ 0 (
            if exist "!temp_output!" (
                rem Successfully normalized, now replace original
                del "!input_file!" >nul 2>&1
                if !errorlevel! equ 0 (
                    ren "!temp_output!" "normalized_!filename!!extension!"
                    if !errorlevel! equ 0 (
                        echo ✓ Successfully normalized: !filename!!extension!
                        set /a success+=1
                    ) else (
                        echo ✗ Failed to rename: !filename!!extension!
                        set /a failed+=1
                        rem Try to restore temp file
                        if exist "!temp_output!" (
                            ren "!temp_output!" "failed_!filename!!extension!"
                        )
                    )
                ) else (
                    echo ✗ Failed to delete original: !filename!!extension!
                    set /a failed+=1
                    rem Clean up temp file
                    if exist "!temp_output!" del "!temp_output!" >nul 2>&1
                )
            ) else (
                echo ✗ No output generated for: !filename!!extension!
                set /a failed+=1
            )
        ) else (
            echo ✗ Normalization failed for: !filename!!extension!
            set /a failed+=1
            rem Clean up any temp file
            if exist "!temp_output!" del "!temp_output!" >nul 2>&1
        )
        
        set /a processed+=1
        echo.
    ) else (
        echo Skipping already normalized file: !filename!!extension!
        echo.
    )
)

echo ==========================================
echo Batch processing completed
echo ==========================================
echo Total files processed: %processed%
echo Successfully normalized: %success%
echo Failed: %failed%
echo.

if %failed% gtr 0 (
    echo Warning: Some files failed to process.
    echo Check the output above for details.
)

echo Press any key to exit...
pause >nul

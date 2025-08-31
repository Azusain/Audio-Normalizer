@echo off
setlocal enabledelayedexpansion

echo ==========================================
echo Audio Normalizer Batch Script v2.0
echo ==========================================
echo.

set "AUDIO_DIR=C:\Users\azusaing\Desktop\Code\Dulcets\frontend\public\audio"
set "NORMALIZER=C:\Users\azusaing\Desktop\Code\Audio-Normalizer\audio_normalizer.exe"
set "TARGET_PEAK=-12.0"

echo Audio directory: %AUDIO_DIR%
echo Target peak level: %TARGET_PEAK% dB
echo Output format: WAV (24-bit)
echo.

set /a processed=0
set /a success=0
set /a failed=0

echo Starting normalization...
echo.

for /r "%AUDIO_DIR%" %%F in (*.mp3 *.flac *.wav) do (
    set "input_file=%%F"
    set "filename=%%~nF"
    set "extension=%%~xF"
    
    echo !filename! | findstr /I "normalized" >nul
    if !errorlevel! neq 0 (
        echo Processing: !filename!!extension!
        
        set "temp_output=%%~dpFtemp_normalized_!filename!.wav"
        
        "%NORMALIZER%" "!input_file!" "!temp_output!" --max-peak=%TARGET_PEAK% --quiet
        
        if !errorlevel! equ 0 (
            if exist "!temp_output!" (
                del "!input_file!" >nul 2>&1
                if !errorlevel! equ 0 (
                    ren "!temp_output!" "normalized_!filename!.wav"
                    if !errorlevel! equ 0 (
                        echo SUCCESS: !filename!!extension! -> normalized_!filename!.wav
                        set /a success+=1
                    ) else (
                        echo ERROR: Failed to rename !filename!!extension!
                        set /a failed+=1
                        if exist "!temp_output!" (
                            ren "!temp_output!" "failed_!filename!.wav"
                        )
                    )
                ) else (
                    echo ERROR: Failed to delete original !filename!!extension!
                    set /a failed+=1
                    if exist "!temp_output!" del "!temp_output!" >nul 2>&1
                )
            ) else (
                echo ERROR: No output generated for !filename!!extension!
                set /a failed+=1
            )
        ) else (
            echo ERROR: Normalization failed for !filename!!extension!
            set /a failed+=1
            if exist "!temp_output!" del "!temp_output!" >nul 2>&1
        )
        
        set /a processed+=1
        echo.
    ) else (
        echo SKIP: Already normalized - !filename!!extension!
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
    echo WARNING: %failed% files failed to process.
    echo Check the output above for details.
) else (
    echo All files processed successfully.
)

echo Press any key to exit...
pause >nul

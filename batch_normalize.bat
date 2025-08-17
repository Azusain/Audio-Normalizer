@echo off
setlocal enabledelayedexpansion

REM Path to audio normalizer executable
set NORMALIZER="audio_normalizer.exe"

REM Check if normalizer exists
if not exist %NORMALIZER% (
    echo ERROR: Audio normalizer not found at %NORMALIZER%
    echo Please build the audio_normalizer project first.
    pause
    exit /b 1
)

REM Count total files for progress tracking (exclude already normalized files)
set /a total=0
for %%f in (*.mp3 *.wav *.flac *.ogg *.aiff *.au) do (
    set "filename=%%~nf"
    echo !filename! | findstr /i "^normalized_" >nul
    if !errorlevel! neq 0 (
        set /a total+=1
    )
)

if %total%==0 (
    echo No files to process.
    pause
    exit /b 0
)

REM Process each audio file (exclude already normalized files)
set /a processed=0
for %%f in (*.mp3 *.wav *.flac *.ogg *.aiff *.au) do (
    set "filename=%%~nf"
    
    REM Skip files that already start with "normalized_"
    echo !filename! | findstr /i "^normalized_" >nul
    if !errorlevel! neq 0 (
        set /a processed+=1
        
        REM Get file extension
        set "extension=%%~xf"
        
        REM Create normalized filename - convert MP3 to FLAC for smaller lossless files
    if /i "!extension!"==".mp3" (
        set "output_file=normalized_!filename!.flac"
    ) else (
        set "output_file=normalized_!filename!!extension!"
    )
        
        REM Skip if normalized version already exists
        if not exist "!output_file!" (
            REM Run normalization (MP3->FLAC for optimal size/quality balance)
            %NORMALIZER% -l -14 "%%f" "!output_file!" >nul 2>&1
            
            if !errorlevel!==0 (
                echo ✓ !output_file!
            ) else (
                echo ✗ %%f
            )
        )
    )
)

echo Done.
pause

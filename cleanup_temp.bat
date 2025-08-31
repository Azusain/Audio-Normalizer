@echo off
setlocal enabledelayedexpansion

echo ================================================
echo Cleaning up temp_normalized files
echo ================================================
echo.

set "AUDIO_DIR=C:\Users\azusaing\Desktop\Code\Dulcets\frontend\public\audio"
echo Target directory: %AUDIO_DIR%
echo.

set /a renamed=0
set /a failed=0

for /r "%AUDIO_DIR%" %%F in (temp_normalized_*.wav temp_normalized_*.mp3 temp_normalized_*.flac) do (
    set "fullpath=%%F"
    set "filename=%%~nF"
    set "filedir=%%~dpF"
    set "extension=%%~xF"
    
    echo !filename! | findstr /B "temp_normalized_" >nul
    if !errorlevel!==0 (
        set "newname=!filename:temp_normalized_=normalized_!"
        set "newpath=!filedir!!newname!!extension!"
        
        echo Renaming: !filename!!extension!
        echo      -> !newname!!extension!
        
        ren "!fullpath!" "!newname!!extension!"
        if !errorlevel!==0 (
            set /a renamed+=1
        ) else (
            echo ERROR: Failed to rename !filename!!extension!
            set /a failed+=1
        )
        echo.
    )
)

echo ================================================
echo Cleanup completed
echo ================================================
echo Files renamed: %renamed%
echo Failed: %failed%
echo.

if %failed% gtr 0 (
    echo WARNING: Some files could not be renamed
) else (
    echo All temp files cleaned up successfully
)

pause

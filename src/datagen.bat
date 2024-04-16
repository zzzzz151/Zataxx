@echo off
setlocal enabledelayedexpansion

set numThreads=%1

if not defined numThreads (
    echo Usage: src\datagen.bat ^<numThreads^>
    pause
) else (
    set command=target\release\zataxx.exe datagen
    if "%2" == "datagen_openings" (
        set command=target\release\zataxx.exe datagen_openings
    )

    for /l %%i in (1, 1, %numThreads%) do (
        timeout /nobreak /t 1 > nul
        start cmd.exe /k "!command!"
    )
)

endlocal
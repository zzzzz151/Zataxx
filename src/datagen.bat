@echo off
setlocal enabledelayedexpansion

set numThreads=%1

if not defined numThreads (
    echo Usage: src\datagen.bat ^<numThreads^>
    pause
)

for /l %%i in (1, 1, %numThreads%) do (
    timeout /nobreak /t 1 > nul
    start cmd.exe /k "target\release\zataxx.exe datagen"
)


endlocal
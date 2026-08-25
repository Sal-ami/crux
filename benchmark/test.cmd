@echo off
findstr /c:"pass" status.txt >nul 2>&1
if %errorlevel%==0 (exit /b 0) else (exit /b 1)

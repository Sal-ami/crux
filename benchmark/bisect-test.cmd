@echo off
git log -1 --format=%s HEAD | findstr /i "break" >nul
if %ERRORLEVEL%==0 (exit 1) else (exit 0)

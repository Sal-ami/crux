@echo off
if exist status.txt (
  findstr "pass" status.txt >nul 2>&1
  exit /b %errorlevel%
)
if exist vendor\lib.txt (
  findstr "v1" vendor\lib.txt >nul 2>&1
  exit /b %errorlevel%
)
exit /b 1

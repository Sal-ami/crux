@echo off
findstr "enabled" config.txt >nul 2>&1 && findstr "v1" handler.txt >nul 2>&1

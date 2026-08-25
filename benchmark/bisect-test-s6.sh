#!/bin/sh
# S6: interaction fault — config.txt must say "enabled" AND handler.txt must say "v1"
config=$(cat config.txt 2>/dev/null)
handler=$(cat handler.txt 2>/dev/null)
if [ "$config" = "enabled" ] && [ "$handler" = "v1" ]; then exit 0; else exit 1; fi

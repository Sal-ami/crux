#!/bin/sh
# S7: dependency blame - vendor/lib.txt must say "v1"
content=$(cat vendor/lib.txt 2>/dev/null)
case "$content" in
  *v1*) exit 0 ;;
  *)    exit 1 ;;
esac

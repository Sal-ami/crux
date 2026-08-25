#!/bin/sh
# Exit 0 = good (behavior intact), exit 1 = bad (behavior broken)
# S1-S5: status.txt must contain "pass"
# S7: vendor/lib.txt must contain "v1"
if [ -f status.txt ]; then
  content=$(cat status.txt 2>/dev/null)
  case "$content" in
    *pass*) exit 0 ;;
    *)      exit 1 ;;
  esac
elif [ -f vendor/lib.txt ]; then
  content=$(cat vendor/lib.txt 2>/dev/null)
  case "$content" in
    *v1*) exit 0 ;;
    *)    exit 1 ;;
  esac
else
  exit 1
fi

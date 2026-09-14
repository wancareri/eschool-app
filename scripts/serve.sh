#!/bin/bash
# Quick IPA server — serves the latest build from build/day/dist/ or ~/Desktop
set -e

DIR="${1:-build/day/dist}"
PORT="${2:-8080}"

if [ ! -d "$DIR" ]; then
  echo "Directory not found: $DIR"
  exit 1
fi

IPA=$(ls -t "$DIR"/*.ipa 2>/dev/null | head -1)
if [ -z "$IPA" ]; then
  echo "No .ipa files in $DIR"
  exit 1
fi

# Get local IP
IP=$(ip -4 addr show 2>/dev/null | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep -v '127.0.0.1' | head -1)
if [ -z "$IP" ]; then
  IP=$(ifconfig 2>/dev/null | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep -v '127.0.0.1' | head -1)
fi

IPA_NAME=$(basename "$IPA")
echo ""
echo "  IPA: $IPA_NAME"
echo "  URL: http://$IP:$PORT/$IPA_NAME"
echo ""
echo "  На iPhone: Safari → вставь ссылку → Share → LiveContainer"
echo "  Ctrl+C для остановки"
echo ""

cd "$(dirname "$IPA")"
python3 -m http.server "$PORT" --bind 0.0.0.0

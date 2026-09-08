#!/usr/bin/env bash
set -euo pipefail

BINWALK="./target/release/binwalk_scan"

if [ $# -ne 1 ]; then
    echo "사용법: $0 <firmware.bin>"
    exit 1
fi

"$BINWALK" "$1"

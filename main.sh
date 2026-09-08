#!/usr/bin/env bash
set -euo pipefail

BINWALK="./target/release/binwalk_scan"

if [ ! -x "$BINWALK" ]; then
    echo "[오류] binwalk 바이너리가 없습니다. 'cargo build --release' 먼저 실행하세요."
    exit 1
fi

if [ $# -ne 1 ]; then
    echo "사용법: $0 <firmware.bin>"
    exit 1
fi

"$BINWALK" "$1"

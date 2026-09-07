#!/usr/bin/env bash
#
# main.sh - IoT Firmware Analysis Tool
#
# 설명:
#   사용자가 펌웨어 파일 하나만 입력하면 다음을 자동으로 수행합니다:
#   1) -b, -M, -e 기능을 수행하는 커스텀 Rust 도구 (fw_extractor) 실행
#   2) 추출된 루트 파일시스템(rootfs) 자동 탐지
#   3) firmwalk.sh 를 이용한 취약점·민감 정보 분석
#   4) Markdown 형식의 분석 리포트 생성
#
# 저장소:
#   https://github.com/K-Shield17/IoT_fw_tool/
#
# 사용법:
#   ./main.sh <firmware_file>
#
# 예:
#   ./main.sh ../samples/router_firmware.bin
#

# =============================================================================
# Bash 에러 처리 옵션
# =============================================================================
# -E: ERR trap 이 서브쉘·함수·명령 치환에도 상속되도록 설정
# -e: 명령이 실패하면 스크립트를 즉시 종료
# -u: 정의되지 않은 변수를 사용하면 에러 발생
# -o pipefail: 파이프라인 내의 어떤 명령이라도 실패하면 전체를 실패로 간주
set -Eeuo pipefail

# =============================================================================
# 프로젝트 루트 및 기본 경로 설정
# =============================================================================

# main.sh 이 위치한 디렉토리를 프로젝트 루트로 간주
# 이렇게 하면 사용자가 어느 디렉토리에서 실행해도 경로 문제가 발생하지 않음
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# =============================================================================
# 설정: 실행 파일 및 스크립트 경로
# =============================================================================

# -----------------------------------------------------------------------------
# fw_extractor: binwalk 의 -b, -M, -e 기능만 구현한 커스텀 Rust 도구
# -----------------------------------------------------------------------------
# - Cargo 프로젝트 (fw_extractor/) 를 cargo build --release 로 빌드하면
#   target/release/fw_extractor 실행 파일이 생성됨
# - 이 실행 파일은 펌웨어 파일을 받아 내부적으로 다음을 수행:
#   * -b: 스캔 시작 오프셋 처리
#   * -M: 재귀적 처리 (중첩된 아카이브·파일시스템까지 분석)
#   * -e: 식별된 파일·아카이브·파일시스템 추출
# - 실제 인자 설계는 Rust 코드와 맞춰야 함 (예: <firmware> <output_dir>)
EXTRACTOR_BIN="${PROJECT_ROOT}/fw_extractor/target/release/fw_extractor"

# -----------------------------------------------------------------------------
# firmwalk.sh: firmwalk 에서 가져온 쉘스크립트 하나만
# -----------------------------------------------------------------------------
# - 추출된 루트 파일시스템(rootfs) 을 인자로 받아
#   계정·비밀번호·SSH 키·설정 파일·백도어 등을 탐색한다고 가정
# - 실제 firmwalk.sh 의 인자·출력 형식에 맞춰 호출 및 리포트 포맷을 조정 가능
FIRMWALK_SCRIPT="${PROJECT_ROOT}/firmwalk.sh"

# =============================================================================
# 입력 검증
# =============================================================================

# 첫 번째 명령행 인자를 펌웨어 파일 경로로 사용
FIRMWARE_FILE="${1:-}"

# 인자가 없으면 사용법 안내 후 종료
if [[ -z "${FIRMWARE_FILE}" ]]; then
    echo "Usage: $0 <firmware_file>"
    echo "Example: $0 ../samples/router_firmware.bin"
    exit 1
fi

# 펌웨어 파일이 존재하지 않으면 에러 메시지 출력 후 종료
if [[ ! -f "${FIRMWARE_FILE}" ]]; then
    echo "[ERROR] Firmware file not found: ${FIRMWARE_FILE}" >&2
    exit 1
fi

# fw_extractor 바이너리가 존재하지 않으면 빌드 안내 후 종료
if [[ ! -x "${EXTRACTOR_BIN}" ]]; then
    echo "[ERROR] Custom extractor binary not found: ${EXTRACTOR_BIN}" >&2
    echo "[INFO] Run: cd fw_extractor && cargo build --release" >&2
    exit 1
fi

# firmwalk.sh 가 존재하지 않거나 실행 권한이 없으면 에러 출력 후 종료
if [[ ! -x "${FIRMWALK_SCRIPT}" ]]; then
    echo "[ERROR] firmwalk.sh not found or not executable: ${FIRMWALK_SCRIPT}" >&2
    echo "[INFO] Run: chmod +x firmwalk.sh" >&2
    exit 1
fi

# =============================================================================
# 작업 디렉토리 및 파일명 생성
# =============================================================================

# 펌웨어 파일명에서 경로 제거 (예: ../samples/fw.bin -> fw.bin)
FIRMWARE_NAME="$(basename "${FIRMWARE_FILE}")"

# 확장자 제거 (예: fw.bin -> fw, fw.tar.gz -> fw.tar)
# 여기서는 마지막 점(.) 이후만 제거하는 간단한 방식 사용
FIRMWARE_STEM="${FIRMWARE_NAME%.*}"

# 타임스탬프 생성 (예: 20260907_163000)
# 같은 펌웨어를 여러 번 분석해도 디렉토리 충돌을 방지하기 위함
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"

# 작업용 루트 디렉토리
# output/ 안에 펌웨어 이름과 타임스탬프를 포함한 서브디렉토리를 생성
# 예: output/fw_20260907_163000
RUN_DIR="${PROJECT_ROOT}/output/${FIRMWARE_STEM}_${TIMESTAMP}"

# fw_extractor 가 파일을 추출할 디렉토리
EXTRACT_DIR="${RUN_DIR}/extracted"

# firmwalk.sh 분석 로그를 저장할 디렉토리
ANALYSIS_DIR="${RUN_DIR}/analysis"

# 보고서가 저장될 디렉토리 (프로젝트 루트의 reports/)
REPORT_DIR="${PROJECT_ROOT}/reports"

# 최종 보고서 파일 경로 (Markdown)
# 예: reports/fw_20260907_163000_report.md
REPORT_FILE="${REPORT_DIR}/${FIRMWARE_STEM}_${TIMESTAMP}_report.md"

# 필요한 디렉토리들 생성
mkdir -p "${EXTRACT_DIR}" "${ANALYSIS_DIR}" "${REPORT_DIR}"

# =============================================================================
# 로그 출력: 분석 시작
# =============================================================================

echo "[*] Target firmware      : ${FIRMWARE_FILE}"
echo "[*] Working directory    : ${RUN_DIR}"
echo "[*] Extract directory    : ${EXTRACT_DIR}"
echo "[*] Analysis directory   : ${ANALYSIS_DIR}"
echo "[*] Report file          : ${REPORT_FILE}"

# =============================================================================
# [1/4] 커스텀 Rust 추출기 실행 (-b, -M, -e 기능 내장)
# =============================================================================

echo "[1/4] Running custom extractor (-b, -M, -e)..."

# fw_extractor 호출 예:
#   fw_extractor <firmware_file> <output_dir>
#
# Rust 코드에서 다음과 같이 동작한다고 가정:
#   - 인자 1: 펌웨어 파일 경로
#   - 인자 2: 추출 결과를 저장할 디렉토리 (EXTRACT_DIR)
#   - 내부적으로 -b, -M, -e 로직을 모두 수행
#   - stdout/stderr 로 binwalk 스타일의 로그 출력
#
# 실제 Rust 실행 파일의 인자 설계에 맞춰 이 부분을 수정하세요.
"${EXTRACTOR_BIN}" "${FIRMWARE_FILE}" "${EXTRACT_DIR}" \
    > "${RUN_DIR}/extractor.log" 2>&1

# 추출기 실행 실패 시 로그를 보여주고 종료
if [[ $? -ne 0 ]]; then
    echo "[ERROR] Extractor failed. Check log: ${RUN_DIR}/extractor.log" >&2
    exit 1
fi

# =============================================================================
# [2/4] 추출된 루트 파일시스템(rootfs) 자동 탐지
# =============================================================================

echo "[2/4] Locating extracted root filesystem..."

# rootfs 후보 디렉토리를 탐색하는 두 단계 전략:
# 1) 이름 기반 탐색: squashfs-root, rootfs 등 일반적인 디렉토리 이름 검색
# 2) 구조 기반 탐색: /etc, /bin, /usr, /sbin 등 Linux 루트 파일시스템 패턴 확인

# 1) 이름으로 탐색: squashfs-root, rootfs 등
ROOTFS_DIR="$(
    find "${EXTRACT_DIR}" -type d \
        \( -name "squashfs-root" -o -name "rootfs" -o -name "*rootfs*" \) \
        2>/dev/null \
    | head -n 1 || true
)"

# 2) 이름이 아닌 Linux 루트 파일시스템 구조로 재탐색
#    - /etc, /bin, /usr, /sbin 중 일부가 존재하면 rootfs 로 간주
#    - 이렇게 하면 squashfs-root 가 아닌 다른 이름이어도 탐지 가능
if [[ -z "${ROOTFS_DIR}" ]]; then
    ROOTFS_DIR="$(
        find "${EXTRACT_DIR}" -type d 2>/dev/null \
        | while read -r dir; do
            if [[ -d "${dir}/etc" ]] && \
               { [[ -d "${dir}/bin" ]] || [[ -d "${dir}/usr" ]] || [[ -d "${dir}/sbin" ]]; }; then
                echo "${dir}"
                break
            fi
        done
    )"
fi

# 그래도 rootfs 를 찾지 못하면 에러 출력 후 종료
if [[ -z "${ROOTFS_DIR}" || ! -d "${ROOTFS_DIR}" ]]; then
    echo "[ERROR] Extracted Linux root filesystem was not found." >&2
    echo "[INFO] Check extractor output: ${RUN_DIR}/extractor.log" >&2
    exit 1
fi

echo "[*] Root filesystem: ${ROOTFS_DIR}"

# =============================================================================
# [3/4] firmwalk.sh 실행 (rootfs 기반 분석)
# =============================================================================

echo "[3/4] Running firmwalk analysis..."

# firmwalk.sh 는 rootfs 디렉토리를 인자로 받아
# 계정·비밀번호·키·설정·백도어 등을 탐색한다고 가정합니다.
#
# 실제 firmwalk.sh 의 인자 설계에 맞춰 호출을 수정하세요.
# 예: ./firmwalk.sh <rootfs_path>
#
# firmwalk 가 실패해도 분석은 계속 진행 (로그만 남김)
# 필요하면 여기서 에러 처리를 더 강화해도 됩니다.
"${FIRMWALK_SCRIPT}" "${ROOTFS_DIR}" \
    > "${ANALYSIS_DIR}/firmwalk.log" 2>&1 || true

# =============================================================================
# [4/4] Markdown 리포트 생성
# =============================================================================

echo "[4/4] Generating report..."

{
    # 보고서 제목
    echo "# IoT Firmware Analysis Report"
    echo

    # 메타 정보
    echo "- **Analysis time:** $(date '+%Y-%m-%d %H:%M:%S')"
    echo "- **Input firmware:** ${FIRMWARE_FILE}"
    echo "- **Extracted rootfs:** ${ROOTFS_DIR}"
    echo "- **Extractor options:** -b, -M, -e (custom Rust implementation)"
    echo

    # fw_extractor(추출기) 출력 섹션
    echo "## Extractor (binwalk-like) Output"
    echo
    echo '```text'
    cat "${RUN_DIR}/extractor.log"
    echo '```'
    echo

    # firmwalk 분석 결과 섹션
    echo "## Firmwalk Output"
    echo
    echo '```text'
    cat "${ANALYSIS_DIR}/firmwalk.log"
    echo '```'
    echo

    # 필요하면 추가 분석 섹션을 덧붙여도 좋습니다.
    # 예: "## Findings", "## Recommendations", "## Sensitive Files" 등

} > "${REPORT_FILE}"

# =============================================================================
# 완료 메시지
# =============================================================================

echo "[*] Analysis completed."
echo "[*] Report: ${REPORT_FILE}"

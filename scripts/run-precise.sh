#!/usr/bin/env bash
set -euo pipefail

# Claude Code 用精密プロンプトを送信するヘルパースクリプト。
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

source "${SCRIPT_DIR}/lib/run-session-common.sh"

START_TIME=$(date +%s)

rsc_parse_args "$@"

PROMPT_FILE="$(rsc_prompt_file "${ROOT_DIR}" "precise" "${RSC_PROMPT_KEY}")"
RESET_SCRIPT="$(rsc_reset_script "${SCRIPT_DIR}" "${RSC_PROMPT_KEY}")"

rsc_ensure_file_exists "${PROMPT_FILE}" "プロンプトファイルが見つかりません"
rsc_ensure_executable "${RESET_SCRIPT}" "リセットスクリプトが実行できません"

"${RESET_SCRIPT}"

rsc_exec_prompt "${RSC_MODE}" "${PROMPT_FILE}"

rsc_run_tests "${ROOT_DIR}" "${RSC_PROMPT_KEY}" || true

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
printf 'elapsed: %02d:%02d\n' $((ELAPSED/60)) $((ELAPSED%60))

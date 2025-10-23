#!/usr/bin/env bash
set -euo pipefail

# Claude Code 用あいまいプロンプトを送信するヘルパースクリプト。
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROMPT_KEY="${1:-model-a}"

case "${PROMPT_KEY}" in
  model-a|a)
    PROMPT_FILE="${ROOT_DIR}/docs/ambiguous/prompt_model_a_ambiguous.md"
    RESET_SCRIPT="${SCRIPT_DIR}/model-a-code-reset.sh"
    ;;
  model-b|b)
    PROMPT_FILE="${ROOT_DIR}/docs/ambiguous/prompt_model_b_ambiguous.md"
    RESET_SCRIPT="${SCRIPT_DIR}/model-b-code-reset.sh"
    ;;
  *)
    echo "使用方法: $(basename "$0") [model-a|model-b]" >&2
    exit 1
    ;;
esac

if [[ ! -f "${PROMPT_FILE}" ]]; then
  echo "プロンプトファイルが見つかりません: ${PROMPT_FILE}" >&2
  exit 2
fi

if [[ -n "${RESET_SCRIPT:-}" ]]; then
  if [[ ! -x "${RESET_SCRIPT}" ]]; then
    echo "リセットスクリプトが実行できません: ${RESET_SCRIPT}" >&2
    exit 3
  fi
  "${RESET_SCRIPT}"
fi

claude \
  --permission-mode acceptEdits \
  --output-format stream-json \
  --verbose \
  -p "$(cat "${PROMPT_FILE}")"

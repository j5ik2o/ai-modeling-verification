#!/usr/bin/env bash
set -euo pipefail

# Claude Code 用あいまいプロンプトを送信するヘルパースクリプト。
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

MODE="codex"
PROMPT_KEY=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode=*)
      MODE="${1#--mode=}"
      shift
      ;;
    --mode)
      if [[ $# -lt 2 ]]; then
        echo "--mode requires an argument" >&2
        exit 1
      fi
      MODE="$2"
      shift 2
      ;;
    --)
      shift
      break
      ;;
    -*)
      echo "未知のオプションです: $1" >&2
      exit 1
      ;;
    *)
      if [[ -z "${PROMPT_KEY}" ]]; then
        PROMPT_KEY="$1"
      else
        echo "引数が多すぎます: $1" >&2
        exit 1
      fi
      shift
      ;;
  esac
done

if [[ -z "${PROMPT_KEY}" ]]; then
  PROMPT_KEY="model-a"
fi

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
    echo "使用方法: $(basename "$0") [--mode codex|claude|gemini] [model-a|model-b]" >&2
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

case "${MODE}" in
  codex)
    EXEC_CMD=(codex exec --full-auto)
    ;;
  claude)
    EXEC_CMD=(claude --permission-mode acceptEdits --output-format stream-json --verbose -p)
    ;;
  gemini)
    EXEC_CMD=(gemini --yolo -p)
    ;;
  *)
    echo "未対応のモードです: ${MODE}" >&2
    exit 1
    ;;
esac

"${EXEC_CMD[@]}" "$(cat "${PROMPT_FILE}")"

if ! cargo test; then
  echo "cargo test failed (continuing)" >&2
fi

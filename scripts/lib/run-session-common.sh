#!/usr/bin/env bash

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  echo "このスクリプトは source して利用してください: ${BASH_SOURCE[0]}" >&2
  exit 1
fi

if [[ -n "${RSC_LIB_LOADED:-}" ]]; then
  return 0
fi
RSC_LIB_LOADED=1

rsc_normalize_mode() {
  local input="$1"
  local lower
  lower="$(printf '%s' "${input}" | tr '[:upper:]' '[:lower:]')"

  case "${lower}" in
    codex|c)
      printf '%s' "codex"
      ;;
    claude|anthropic|a|cl)
      printf '%s' "claude"
      ;;
    gemini|g)
      printf '%s' "gemini"
      ;;
    *)
      return 1
      ;;
  esac
}

rsc_parse_args() {
  RSC_MODE="claude"
  RSC_PROMPT_KEY=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --mode=*)
        local value="${1#--mode=}"
        if ! RSC_MODE="$(rsc_normalize_mode "${value}")"; then
          echo "未対応のモードです: ${value}" >&2
          return 1
        fi
        shift
        ;;
      --mode)
        if [[ $# -lt 2 ]]; then
          echo "--mode オプションには引数が必要です" >&2
          return 1
        fi
        if ! RSC_MODE="$(rsc_normalize_mode "$2")"; then
          echo "未対応のモードです: $2" >&2
          return 1
        fi
        shift 2
        ;;
      model-a|a)
        if [[ -n "${RSC_PROMPT_KEY}" ]]; then
          echo "モデル指定が複数回行われました" >&2
          return 1
        fi
        RSC_PROMPT_KEY="model-a"
        shift
        ;;
      model-b|b)
        if [[ -n "${RSC_PROMPT_KEY}" ]]; then
          echo "モデル指定が複数回行われました" >&2
          return 1
        fi
        RSC_PROMPT_KEY="model-b"
        shift
        ;;
      *)
        echo "未知の引数です: $1" >&2
        return 1
        ;;
    esac
  done

  if [[ -z "${RSC_PROMPT_KEY}" ]]; then
    RSC_PROMPT_KEY="model-a"
  fi

  return 0
}

rsc_prompt_file() {
  local root_dir="$1"
  local kind="$2" # ambiguous / precise
  local key="$3"

  local suffix
  case "${key}" in
    model-a)
      suffix="model_a"
      ;;
    model-b)
      suffix="model_b"
      ;;
    *)
      echo "未知のモデルです: ${key}" >&2
      return 1
      ;;
  esac

  echo "${root_dir}/docs/${kind}/prompt_${suffix}_${kind}.md"
}

rsc_reset_script() {
  local script_dir="$1"
  local key="$2"

  case "${key}" in
    model-a)
      echo "${script_dir}/model-a-code-reset.sh"
      ;;
    model-b)
      echo "${script_dir}/model-b-code-reset.sh"
      ;;
    *)
      echo "未知のモデルです: ${key}" >&2
      return 1
      ;;
  esac
}

rsc_ensure_file_exists() {
  local path="$1"
  local message="$2"
  if [[ ! -f "${path}" ]]; then
    echo "${message}: ${path}" >&2
    exit 2
  fi
}

rsc_ensure_executable() {
  local path="$1"
  local message="$2"
  if [[ ! -x "${path}" ]]; then
    echo "${message}: ${path}" >&2
    exit 3
  fi
}

rsc_exec_prompt() {
  local mode="$1"
  local prompt_file="$2"

  local cmd
  case "${mode}" in
  codex)
    cmd=(codex exec --full-auto)
    ;;
  claude)
    cmd=(claude --dangerously-skip-permissions --verbose -p)
    ;;
  gemini)
    cmd=(gemini --yolo -p)
    ;;
  *)
    echo "未対応のモードです: ${mode}" >&2
    exit 1
    ;;
  esac

  "${cmd[@]}" "$(cat "${prompt_file}")"
}

rsc_run_package_tests() {
  local root_dir="$1"
  local package="$2"
  shift 2
  local status=0

  echo "[tests] cargo test -p ${package} $*" >&2
  if ! (cd "${root_dir}" && cargo test -p "${package}" "$@" 2>&1); then
    echo "cargo test failed for package ${package} (continuing)" >&2
    status=1
  fi

  return ${status}
}

rsc_run_tests() {
  local root_dir="$1"
  local key="$2"
  local overall=0

  case "${key}" in
    model-a)
      rsc_run_package_tests "${root_dir}" "model-a-non-avdm" || overall=1
      rsc_run_package_tests "${root_dir}" "spec-tests" --test acceptance_model_a || overall=1
      ;;
    model-b)
      rsc_run_package_tests "${root_dir}" "model-b-avdm" || overall=1
      rsc_run_package_tests "${root_dir}" "spec-tests" --test acceptance_model_b || overall=1
      ;;
  esac

  return ${overall}
}

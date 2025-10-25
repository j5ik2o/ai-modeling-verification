#!/usr/bin/env bash

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  echo "このスクリプトは source して利用してください: ${BASH_SOURCE[0]}" >&2
  exit 1
fi

if [[ -n "${RWT_LIB_LOADED:-}" ]]; then
  return 0
fi
RWT_LIB_LOADED=1

rwt_run() {
  local script_dir="${1:-}"
  local repo_root="${2:-}"
  local target_script="${3:-}"
  local tmp_prefix="${4:-}"
  shift 4

  if [[ -z "${script_dir}" || -z "${repo_root}" || -z "${target_script}" || -z "${tmp_prefix}" ]]; then
    echo "rwt_run requires script_dir, repo_root, target_script, tmp_prefix" >&2
    return 1
  fi

  local run_timestamp
  run_timestamp="${RWT_RUN_TIMESTAMP:-$(date +%Y%m%d-%H%M%S)}"

  local worktree_root="${repo_root}/tmp/worktrees/run-${run_timestamp}"
  mkdir -p "${worktree_root}"

  local mode_hint="claude"
  local prompt_hint="model-a"
  local expect_mode_value=0

  for arg in "$@"; do
    if ((expect_mode_value)); then
      mode_hint="${arg}"
      expect_mode_value=0
      continue
    fi
    case "${arg}" in
      --mode=*)
        mode_hint="${arg#--mode=}"
        ;;
      --mode)
        expect_mode_value=1
        ;;
      model-a|a)
        prompt_hint="model-a"
        ;;
      model-b|b)
        prompt_hint="model-b"
        ;;
    esac
  done

  # 正規化してディレクトリ名に安全な形式へ揃える
  local safe_mode safe_prompt
  safe_mode="$(printf '%s' "${mode_hint}" | tr '[:upper:]' '[:lower:]' | tr -c '[:alnum:]-' '-')"
  safe_prompt="$(printf '%s' "${prompt_hint}" | tr '[:upper:]' '[:lower:]' | tr -c '[:alnum:]-' '-')"

  local worktree_path
  worktree_path="$(mktemp -d "${worktree_root}/${tmp_prefix}-${safe_prompt}-${safe_mode}.XXXXXX")"

  git -C "${repo_root}" worktree add --force "${worktree_path}" HEAD
  echo "worktree created: ${worktree_path}" >&2

  (cd "${worktree_path}" && "./scripts/${target_script}" "$@")

  echo "worktree preserved at ${worktree_path}" >&2
}

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
  local script_dir="$1"
  local repo_root="$2"
  local target_script="$3"
  local tmp_prefix="$4"
  shift 4

  local tmp_base
  tmp_base="$(mktemp -d "${TMPDIR:-/tmp}/${tmp_prefix}-worktree.XXXXXX")"
  local worktree_path="${tmp_base}/worktree"

  cleanup() {
    git -C "${repo_root}" worktree remove --force "${worktree_path}" >/dev/null 2>&1 || true
    rm -rf "${tmp_base}"
  }

  trap cleanup EXIT INT TERM

  git -C "${repo_root}" worktree add --force "${worktree_path}" HEAD

  (cd "${worktree_path}" && "./scripts/${target_script}" "$@")

  trap - EXIT INT TERM
  cleanup
}

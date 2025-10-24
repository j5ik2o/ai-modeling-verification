#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

TMP_BASE="$(mktemp -d "${TMPDIR:-/tmp}/precise-worktree.XXXXXX")"
WORKTREE_PATH="${TMP_BASE}/worktree"

cleanup() {
  git -C "${REPO_ROOT}" worktree remove --force "${WORKTREE_PATH}" >/dev/null 2>&1 || true
  rm -rf "${TMP_BASE}"
}
trap cleanup EXIT INT TERM

git -C "${REPO_ROOT}" worktree add --force "${WORKTREE_PATH}" HEAD

(cd "${WORKTREE_PATH}" && ./scripts/run-precise.sh "$@")

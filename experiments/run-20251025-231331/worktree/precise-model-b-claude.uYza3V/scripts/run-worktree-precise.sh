#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

source "${SCRIPT_DIR}/lib/run-worktree-common.sh"

rwt_run "${SCRIPT_DIR}" "${ROOT_DIR}" "run-precise.sh" "precise" "$@"

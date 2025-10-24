#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

timestamp="$(date +%Y%m%d-%H%M%S)"
log_root="${TMPDIR:-/tmp}/ai-modeling-verification-logs"
run_dir="${log_root}/run-${timestamp}"

mkdir -p "${run_dir}"

start_job() {
  local name="$1"
  shift
  local script_path="$1"
  shift
  local log_path="${run_dir}/${name}.log"

  nohup "${script_path}" "$@" >"${log_path}" 2>&1 &
  local pid=$!
  echo "${pid}" > "${run_dir}/${name}.pid"
  echo "${name} started (pid=${pid})" >&2
  echo "  log: ${log_path}" >&2
}

start_job "ambiguous" "${SCRIPT_DIR}/run-worktree-ambiguous.sh" "$@"
start_job "precise" "${SCRIPT_DIR}/run-worktree-precise.sh" "$@"

echo "jobs launched. details in ${run_dir}" >&2

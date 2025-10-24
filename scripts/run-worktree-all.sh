#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

timestamp="$(date +%Y%m%d-%H%M%S)"
log_root="${TMPDIR:-/tmp}/ai-modeling-verification-logs"
run_dir="${log_root}/run-${timestamp}"

mkdir -p "${run_dir}"

MODE="codex"
EXTRA_ARGS=()

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
      if [[ $# -gt 0 ]]; then
        EXTRA_ARGS+=("$@")
      fi
      break
      ;;
    *)
      EXTRA_ARGS+=("$1")
      shift
      ;;
  esac
done

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

start_job "ambiguous-a" "${SCRIPT_DIR}/run-worktree-ambiguous.sh" --mode "${MODE}" model-a "${EXTRA_ARGS[@]}"
start_job "ambiguous-b" "${SCRIPT_DIR}/run-worktree-ambiguous.sh" --mode "${MODE}" model-b "${EXTRA_ARGS[@]}"
start_job "precise-a" "${SCRIPT_DIR}/run-worktree-precise.sh" --mode "${MODE}" model-a "${EXTRA_ARGS[@]}"
start_job "precise-b" "${SCRIPT_DIR}/run-worktree-precise.sh" --mode "${MODE}" model-b "${EXTRA_ARGS[@]}"

echo "jobs launched. details in ${run_dir}" >&2

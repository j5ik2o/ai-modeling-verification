#!/usr/bin/env bash

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  echo "このスクリプトは source して利用してください: ${BASH_SOURCE[0]}" >&2
  exit 1
fi

rc_copy() {
  local script_dir="$1"
  local relative_source="$2"
  local relative_dest="$3"

  local repo_root
  repo_root="$(cd "${script_dir}/.." && pwd)"

  local source_path="${repo_root}/${relative_source}"
  local dest_path="${repo_root}/${relative_dest}"

  if [[ ! -f "${source_path}" ]]; then
    echo "reset source missing: ${source_path}" >&2
    return 1
  fi

  mkdir -p "$(dirname "${dest_path}")"
  cp "${source_path}" "${dest_path}"
}

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
source="${repo_root}/modules/model-a-non-avdm/reset/session.rs"
dest="${repo_root}/modules/model-a-non-avdm/src/session.rs"

cp "$source" "$dest"

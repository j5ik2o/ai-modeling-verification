#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
source1="${repo_root}/modules/model-b-avdm/reset/base.rs"
dest1="${repo_root}/modules/model-b-avdm/src/session/base.rs"

cp "$source1" "$dest1"

source2="${repo_root}/modules/model-b-avdm/reset/tests.rs"
dest2="${repo_root}/modules/model-b-avdm/src/session/tests.rs"

cp "$source2" "$dest2"

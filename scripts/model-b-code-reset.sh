#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "${SCRIPT_DIR}/lib/reset-common.sh"

rc_copy "${SCRIPT_DIR}" "modules/model-b-avdm/reset/base.rs" "modules/model-b-avdm/src/session/base.rs"
rc_copy "${SCRIPT_DIR}" "modules/model-b-avdm/reset/tests.rs" "modules/model-b-avdm/src/session/tests.rs"

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "${SCRIPT_DIR}/lib/reset-common.sh"

rc_copy "${SCRIPT_DIR}" "modules/model-a-non-avdm/reset/session.rs" "modules/model-a-non-avdm/src/session.rs"

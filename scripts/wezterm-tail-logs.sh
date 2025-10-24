#!/usr/bin/env bash
set -euo pipefail

LOG_ROOT="${TMPDIR:-/tmp}/ai-modeling-verification-logs"
RUN_DIR="$(ls -td "${LOG_ROOT}"/run-* 2>/dev/null | head -n 1)"

if [[ -z "${RUN_DIR}" ]]; then
  echo "最新のログディレクトリが見つかりません" >&2
  exit 1
fi

files=(ambiguous-a.log ambiguous-b.log precise-a.log precise-b.log)
missing=()
for f in "${files[@]}"; do
  [[ -f "${RUN_DIR}/${f}" ]] || missing+=("${f}")
done
if [[ ${#missing[@]} -gt 0 ]]; then
  echo "欠損しているログがあります: ${missing[*]}" >&2
fi

if ! command -v wezterm >/dev/null 2>&1; then
  echo "wezterm CLI が見つかりません。インストールと CLI サポートを有効化してください" >&2
  exit 1
fi

before_json="$(mktemp)"
after_json="$(mktemp)"

cleanup_tmp() {
  rm -f "${before_json}" "${after_json}"
}
trap cleanup_tmp EXIT

capture_state() {
  local path="$1"
  wezterm cli list --format json >"${path}"
}

extract_new_window_and_pane() {
  python3 - "$1" "$2" <<'PY'
import json, sys
before = json.load(open(sys.argv[1]))
after = json.load(open(sys.argv[2]))
before_windows = {entry["window_id"] for entry in before}
for entry in after:
    wid = entry["window_id"]
    if wid not in before_windows:
        print(f"{wid} {entry['pane_id']}")
        break
else:
    raise SystemExit("could not find new window")
PY
}

extract_new_pane() {
  python3 - "$1" "$2" "$3" <<'PY'
import json, sys
before = json.load(open(sys.argv[1]))
after = json.load(open(sys.argv[2]))
target_window = int(sys.argv[3])
before_panes = {entry["pane_id"] for entry in before if entry["window_id"] == target_window}
for entry in after:
    if entry["window_id"] == target_window and entry["pane_id"] not in before_panes:
        print(entry["pane_id"])
        break
else:
    raise SystemExit("could not find new pane")
PY
}

# 1. 新しいウィンドウで左上ペインを作成
capture_state "${before_json}"
wezterm cli spawn --new-window --cwd "${RUN_DIR}" -- sh -lc "tail -f ambiguous-a.log"
capture_state "${after_json}"
read -r TARGET_WINDOW LEFT_TOP_PANE < <(extract_new_window_and_pane "${before_json}" "${after_json}")

# ウィンドウタイトルを設定
wezterm cli set-window-title --window-id "${TARGET_WINDOW}" "AI Logs"
wezterm cli set-tab-title --pane-id "${LEFT_TOP_PANE}" "ambiguous-a"

# 2. 左上ペインを左右に分割し、右上を取得
capture_state "${before_json}"
wezterm cli split-pane --pane-id "${LEFT_TOP_PANE}" --right --cwd "${RUN_DIR}" -- \
  sh -lc "tail -f ambiguous-b.log"
capture_state "${after_json}"
RIGHT_TOP_PANE="$(extract_new_pane "${before_json}" "${after_json}" "${TARGET_WINDOW}")"
wezterm cli set-tab-title --pane-id "${RIGHT_TOP_PANE}" "ambiguous-b"

# 3. 左列を上下に分割し、左下を取得
capture_state "${before_json}"
wezterm cli split-pane --pane-id "${LEFT_TOP_PANE}" --bottom --percent 50 --cwd "${RUN_DIR}" -- \
  sh -lc "tail -f precise-a.log"
capture_state "${after_json}"
LEFT_BOTTOM_PANE="$(extract_new_pane "${before_json}" "${after_json}" "${TARGET_WINDOW}")"
wezterm cli set-tab-title --pane-id "${LEFT_BOTTOM_PANE}" "precise-a"

# 4. 右列を上下に分割し、右下を取得
capture_state "${before_json}"
wezterm cli split-pane --pane-id "${RIGHT_TOP_PANE}" --bottom --percent 50 --cwd "${RUN_DIR}" -- \
  sh -lc "tail -f precise-b.log"
capture_state "${after_json}"
RIGHT_BOTTOM_PANE="$(extract_new_pane "${before_json}" "${after_json}" "${TARGET_WINDOW}")"
wezterm cli set-tab-title --pane-id "${RIGHT_BOTTOM_PANE}" "precise-b"

# 5. フォーカスを左上に戻す
wezterm cli activate-pane-direction --pane-id "${RIGHT_TOP_PANE}" Left

echo "ログ監視を開始しました (ディレクトリ: ${RUN_DIR})"

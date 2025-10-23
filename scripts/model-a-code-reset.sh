#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
target="${repo_root}/modules/model-a-non-avdm/src/session.rs"

python3 - "$target" <<'PY'
import sys
import re
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")


DOC_LINE = "/// `Session` の生データを手続き的に処理して料金を算出する。\n"
DOC_PATTERN = re.compile(
    r"(?:^[\\t ]*///.*\n)+(?=^[\\t ]*pub fn calculate_charge)", re.MULTILINE
)


def normalize_doc_comment(source: str) -> str:
    match = DOC_PATTERN.search(source)
    if not match:
        raise SystemExit("calculate_charge のドキュメントコメントが見つかりません")

    if match.group(0) == DOC_LINE:
        return source

    return source[: match.start()] + DOC_LINE + source[match.end() :]


def reset_function_body(source: str, name: str) -> str:
    source = normalize_doc_comment(source)

    marker = f"pub fn {name}"
    idx = source.find(marker)
    if idx == -1:
        raise SystemExit(f"関数 {name} のシグネチャが {path} で見つかりません")

    brace_start = source.find("{", idx)
    if brace_start == -1:
        raise SystemExit(f"関数 {name} に対応する開き波括弧が見つかりません")

    depth = 0
    end_idx = None
    for pos in range(brace_start, len(source)):
        ch = source[pos]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end_idx = pos
                break
    if end_idx is None:
        raise SystemExit(f"関数 {name} に対応する閉じ波括弧が見つかりません")

    body = source[brace_start + 1 : end_idx]

    indent = None
    for line in body.splitlines():
        stripped = line.lstrip()
        if stripped:
            indent = line[: len(line) - len(stripped)]
            break
    if indent is None:
        indent = "  "

    stub = f"\n{indent}todo!(\"AIに実装させる\")\n"

    if body == stub:
        return source

    return source[: brace_start + 1] + stub + source[end_idx:]


def remove_tests_module(source: str) -> str:
    marker = "#[cfg(test)]"
    idx = source.find(marker)
    if idx == -1:
        return source

    mod_idx = source.find("mod tests", idx)
    if mod_idx == -1:
        return source

    brace_start = source.find("{", mod_idx)
    if brace_start == -1:
        return source

    depth = 0
    end_idx = None
    for pos in range(brace_start, len(source)):
        ch = source[pos]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end_idx = pos
                break

    if end_idx is None:
        return source

    return source[:idx] + source[end_idx + 1 :]


updated = reset_function_body(text, "calculate_charge")
updated = remove_tests_module(updated)

if updated != text:
    path.write_text(updated, encoding="utf-8")
    print(f"{path} を初期状態にリセットしました")
else:
    print(f"{path} は既に初期状態です")
PY

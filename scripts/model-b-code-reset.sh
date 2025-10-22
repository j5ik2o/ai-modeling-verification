#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
target="${repo_root}/modules/model-b-avdm/src/session/base.rs"

python3 - "$target" <<'PY'
import sys
import re
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")

DOC_LINES = {
    "stop": "セッションを停止し、請求を確定させる。",
    "bill_snapshot": "指定時点での課金スナップショットを取得する。",
    "bill_after_stop": "停止後の追加課金要求に応答する。",
}


def normalize_doc_comment(source: str, name: str) -> str:
    doc_text = DOC_LINES[name]
    pattern = re.compile(
        rf"(?P<block>(?:(?:^[\t ]*)///.*\n)+)(?=^[\t ]*pub fn {name}\b)",
        re.MULTILINE,
    )

    match = pattern.search(source)
    fn_match = re.search(rf"^[\t ]*pub fn {name}\b", source, re.MULTILINE)
    if not fn_match:
        raise SystemExit(f"関数 {name} のシグネチャが {path} で見つかりません")

    indent = source[fn_match.start() : fn_match.end()].split("pub", 1)[0]
    desired = f"{indent}/// {doc_text}\n"

    if not match:
        return source[: fn_match.start()] + desired + source[fn_match.start() :]

    block = match.group("block")
    post = source[match.end("block") :]
    while post.startswith("\n"):
        post = post[1:]

    updated = source[: match.start("block")] + desired + post
    return updated


def reset_function_body(source: str, name: str, *, normalize_doc: bool) -> str:
    if normalize_doc and name in DOC_LINES:
        source = normalize_doc_comment(source, name)

    fn_pattern = re.compile(rf"^[\t ]*(?:pub\s+)?fn {name}\b", re.MULTILINE)
    fn_match = fn_pattern.search(source)
    if not fn_match:
        raise SystemExit(f"関数 {name} のシグネチャが {path} で見つかりません")

    idx = source.find("fn", fn_match.start())
    if idx == -1:
        raise SystemExit(f"関数 {name} の `fn` トークンが {path} で見つかりません")

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
        indent = "    "

    stub = f"\n{indent}todo!(\"AIに実装させる\")\n"

    if body == stub:
        return source

    tail = source[end_idx:]
    line_end = source.find("\n", fn_match.start())
    if line_end == -1:
        line_end = len(source)
    fn_line = source[fn_match.start() : line_end]
    if "pub" in fn_line:
        fn_indent = fn_line.split("pub", 1)[0]
    else:
        fn_indent = fn_line.split("fn", 1)[0]
    if tail.startswith("}") and not tail.startswith(fn_indent + "}"):
        tail = fn_indent + "}" + tail[1:]
    elif tail.startswith("\n}") and not tail.startswith("\n" + fn_indent + "}"):
        tail = "\n" + fn_indent + "}" + tail[2:]

    return source[: brace_start + 1] + stub + tail


current = text
for fn_name in DOC_LINES:
    current = reset_function_body(current, fn_name, normalize_doc=True)

for fn_name in ("bill_snapshot_for", "billed_energy_for", "duration_millis"):
    current = reset_function_body(current, fn_name, normalize_doc=False)

if current != text:
    path.write_text(current, encoding="utf-8")
    print(f"{path} の対象メソッドを todo!(\"AIに実装させる\") にリセットしました")
else:
    print(f"{path} の対象メソッドは既に todo!(\"AIに実装させる\") です")
PY

#!/usr/bin/env python3
"""核对上游在某区间内新增的每一行是否都进入合并结果；缺失即可能是合并丢弃了上游改动。

合并 upstream 后用于代替人工读 diff：正常缺失只应是「被 t! 替换的英文字面量」与
「两侧各加一项的 use 导入」两类，其余缺失都要逐个查原因。

用法：
  python3 scripts/check-upstream-added.py f126950f9d upstream/main
  python3 scripts/check-upstream-added.py <基线> <上游 ref> --min-len 12
"""

import argparse
import subprocess
import sys
from pathlib import Path


def run(args: list[str]) -> str:
    return subprocess.run(args, capture_output=True, text=True, check=True).stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", help="上次合并的上游基线提交")
    parser.add_argument("upstream", help="本次合并的上游 ref")
    parser.add_argument("--min-len", type=int, default=12, help="忽略过短的噪声行")
    opts = parser.parse_args()

    root = Path(run(["git", "rev-parse", "--show-toplevel"]).strip())
    files = [
        path
        for path in run(["git", "diff", "--name-only", f"{opts.base}..{opts.upstream}"]).split()
        if path.endswith(".rs")
    ]
    findings = 0
    for path in files:
        diff = run(["git", "diff", "-U0", f"{opts.base}..{opts.upstream}", "--", path])
        added = [
            line[1:].strip()
            for line in diff.splitlines()
            if line.startswith("+")
            and not line.startswith("+++")
            and len(line[1:].strip()) >= opts.min_len
        ]
        if not added:
            continue
        try:
            merged = {line.strip() for line in open(root / path, encoding="utf-8")}
        except (FileNotFoundError, IsADirectoryError):
            merged = set()
        missing = [line for line in added if line not in merged]
        if missing:
            findings += 1
            print(f"\n### {path}: 上游新增 {len(added)} 行，合并结果缺 {len(missing)} 行")
            for line in missing:
                print("   -", line[:160])
    if findings == 0:
        print(f"上游新增行全部进入合并结果（检查 {len(files)} 个 .rs 文件）")
    else:
        print(f"\n共 {findings} 个文件有缺失行，逐条确认是汉化替换还是真实丢失")
    return 0


if __name__ == "__main__":
    sys.exit(main())

"""把本仓库的版本号同步为上游（zed-industries/zed）的版本号。

fork 手改 `crates/zed/Cargo.toml` 的版本行会与上游长期冲突，发布时也容易与上游
版本号错位。本脚本统一从上游取版本号，写到 `crates/zed/Cargo.toml` 与
`Cargo.lock` 的 `zed` 条目；两处必须一致，否则 cargo 会报 lock 不一致。

用法：
  python3 scripts/sync-upstream-version.py            # 取 upstream/main 的版本号并写入
  python3 scripts/sync-upstream-version.py --check    # 只校验，不一致则退出码 1
  python3 scripts/sync-upstream-version.py --source stable  # 取上游最新正式 release 的标签
  python3 scripts/sync-upstream-version.py --version 1.27.0 # 离线指定本次发布版本号
  python3 scripts/sync-upstream-version.py --check           # 与上游 main 完全一致
  python3 scripts/sync-upstream-version.py --check-not-behind stable  # 不落后于上游最新正式 release
  python3 scripts/sync-upstream-version.py --print           # 打印仓库当前版本号

发布工作流用 `--check-not-behind stable` 加「标签与 Cargo.toml 一致」作为门禁：版本号允许
由发布者指定（例如发布基于上游 main 的自有版本号），但不得低于上游已发布的正式版本，
且标签、Release 标题、二进制自报版本必须一致。
"""

import argparse
import json
import re
import sys
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
ZED_MANIFEST = REPO_ROOT / "crates" / "zed" / "Cargo.toml"
CARGO_LOCK = REPO_ROOT / "Cargo.lock"

MAIN_MANIFEST_URL = (
    "https://raw.githubusercontent.com/zed-industries/zed/main/crates/zed/Cargo.toml"
)
LATEST_RELEASE_URL = "https://api.github.com/repos/zed-industries/zed/releases/latest"

MANIFEST_VERSION = re.compile(r'(?m)^version = "([^"]+)"')
LOCK_ZED_VERSION = re.compile(r'(?m)^name = "zed"\r?\nversion = "([^"]+)"')


def fetch(url: str) -> str:
    request = urllib.request.Request(url, headers={"User-Agent": "zed-zh-cn-sync-version"})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.read().decode("utf-8")
    except urllib.error.URLError as error:
        # 网络不可达时给出可操作的提示（本机需 HTTPS_PROXY，CI runner 直连）
        raise SystemExit(
            f"请求上游失败：{url}\n{error}\n"
            "网络受限时先设置 HTTPS_PROXY（CI runner 不受影响），或改用 --version 指定版本号。"
        ) from error


def upstream_version(source: str) -> str:
    if source == "main":
        match = MANIFEST_VERSION.search(fetch(MAIN_MANIFEST_URL))
        if match is None:
            raise SystemExit(f"上游 main 的 Cargo.toml 找不到 version 行：{MAIN_MANIFEST_URL}")
        return match.group(1)
    if source == "stable":
        tag = json.loads(fetch(LATEST_RELEASE_URL))["tag_name"]
        return tag[1:] if tag.startswith("v") else tag
    raise SystemExit(f"未知来源：{source}")


def compare_version(left: str, right: str) -> int:
    """按数值比较版本号，忽略预发布后缀（1.27.0-pre 视为 1.27.0）。"""

    def numbers(version: str) -> tuple[int, ...]:
        core = re.split(r"[-+]", version, maxsplit=1)[0]
        return tuple(int(part) for part in core.split(".") if part.isdigit())

    left_numbers, right_numbers = numbers(left), numbers(right)
    return (left_numbers > right_numbers) - (left_numbers < right_numbers)


def read_versions() -> tuple[str, str]:
    # 只取 [package] 段，避免命中依赖表里的同名键
    manifest = ZED_MANIFEST.read_text(encoding="utf-8").partition("[package]")[2]
    manifest_match = MANIFEST_VERSION.search(manifest)
    lock_match = LOCK_ZED_VERSION.search(CARGO_LOCK.read_text(encoding="utf-8"))
    if manifest_match is None:
        raise SystemExit(f"{ZED_MANIFEST} 的 [package] 段找不到 version 行")
    if lock_match is None:
        raise SystemExit(f"{CARGO_LOCK} 找不到 name = \"zed\" 包条目")
    return manifest_match.group(1), lock_match.group(1)


def replace_once(path: Path, pattern: re.Pattern[str], replacement: str, label: str) -> bool:
    raw = path.read_bytes()
    text = raw.decode("utf-8")
    updated, count = pattern.subn(replacement, text, count=1)
    if count != 1:
        raise SystemExit(f"{path} 中找不到{label}")
    if updated == text:
        return False
    # 保持原文件的换行风格，避免把 CRLF 文件写成混合行尾
    path.write_bytes(updated.encode("utf-8"))
    return True


def main() -> None:
    parser = argparse.ArgumentParser(description="同步仓库版本号为上游版本号")
    parser.add_argument(
        "--source",
        choices=["main", "stable"],
        default="main",
        help="main 取上游 main 的版本号（默认）；stable 取上游最新正式 release 的标签",
    )
    parser.add_argument("--version", help="直接指定版本号，跳过网络查询")
    parser.add_argument("--check", action="store_true", help="只校验仓库版本与上游完全一致")
    parser.add_argument(
        "--check-not-behind",
        choices=["main", "stable"],
        help="只校验仓库版本不低于该上游来源的版本号",
    )
    parser.add_argument("--print", action="store_true", help="打印仓库当前版本号（CI 比对标签用）")
    args = parser.parse_args()

    manifest_version, lock_version = read_versions()

    if args.print:
        print(manifest_version)
        return

    if args.check_not_behind:
        upstream = upstream_version(args.check_not_behind)
        if compare_version(manifest_version, upstream) < 0:
            print(
                f"仓库版本 {manifest_version} 低于上游（{args.check_not_behind}）{upstream}。\n"
                "先运行 `python3 scripts/sync-upstream-version.py` 同步，或调整本次发布版本号。",
                file=sys.stderr,
            )
            raise SystemExit(1)
        if lock_version != manifest_version:
            print(
                f"Cargo.lock 的 zed 版本 {lock_version} 与 Cargo.toml 的 {manifest_version} 不一致。\n"
                "运行 `python3 scripts/sync-upstream-version.py` 修正后提交。",
                file=sys.stderr,
            )
            raise SystemExit(1)
        print(
            f"版本号 {manifest_version} 不低于上游（{args.check_not_behind}）{upstream}，"
            "且与 Cargo.lock 一致"
        )
        return

    version = args.version or upstream_version(args.source)

    if args.check:
        if manifest_version != version:
            print(
                f"版本号与上游不一致：仓库 {manifest_version}，上游（{args.source}）{version}。\n"
                f"运行 `python3 scripts/sync-upstream-version.py --source {args.source}` "
                "并提交后再发布。",
                file=sys.stderr,
            )
            raise SystemExit(1)
        if lock_version != manifest_version:
            print(
                f"Cargo.lock 的 zed 版本 {lock_version} 与 Cargo.toml 的 {manifest_version} 不一致。\n"
                "运行 `python3 scripts/sync-upstream-version.py` 修正后提交。",
                file=sys.stderr,
            )
            raise SystemExit(1)
        print(f"版本号已与上游一致：{manifest_version}")
        return

    changed = [
        path
        for path, pattern, replacement, label in [
            (ZED_MANIFEST, re.compile(r'(?m)^version = "[^"]*"'), f'version = "{version}"', " version 行"),
            (
                CARGO_LOCK,
                re.compile(r'(?m)^(name = "zed"\r?\n)version = "[^"]*"'),
                rf'\g<1>version = "{version}"',
                ' name = "zed" 包条目',
            ),
        ]
        if replace_once(path, pattern, replacement, label)
    ]

    if not changed:
        print(f"版本号已是 {version}，无需修改")
        return
    for path in changed:
        print(f"已更新 {path.relative_to(REPO_ROOT)}")
    source = "命令行指定" if args.version else args.source
    print(f"版本号 {manifest_version} → {version}（来源：{source}）")


if __name__ == "__main__":
    main()

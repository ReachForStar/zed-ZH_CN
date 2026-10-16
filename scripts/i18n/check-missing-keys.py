"""检查代码里 t!("key") 引用的键是否都在 en 与 zh-CN locale 中存在，并比对两侧键集合。

用法：python3 scripts/i18n/check-missing-keys.py
退出码：0 = 无缺失且两侧一致；1 = 有缺失或两侧不一致。

只收集静态字面量键（`t!("a.b")` 与 `t!("a.b", 参数 = 值)`）；ts! / tr! 等变体与动态拼接的键
不进本检查，缺失时会以「代码引用但 locale 缺失」行报出。
"""

import pathlib
import re
import sys
import tomllib

REPO = pathlib.Path(__file__).resolve().parents[2]
LOCALES = REPO / "crates/zed_i18n/locales"
KEY_RE = re.compile(r'\bt!\(\s*"([A-Za-z0-9_.\-]+)"')
SKIP_DIRS = {"target", "node_modules", ".git"}


def flatten(node, prefix=""):
    for key, value in node.items():
        full = f"{prefix}.{key}" if prefix else key
        if isinstance(value, dict):
            yield from flatten(value, full)
        else:
            yield full


def load_side(locale):
    keys = set()
    for path in sorted((LOCALES / locale).glob("*.toml")):
        keys.update(flatten(tomllib.loads(path.read_text(encoding="utf-8"))))
    root = LOCALES / f"{locale}.toml"
    if root.exists():
        keys.update(flatten(tomllib.loads(root.read_text(encoding="utf-8"))))
    keys.discard("_version")
    return keys


def collect_used():
    used = {}
    for path in REPO.glob("crates/**/*.rs"):
        if SKIP_DIRS & set(path.parts):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        # 屏蔽行注释（含 `///` 文档示例），保留行数以便报行号
        masked = "\n".join("" if line.lstrip().startswith("//") else line for line in text.split("\n"))
        for match in KEY_RE.finditer(masked):
            key = match.group(1)
            lineno = masked[: match.start()].count("\n") + 1
            used.setdefault(key, []).append(f"{path.relative_to(REPO)}:{lineno}")
    return used


def main():
    en = load_side("en")
    zh = load_side("zh-CN")
    used = collect_used()
    problems = []

    for key in sorted(set(used) - en):
        problems.append(f"代码引用但 en 缺失：{key}（{used[key][0]}）")
    for key in sorted(set(used) - zh):
        problems.append(f"代码引用但 zh-CN 缺失：{key}（{used[key][0]}）")
    for key in sorted(en - zh):
        problems.append(f"仅 en 有：{key}")
    for key in sorted(zh - en):
        problems.append(f"仅 zh-CN 有：{key}")

    print(f"代码静态引用键 {len(used)} 个；en {len(en)} 个；zh-CN {len(zh)} 个")
    for line in problems:
        print("  " + line)
    print(f"问题 {len(problems)} 项")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

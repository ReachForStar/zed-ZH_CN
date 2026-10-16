"""按计划表把一个 crate 的界面文本替换为 t! 调用，并同步写入 en / zh-CN locale 文件。

用法：python3 scripts/i18n/i18n-apply.py <crate> [后缀] [计划目录，默认 tmp/i18n-plan]
（后缀用于一个 crate 拆多批时区分文件名，如 `workspace part2` 读 <计划目录>/workspace part2.tsv）

计划表（TSV，UTF-8，LF）：<计划目录>/<crate>.tsv，每行
    <相对路径>\t<Rust 源码中的英文字面量内容>\t<完整 key>\t<中文译文>
    [\t<该文件内预期出现次数，默认 1>][\t<行号；`allbut:<行号>` 表示该行排除；`last` 由 mkplan 消费>]

行为：
1. 把文件中的 "<英文原文>" 替换为 t!("<key>")，出现次数与预期不符则报错且不写入；
2. 给该文件补 `use zed_i18n::t;`（若缺）；
3. 若 crates/<crate>/Cargo.toml 未依赖 zed_i18n，则补 `zed_i18n.workspace = true`；
4. 把 key 写入 crates/zed_i18n/locales/{en,zh-CN}/<crate>.*.toml：
   沿用该文件已有风格——有表头则写入对应表，无表头则写扁平点号键；
   文件不存在时新建为 `[<crate>]` 表风格；写后用 tomllib 回读校验。
"""

import os
import re
import sys
import tomllib
from collections import OrderedDict

LOCALES = "crates/zed_i18n/locales"
TABLE_RE = re.compile(r"^\[([^\]]+)\]\s*$")


def insert_import(text: str) -> str:
    if re.search(r"^use zed_i18n::t;\s*$", text, flags=re.M):
        return text
    lines = text.split("\n")
    target = None
    for i, line in enumerate(lines):
        if line.startswith("use "):
            target = i
            break
    if target is None:
        for i, line in enumerate(lines):
            s = line.strip()
            if not s or s.startswith("//") or s.startswith("#!"):
                continue
            target = i
            break
    if target is None:
        sys.exit("找不到插入 use 语句的位置")
    lines.insert(target, "use zed_i18n::t;")
    return "\n".join(lines)


def ensure_dependency(crate: str) -> None:
    path = f"crates/{crate}/Cargo.toml"
    if not os.path.exists(path):
        sys.exit(f"{path} 不存在")
    text = open(path, encoding="utf-8", newline="").read()
    if re.search(r"^zed_i18n\b", text, flags=re.M):
        return
    lines = text.split("\n")
    start = next((i for i, l in enumerate(lines) if l.strip() == "[dependencies]"), None)
    if start is None:
        sys.exit(f"{path} 找不到 [dependencies]")
    pos = None
    for i in range(start + 1, len(lines)):
        l = lines[i]
        if l.startswith("["):
            break
        if re.match(r"^[A-Za-z0-9_\-]+", l) and l > "zed_i18n":
            pos = i
            break
    if pos is None:
        pos = next(
            (i for i in range(start + 1, len(lines)) if lines[i].startswith("[")), len(lines)
        )
    lines.insert(pos, "zed_i18n.workspace = true")
    open(path, "w", encoding="utf-8", newline="").write("\n".join(lines))
    print(f"{path}: 已补 zed_i18n 依赖")


def resolve(data: dict, key: str):
    node = data
    for part in key.split("."):
        if not isinstance(node, dict) or part not in node:
            return None
        node = node[part]
    return node


def update_locale(path: str, table: str, key: str, value: str) -> None:
    if re.search(r"[\r\n]", value):
        sys.exit(f"{key}: 译文含换行，需手工处理")
    if os.path.exists(path):
        with open(path, "rb") as f:
            data = tomllib.load(f)
        if resolve(data, key) is not None:
            if resolve(data, key) == value:
                return  # 键已存在且一致，直接复用
            sys.exit(f"{path}: 键 {key} 已存在且值不同（{resolve(data, key)!r} != {value!r}）")
    line_value = value.replace("\\", "\\\\").replace('"', '\\"')
    short = key[len(table) + 1 :]
    if not os.path.exists(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(f"_version = 1\n\n[{table}]\n{short} = \"{line_value}\"\n".replace("\n", "\r\n"))
    else:
        crlf = b"\r\n" in open(path, "rb").read()
        with open(path, encoding="utf-8", newline="") as f:
            text = f.read()
        text = text.replace("\r\n", "\n")
        lines = text.split("\n")
        if not any(TABLE_RE.match(l) for l in lines):
            # 扁平点号键风格：在文件末尾追加 `完整.key = "译文"`
            if any(l.startswith(f"{key} ") or l.startswith(f"{key}=") for l in lines):
                sys.exit(f"{path}: 已存在键 {key}")
            while lines and not lines[-1].strip():
                lines.pop()
            lines.append(f'{key} = "{line_value}"')
            lines.append("")
        else:
            header = nxt = None
            for i, l in enumerate(lines):
                m = TABLE_RE.match(l)
                if m and m.group(1) == table:
                    header = i
                elif header is not None and m and i > header:
                    nxt = i
                    break
            if header is None:
                while lines and not lines[-1].strip():
                    lines.pop()
                lines += ["", f"[{table}]", f'{short} = "{line_value}"', ""]
            else:
                end = len(lines) if nxt is None else nxt
                if any(l.strip().split(" ")[0].split("=")[0] == short for l in lines[header + 1 : end]):
                    sys.exit(f"{path}: 表 [{table}] 已存在键 {short}")
                pos = end
                while pos - 1 > header and not lines[pos - 1].strip():
                    pos -= 1
                lines.insert(pos, f'{short} = "{line_value}"')
        out = "\n".join(lines)
        if crlf:
            out = out.replace("\n", "\r\n")
        with open(path, "w", encoding="utf-8", newline="") as f:
            f.write(out)
    # 回读校验：写入的键必须能按点号路径取到，且值等于原文
    with open(path, "rb") as f:
        data = tomllib.load(f)
    node = data
    for part in key.split("."):
        if not isinstance(node, dict) or part not in node:
            sys.exit(f"{path}: 校验失败，键 {key} 未被正确写入")
        node = node[part]
    if node != value:
        sys.exit(f"{path}: 校验失败，键 {key} 的值为 {node!r}，期望 {value!r}")


def main():
    crate = sys.argv[1]
    suffix = sys.argv[2] if len(sys.argv) > 2 else ""
    plan_dir = sys.argv[3] if len(sys.argv) > 3 else "tmp/i18n-plan"
    plan_path = f"{plan_dir}/{crate}{suffix}.tsv"
    rows = OrderedDict()
    for raw in open(plan_path, encoding="utf-8", newline="").read().split("\n"):
        if not raw.strip() or raw.startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) < 4:
            sys.exit(f"计划表行字段不足：{raw!r}")
        path, en, key, zh = parts[:4]
        expect = int(parts[4]) if len(parts) > 4 and parts[4].strip() else 1
        line = parts[5] if len(parts) > 5 and parts[5].strip() else None
        rows[path] = rows.get(path, []) + [(en, key, zh, expect, line)]

    ensure_dependency(crate)
    for path, entries in rows.items():
        with open(path, "rb") as f:
            data = f.read()
        crlf = data.count(b"\r\n")
        lf = data.count(b"\n") - crlf
        if crlf and lf:
            sys.exit(f"{path}: 行尾混用，拒绝处理")
        text = data.decode("utf-8").replace("\r\n", "\n")
        # 先整体校验所有条目的出现次数，一次性报出全部不匹配，避免逐个往返
        problems = []
        for en, _key, _zh, expect, line in entries:
            needle = f'"{en}"'
            if line is not None and line.startswith("allbut:"):
                skip = int(line.split(":", 1)[1]) - 1
                seg = text.split("\n")
                found = sum(l.count(needle) for i, l in enumerate(seg) if i != skip)
                hint = f"除第 {skip + 1} 行外"
            elif line is not None:
                idx = int(line) - 1
                found = text.split("\n")[idx].count(needle)
                hint = f"第 {idx + 1} 行"
            else:
                found = text.count(needle)
                hint = ""
            if found != expect:
                problems.append(f"  {hint}期望 {expect} 处 {needle}，实际 {found} 处")
        if problems:
            sys.exit(f"{path} 出现次数不匹配：\n" + "\n".join(problems))
        for en, key, _zh, _expect, line in entries:
            needle = f'"{en}"'
            if line is not None and line.startswith("allbut:"):
                # 同一字面量在文件内多处出现，但某一行是 action/键位名等非界面用途，需排除
                skip = int(line.split(":", 1)[1]) - 1
                seg = text.split("\n")
                seg = [
                    l if i == skip else l.replace(needle, f't!("{key}")')
                    for i, l in enumerate(seg)
                ]
                text = "\n".join(seg)
                continue
            if line is not None:
                # 同一字面量在文件内多处出现时，用行号锁定要替换的那一处
                idx = int(line) - 1
                seg = text.split("\n")
                seg[idx] = seg[idx].replace(needle, f't!("{key}")')
                text = "\n".join(seg)
                continue
            text = text.replace(needle, f't!("{key}")')
        text = insert_import(text)
        out = text.replace("\n", "\r\n") if crlf else text
        with open(path, "wb") as f:
            f.write(out.encode("utf-8"))
        print(f"{path}: 替换 {len(entries)} 条")

    for _path, entries in rows.items():
        for en, key, zh, _e, _l in entries:
            table = key.rsplit(".", 1)[0]
            update_locale(os.path.join(LOCALES, "en", f"{crate}.en.toml"), table, key, en)
            update_locale(os.path.join(LOCALES, "zh-CN", f"{crate}.zh-CN.toml"), table, key, zh)
    print(f"已写入 {sum(len(v) for v in rows.values())} 条 locale 键")


main()

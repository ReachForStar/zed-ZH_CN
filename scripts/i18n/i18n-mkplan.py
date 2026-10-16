"""按 `文件:行<TAB>key<TAB>译文` 生成汉化计划行，自动从源码取该行的英文原文。

用法：python3 scripts/i18n/i18n-mkplan.py <输入文件> <输出 tsv>
输入文件每行：相对路径:行号<TAB>key<TAB>译文[<TAB>期望出现次数[<TAB>行限定]]
可选第三列后追加 ``;`` 分隔的修饰：``skipword`` 表示取该行最后一个字面量（少见）。
"""

import pathlib
import re
import sys

LITERAL = re.compile(r'"((?:[^"\\]|\\.)*)"')


def main() -> None:
    rows = []
    for raw in pathlib.Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        parts = raw.split("\t")
        loc, key, zh = parts[0], parts[1], parts[2]
        extra = parts[3:]
        path, lineno = loc.rsplit(":", 1)
        text = pathlib.Path(path).read_text(encoding="utf-8").replace("\r\n", "\n")
        line = text.split("\n")[int(lineno) - 1]
        found = LITERAL.findall(line)
        if not found:
            sys.exit(f"{loc} 该行没有字面量：{line!r}")
        en = found[-1] if extra and extra[0] == "last" else found[0]
        if extra and extra[0] == "last":
            extra = extra[1:]
        rows.append("\t".join([path, en, key, zh] + extra))
    pathlib.Path(sys.argv[2]).write_text("\n".join(rows) + "\n", encoding="utf-8")
    print(f"{sys.argv[2]}: {len(rows)} 行")
    for r in rows:
        print("  " + r)


main()

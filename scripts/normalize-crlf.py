"""把变更文件的工作区行尾统一为 CRLF（工作区约定 CRLF，Git 存储 LF）。

`cargo fmt` / `clippy --fix` 重排过的文件会被写成 LF，与工作区其余文件混用行尾，
提交前需要还原。无参数时处理 `git diff HEAD` 里的 .rs 文件；给路径时按参数处理。

用法：
  python3 scripts/normalize-crlf.py                # 还原本次改动涉及的 .rs 文件
  python3 scripts/normalize-crlf.py a.rs b.toml    # 只处理指定文件（任意扩展名）

只改写文本文件；含 NUL 字节的二进制与不存在的路径跳过。可重复执行（幂等）。
"""

import subprocess
import sys
from pathlib import Path


def changed_rust_files() -> list[Path]:
    output = subprocess.run(
        ["git", "diff", "--name-only", "-z", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return [Path(name) for name in output.split("\0") if name.endswith(".rs")]


def normalize(path: Path) -> bool:
    data = path.read_bytes()
    if not data or b"\0" in data:
        return False
    normalized = data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    if normalized == data:
        return False
    path.write_bytes(normalized)
    return True


def main() -> None:
    # 先做参数判断：无参时取 git 变更集，显式路径优先，避免误改整仓。
    paths = [Path(arg) for arg in sys.argv[1:]] or changed_rust_files()
    for path in paths:
        if path.is_file() and normalize(path):
            print(f"CRLF\t{path}")


if __name__ == "__main__":
    main()

"""AT-SPI のアクセシビリティツリーを書き出す（`tools/a11y-tree/run.sh` から呼ぶ）。

各行は `[役割] '名前' desc='説明' [状態...] value=値`。字下げが木の深さ。
"""

import pyatspi

STATES = [
    "focusable",
    "focused",
    "checked",
    "indeterminate",
    "selected",
    "expanded",
    "pressed",
    "editable",
    "sensitive",
    "enabled",
    "read_only",
    "modal",
    "invalid_entry",
]


def states(node):
    state = node.getState()
    return [name for name in STATES if state.contains(getattr(pyatspi, "STATE_" + name.upper()))]


def walk(node, depth):
    try:
        line = f"[{node.getRoleName()}] {node.name!r}"
        if node.description:
            line += f" desc={node.description!r}"
        line += f" {states(node)}"
    except Exception as error:  # ツリーが描画の途中で変わると読めないことがある
        print("  " * depth + f"<error {error}>")
        return
    try:
        line += f" value={node.queryValue().currentValue}"
    except NotImplementedError:
        pass
    print("  " * depth + line)
    for index in range(node.childCount):
        walk(node.getChildAtIndex(index), depth + 1)


desktop = pyatspi.Registry.getDesktop(0)
for index in range(desktop.childCount):
    walk(desktop.getChildAtIndex(index), 0)

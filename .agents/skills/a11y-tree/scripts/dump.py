"""AT-SPI に登録されたアプリの a11y ツリーを書き出す

使い方: python3 -I dump.py <アプリ名>

各行は `<パス> [role] 'name' {states} value=... desc='...' rel=[...] actions=[...]`。
パスは act.py に渡す子の番号の列
"""

import sys

import pyatspi

STATES = {
    pyatspi.STATE_SHOWING: "showing",
    pyatspi.STATE_FOCUSABLE: "focusable",
    pyatspi.STATE_FOCUSED: "focused",
    pyatspi.STATE_SENSITIVE: "sensitive",
    pyatspi.STATE_READ_ONLY: "read-only",
    pyatspi.STATE_CHECKABLE: "checkable",
    pyatspi.STATE_CHECKED: "checked",
    pyatspi.STATE_INDETERMINATE: "indeterminate",
    pyatspi.STATE_PRESSED: "pressed",
    pyatspi.STATE_SELECTED: "selected",
    pyatspi.STATE_EXPANDED: "expanded",
    pyatspi.STATE_MODAL: "modal",
    pyatspi.STATE_INVALID_ENTRY: "invalid",
}


def describe(acc):
    states = acc.getState()
    names = [name for state, name in STATES.items() if states.contains(state)]
    relations = []
    for relation in acc.getRelationSet():
        targets = [relation.getTarget(i) for i in range(relation.getNTargets())]
        relations.append(
            f"{relation.getRelationType().value_nick}->"
            + ",".join(repr(target.name) for target in targets)
        )
    try:
        action = acc.queryAction()
        actions = [action.getName(i) for i in range(action.nActions)]
    except NotImplementedError:
        actions = []
    line = f"[{acc.getRoleName()}] {acc.name!r} {{{', '.join(names)}}}"
    try:
        value = acc.queryValue()
        line += f" value={value.currentValue} [{value.minimumValue}..{value.maximumValue}]"
    except NotImplementedError:
        pass
    if acc.description:
        line += f" desc={acc.description!r}"
    if relations:
        line += f" rel={relations}"
    if actions:
        line += f" actions={actions}"
    return line


def walk(acc, path):
    indent = "  " * len(path)
    label = ".".join(map(str, path)) or "-"
    # アプリはフレームごとにツリーを更新するため、書き出しの途中で消えたノードは印を付けて飛ばす
    try:
        print(f"{indent}{label} {describe(acc)}")
        count = acc.childCount
    except Exception as error:
        print(f"{indent}{label} (gone: {error})")
        return
    for i in range(count):
        child = acc.getChildAtIndex(i)
        if child is None:
            print(f"{indent}  {label}.{i} (gone)")
            continue
        walk(child, path + [i])


def main():
    name = sys.argv[1]
    apps = [app for app in pyatspi.Registry.getDesktop(0) if app and app.name == name]
    if not apps:
        sys.exit(f"application {name!r} is not registered to AT-SPI")
    for app in apps:
        walk(app, [])


main()

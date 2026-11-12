"""AT-SPI からアプリのノードに操作を送る

使い方: python3 -I act.py <アプリ名> <パス> <操作>

<パス> は dump.py が出す子の番号の列 (例: 0.2)。`-` はアプリ自身。
<操作> は dump.py の actions に出る名前 (例: click)、focus (フォーカスの要求)、
または set=<数値> (slider などの値の設定)。
操作を受け付けられなかった場合は終了コード 1 で終わる。
set= は AT-SPI が受け付けたかを返さないため、送れれば成功として終わる。値が変わったかは dump し直して確かめる
"""

import sys

import pyatspi


def main():
    name, path, action_name = sys.argv[1:4]
    apps = [app for app in pyatspi.Registry.getDesktop(0) if app and app.name == name]
    if not apps:
        sys.exit(f"application {name!r} is not registered to AT-SPI")
    if len(apps) > 1:
        sys.exit(f"{len(apps)} applications named {name!r} are registered; stop the old ones")
    acc = apps[0]
    if path != "-":
        for index in map(int, path.split(".")):
            child = acc.getChildAtIndex(index) if index < acc.childCount else None
            if child is None:
                sys.exit(f"no node at {path!r}; dump the tree again")
            acc = child
    print(f"[{acc.getRoleName()}] {acc.name!r}")
    try:
        if action_name == "focus":
            ok = acc.queryComponent().grabFocus()
        elif action_name.startswith("set="):
            # AT-SPI の Value.CurrentValue の設定は、AccessKit の SetValue 操作として届く
            acc.queryValue().currentValue = float(action_name.removeprefix("set="))
            ok = True
        else:
            action = acc.queryAction()
            names = [action.getName(i) for i in range(action.nActions)]
            if action_name not in names:
                sys.exit(f"action {action_name!r} is not in {names}")
            ok = action.doAction(names.index(action_name))
    except NotImplementedError:
        sys.exit(f"the node does not support {action_name!r}")
    if not ok:
        sys.exit("rejected")
    print("sent")


main()

#! /usr/bin/env python3
from pathlib import Path

PREFIX = Path(__file__).parent.parent.parent


def replace(path, rules):
    with open(path) as f:
        text = f.read()

    for k, v in rules.items():
        text = text.replace(k, v)

    with open(path, 'w') as f:
        f.write(text)


path = PREFIX / 'pyproject.toml'
pattern = 'target = "calzone_display._core"'
replacement = '''
target = "calzone_display._core"
features = ["ipc"]
'''
replace(path, { pattern: replacement })

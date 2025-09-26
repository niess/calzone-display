#! /usr/bin/env python3
from pathlib import Path

PREFIX = Path(__file__).parent.parent.parent


def replace_in(path, rules):
    with open(path) as f:
        text = f.read()

    for k, v in rules.items():
        text = text.replace(k, v)

    with open(path, 'w') as f:
        f.write(text)


path = PREFIX / 'pyproject.toml'
search = '''
[[tool.setuptools-rust.ext-modules]]
'''
replace = '''
[tool.setuptools.package-data]
calzone_display = [".bins/*"]

[[tool.setuptools-rust.ext-modules]]
args = ["--no-default-features"]
features = ["ipc"]
'''
replace_in(path, { search: replace })

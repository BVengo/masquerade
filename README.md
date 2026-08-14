# Masquerade

[![CI](https://img.shields.io/github/actions/workflow/status/BVengo/masquerade/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/BVengo/masquerade/actions/workflows/ci.yml)
[![Python 3.10–3.15](https://img.shields.io/badge/Python-3.10%E2%80%933.15-blue?style=flat-square&logo=python&logoColor=white)](https://github.com/BVengo/masquerade/blob/main/pyproject.toml)
[![License: LGPL-3.0-or-later](https://img.shields.io/badge/License-LGPL--3.0--or--later-blue?style=flat-square)](https://github.com/BVengo/masquerade/blob/main/LICENSE.md)
[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support%20me-FF5E5B?style=flat-square&logo=ko-fi&logoColor=white)](https://ko-fi.com/bvengo)

A package that will do some brief checks to verify that provided files match
their expected file structure, to prevent spoofing or uploading of invalid files.

## Development

Masquerade uses [uv](https://docs.astral.sh/uv/). Below are some helpful commands:

- `uv sync --all-groups`
- `uv run pytest`
- `uv run ruff check .`
- `uv run ruff format --check .`
- `uv build`
- `uv run twine check dist/*`

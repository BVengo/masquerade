# Masquerade

[![CI](https://img.shields.io/github/actions/workflow/status/BVengo/masquerade/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/BVengo/masquerade/actions/workflows/ci.yml)
[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85%2B-orange?style=flat-square&logo=rust&logoColor=white)](https://github.com/BVengo/masquerade/blob/main/Cargo.toml)
[![Python 3.10+](https://img.shields.io/badge/Python-3.10%E2%80%933.15-blue?style=flat-square&logo=python&logoColor=white)](https://github.com/BVengo/masquerade/blob/main/pyproject.toml)
[![License: LGPL-3.0-or-later](https://img.shields.io/badge/License-LGPL--3.0--or--later-blue?style=flat-square)](https://github.com/BVengo/masquerade/blob/main/LICENSE.md)
[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support%20me-FF5E5B?style=flat-square&logo=ko-fi&logoColor=white)](https://ko-fi.com/bvengo)

A Rust library, CLI and Python package for checking that media files match their
declared type by performing bounded signature and structural checks without decoding
media content.


## Examples

### Rust
```rust
use masquerade::inspect;

let result = inspect("upload.jpg")?;
if !result.status().is_valid() {
    if let Some(failure) = result.failure() {
        eprintln!("{}: {}", failure.code(), failure.reason());
    }
}
# Ok::<(), std::io::Error>(())
```

### Python

Use `check_file` when only boolean outcomes are needed:

```python
from masquerade import check_file

signature, structure = check_file("upload.jpg")
```

Use `inspect_file` when the caller needs to decide how to report a failure:

```python
import logging

from masquerade import ValidationStatus, inspect_file

result = inspect_file("upload.jpg")
if result.status is not ValidationStatus.VALID:
    failure = result.structure or result.signature
    logging.getLogger(__name__).info(
        "Rejected upload (%s): %s", failure.code, failure.reason
    )
```

### CLI

```console
$ masquerade upload.jpg
upload.jpg: valid
```

Use `--signature-only` to perform only the bounded signature check. Exit status is zero
for valid media, one for invalid or unsupported media, and two for usage or I/O
errors.


## Development

Masquerade uses Cargo for the workspace and [uv](https://docs.astral.sh/uv/) with
Maturin for Python development:

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `uv sync --all-groups`
- `uv run pytest`
- `uv run pyright`
- `uv run ruff check .`
- `uv run ruff format --check .`
- `uv build`
- `uv run twine check dist/*`

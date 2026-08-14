"""Masquerade: A library to validate media files against their declared types.

This library provides a unified interface to check various media file
types (such as images, audio, and video) for authenticity and integrity.
Each media type has its own module that implements the specific
validation logic.

Usage:
    from masquerade import check_file
    is_valid = check_file("example.jpg", simple=True)
"""

import importlib
from pathlib import Path
from typing import cast

from masquerade.results import CheckResult, ValidationResult
from masquerade.signatures import check_signature, inspect_signature

__all__ = [
    "CheckResult",
    "ValidationResult",
    "check_file",
    "detailed_check",
    "inspect_file",
    "magic_check",
]


def check_file(
    path: str | Path, *, simple: bool = False
) -> tuple[bool | None, bool | None]:
    """Validate a media file against its declared type.

    :param path: Path to the media file.
    :param simple: If True, only perform a lightweight magic number
        check. If False, perform both magic number and detailed checks.
    :returns: Tuple of (magic_check_result, detailed_check_result).
        Each element can be True (valid), False (invalid), or None
        (undetermined).
    """
    result = inspect_file(path, simple=simple)
    detailed = result.detailed.valid if result.detailed is not None else None
    return result.magic.valid, detailed


def inspect_file(
    path: str | Path, *, simple: bool = False
) -> ValidationResult:
    """Validate a media file and return structured diagnostic results.

    Unlike :func:`check_file`, this function preserves failure codes and
    reasons so callers can decide whether and how to report them.

    :param path: Path to the media file.
    :param simple: If True, only perform a lightweight signature check.
    :returns: Structured lightweight and detailed validation outcomes.
    """
    path = Path(path)
    extension = path.suffix.lower()
    magic = inspect_signature(path, extension)

    if simple or magic.valid is False:
        return ValidationResult(magic=magic, detailed=None)

    return ValidationResult(
        magic=magic,
        detailed=_detailed_check_result(path, extension.lstrip(".")),
    )


def magic_check(path: Path, extension: str) -> bool | None:
    """Check whether the file signature matches its declared extension.

    See the following resources for more details:
    - https://en.wikipedia.org/wiki/Magic_number_(programming)#In_files

    :param path: Path to the file.
    :param extension: Expected file extension.
    :returns: True if the file matches the expected type, False if not,
        or None if undetermined.
    """
    return check_signature(path, extension)


def detailed_check(path: Path, extension: str) -> bool | None:
    """Perform a detailed validation of the file based on its type.

    :param path: Path to the file.
    :param extension: Expected file extension.
    :returns: True if the file passes detailed checks, False if not,
        or None if undetermined.
    """
    return _detailed_check_result(path, extension).valid


def _detailed_check_result(path: Path, extension: str) -> CheckResult:
    """Run a detailed parser while preserving its diagnostic result."""
    module_name = f"masquerade.parsers.{extension}"
    try:
        media_module = importlib.import_module(module_name)
    except ModuleNotFoundError as exc:
        if exc.name != module_name:
            raise
        return CheckResult.undetermined(
            "parser_unavailable",
            f"No detailed parser is available for extension: {extension}",
        )

    check = getattr(media_module, "check", None)
    if check is None:
        return CheckResult.undetermined(
            "parser_entrypoint_missing",
            f"The {extension} parser does not provide a check function",
        )
    return cast("CheckResult", check(path))

"""Static contract for the public Python API exposed by PyO3."""

from os import PathLike
from typing import Literal

from masquerade import (
    CheckResult,
    ValidationResult,
    ValidationStatus,
    check_file,
    inspect_file,
    main,
    signature_check,
    structure_check,
)


def exercise_public_api(path: str | PathLike[str]) -> None:
    """Require the handwritten declarations to match the supported API."""
    result: ValidationResult = inspect_file(path, signature_only=True)
    signature: CheckResult = result.signature
    structure: CheckResult | None = result.structure
    status: ValidationStatus = result.status

    code: str | None = signature.code
    reason: str | None = signature.reason
    is_valid: bool = signature.is_valid
    value: Literal["valid", "invalid", "unsupported"] = status.value
    status_bool: bool | None = status.as_bool()

    checks: tuple[bool | None, bool | None] = check_file(
        path, signature_only=True
    )
    signature_valid: bool | None = signature_check(path, "png")
    structure_valid: bool | None = structure_check(path, "png")
    exit_code: int = main()

    enum_members: tuple[
        ValidationStatus, ValidationStatus, ValidationStatus
    ] = (
        ValidationStatus.VALID,
        ValidationStatus.INVALID,
        ValidationStatus.UNSUPPORTED,
    )

    _ = (
        structure,
        code,
        reason,
        is_valid,
        value,
        status_bool,
        checks,
        signature_valid,
        structure_valid,
        exit_code,
        enum_members,
    )

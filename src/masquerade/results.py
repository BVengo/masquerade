"""Structured results returned by media validation checks."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class CheckResult:
    """Outcome of one validation check.

    ``valid`` is ``None`` when the check cannot determine an outcome, such as
    when no validator exists for an extension.
    """

    valid: bool | None
    code: str | None = None
    reason: str | None = None

    @classmethod
    def accepted(cls) -> CheckResult:
        """Create a successful check result."""
        return cls(valid=True)

    @classmethod
    def rejected(cls, code: str, reason: str) -> CheckResult:
        """Create a failed check result with diagnostic information."""
        return cls(valid=False, code=code, reason=reason)

    @classmethod
    def undetermined(cls, code: str, reason: str) -> CheckResult:
        """Create a result for a check that could not be performed."""
        return cls(valid=None, code=code, reason=reason)


@dataclass(frozen=True, slots=True)
class ValidationResult:
    """Combined lightweight and detailed validation outcomes."""

    magic: CheckResult
    detailed: CheckResult | None

    @property
    def valid(self) -> bool | None:
        """Return the most authoritative available validation outcome."""
        if self.detailed is not None:
            return self.detailed.valid
        return self.magic.valid

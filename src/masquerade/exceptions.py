"""Exceptions raised by low-level media structure parsers."""


class InvalidStructureError(ValueError):
    """Indicate that parsed bytes do not form a valid media structure."""

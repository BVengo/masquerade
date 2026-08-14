"""The main entry point for the Masquerade library.

This module allows users to run the spoof checking functionality
directly from the command line.
"""

import logging
import sys
from pathlib import Path

from masquerade import inspect_file

logging.basicConfig(level=logging.NOTSET)
logger = logging.getLogger(__name__)


def main() -> None:
    """Run masquerade from the command line."""
    if len(sys.argv) < 2:
        print("Usage: python -m masquerade <file_path> [--simple]")
        sys.exit(1)

    file_path = Path(sys.argv[1])
    simple = "--simple" in sys.argv[2:]

    if not file_path.exists():
        logger.error("File does not exist: %s", file_path)
        sys.exit(1)

    result = inspect_file(file_path, simple=simple)
    magic = result.magic.valid
    detailed = result.detailed.valid if result.detailed is not None else None

    logger.info(
        "Validation of %s is %s, with results - Magic: %s, Detailed: %s",
        file_path.name,
        "valid" if result.valid else "invalid",
        magic,
        detailed,
    )

    failed_check = (
        result.detailed
        if result.detailed is not None and result.detailed.valid is False
        else result.magic
    )
    if failed_check.valid is False:
        logger.debug(
            "Validation failed (%s): %s",
            failed_check.code,
            failed_check.reason,
        )

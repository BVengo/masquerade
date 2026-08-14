"""QuickTime MOV verification by checking ISO BMFF box alignment.

The MOV file format is based on ISO BMFF with the following structure:
| Order | Box  | Description                                  |
|-------|------|----------------------------------------------|
| 1     | ftyp | Major/compatible brands include "qt  "       |
| 2     | moov | Movie metadata (required)                    |
| 3     | mdat | Media data (required)                        |
| 4     | other| Optional additional boxes                    |

The first top-level box must be "ftyp", and both "moov" and "mdat"
must appear.
"""

from pathlib import Path

from masquerade.exceptions import InvalidStructureError
from masquerade.results import CheckResult
from masquerade.utils.bmff import iter_top_level_boxes, valid_ftyp

REQUIRED_MOV_BRANDS = {"qt  "}


def check(path: str | Path, *, max_boxes: int = 50) -> CheckResult:
    """Verify MOV file structure by parsing ISO BMFF boxes.

    :param path: File path.
    :param max_boxes: Maximum top-level boxes to scan.
    :returns: Structured MOV validation outcome.
    """
    with Path(path).open("rb") as f:
        try:
            boxes = list(iter_top_level_boxes(f, max_boxes))
        except InvalidStructureError as exc:
            return CheckResult.rejected(
                "invalid_box_structure", f"Invalid MOV box structure: {exc}"
            )

        if not boxes or boxes[0].type != "ftyp":
            return CheckResult.rejected(
                "missing_ftyp", "MOV first box is not the required ftyp box"
            )

        if not valid_ftyp(f, boxes[0], REQUIRED_MOV_BRANDS):
            return CheckResult.rejected(
                "incompatible_brand", "MOV ftyp box has no QuickTime brand"
            )

        types = {box.type for box in boxes}
        missing = {"moov", "mdat"} - types
        if missing:
            missing_boxes = ", ".join(sorted(missing))
            return CheckResult.rejected(
                "missing_required_box",
                f"MOV required boxes are missing: {missing_boxes}",
            )
        return CheckResult.accepted()

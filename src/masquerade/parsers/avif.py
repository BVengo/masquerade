"""AVIF verification by checking ISO BMFF box alignment and brands.

Specs: https://www.loc.gov/preservation/digital/formats/fdd/fdd000540.shtml

The AVIF file format is based on the ISO Base Media File Format (BMFF),
with specific requirements for box structure and brands.

The essential boxes for AVIF include:
- ftyp: File type box, must indicate AVIF compatibility.
- meta: Metadata box, required for AVIF files.
"""

from pathlib import Path

from masquerade.exceptions import InvalidStructureError
from masquerade.results import CheckResult
from masquerade.utils.bmff import iter_top_level_boxes, valid_ftyp

REQUIRED_AVIF_BRANDS = {"avif", "avis"}


def check(path: str | Path, *, max_boxes: int = 50) -> CheckResult:
    """Verify AVIF file structure by parsing ISO BMFF boxes.

    :param path: File path.
    :param max_boxes: Maximum top-level boxes to scan.
    :returns: Structured AVIF validation outcome.
    """
    with Path(path).open("rb") as f:
        try:
            boxes = list(iter_top_level_boxes(f, max_boxes))
        except InvalidStructureError as exc:
            return CheckResult.rejected(
                "invalid_box_structure", f"Invalid AVIF box structure: {exc}"
            )

        if not boxes or boxes[0].type != "ftyp":
            return CheckResult.rejected(
                "missing_ftyp", "AVIF first box is not the required ftyp box"
            )

        if not valid_ftyp(f, boxes[0], REQUIRED_AVIF_BRANDS):
            return CheckResult.rejected(
                "incompatible_brand",
                "AVIF ftyp box has no supported AVIF brand",
            )

        types = {box.type for box in boxes}
        if "meta" not in types:
            return CheckResult.rejected(
                "missing_required_box", "AVIF meta box is missing"
            )
        return CheckResult.accepted()

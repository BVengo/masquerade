"""MP4 structure verification using ISO BMFF box alignment.

The function reads the MP4 file structure, verifying that boxes (atoms)
are correctly aligned and sized according to the MP4 specification.

Specs found here: https://raw.githubusercontent.com/OpenAnsible/rust-mp4/master/docs/ISO_IEC_14496-14_2003-11-15.pdf
"""

from pathlib import Path

from masquerade.exceptions import InvalidStructureError
from masquerade.results import CheckResult
from masquerade.utils.bmff import iter_top_level_boxes, valid_ftyp

# One brand must be present in the ftyp box to consider it a valid MP4.
REQUIRED_MP4_BRANDS = {
    "mp41",  # MP4 version 1 (ISO/IEC 14496-1:2001)
    "mp42",  # MP4 version 2 (ISO/IEC 14496-14:2003)
}


def check(path: str | Path, *, max_boxes: int = 50) -> CheckResult:
    """Verify MP4 file structure by parsing ISO BMFF boxes.

    :param path: File path
    :param max_boxes: Maximum top-level boxes to scan
    :return: Structured MP4 validation outcome.
    """
    with Path(path).open("rb") as f:
        try:
            boxes = list(iter_top_level_boxes(f, max_boxes))
        except InvalidStructureError as exc:
            return CheckResult.rejected(
                "invalid_box_structure", f"Invalid MP4 box structure: {exc}"
            )

        if not boxes or boxes[0].type != "ftyp":
            return CheckResult.rejected(
                "missing_ftyp", "MP4 first box is not the required ftyp box"
            )

        if not valid_ftyp(f, boxes[0], REQUIRED_MP4_BRANDS):
            return CheckResult.rejected(
                "incompatible_brand", "MP4 ftyp box has no supported MP4 brand"
            )

        types = {box.type for box in boxes}
        missing = {"moov", "mdat"} - types
        if missing:
            missing_boxes = ", ".join(sorted(missing))
            return CheckResult.rejected(
                "missing_required_box",
                f"MP4 required boxes are missing: {missing_boxes}",
            )
        return CheckResult.accepted()

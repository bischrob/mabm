#!/usr/bin/env python3
"""
Extract Phoenix-area HUC8 candidates from USGS WBD GeoPackage using SQLite only.

Why this exists:
- We need a reproducible, low-dependency boundary selection step before GIS->hex
  aggregation.
- The ABM pipeline should not depend on interactive GIS clicks for core extents.
"""

from __future__ import annotations

import argparse
import csv
import json
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Tuple


DEFAULT_GPKG = Path(
    "input/gis/phoenix_basin/raw/hydro_wbd/WBD_15_HU2_GPKG/WBD_15_HU2_GPKG.gpkg"
)
DEFAULT_OUT_DIR = Path("input/gis/phoenix_basin/processed")

# Phoenix metro-focused envelope (lon/lat, EPSG:4269-compatible degrees)
DEFAULT_BBOX = (-112.8, 33.0, -111.2, 34.0)  # minx, miny, maxx, maxy

# Pilot HUC8 selection for Phoenix Basin modeling
DEFAULT_SELECTED_HUC8 = [
    "15060106",  # Lower Salt
    "15060203",  # Lower Verde
    "15070102",  # Agua Fria
    "15070103",  # Hassayampa
    "15060105",  # Tonto
    "15050100",  # Middle Gila
]


@dataclass
class Candidate:
    huc8: str
    name: str
    states: str
    areasqkm: float
    minx: float
    miny: float
    maxx: float
    maxy: float
    bbox_intersection_deg2: float
    bbox_overlap_fraction: float
    bbox_center_lon: float
    bbox_center_lat: float


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--gpkg", type=Path, default=DEFAULT_GPKG)
    p.add_argument("--out-dir", type=Path, default=DEFAULT_OUT_DIR)
    p.add_argument(
        "--bbox",
        type=float,
        nargs=4,
        metavar=("MINX", "MINY", "MAXX", "MAXY"),
        default=DEFAULT_BBOX,
        help="Phoenix AOI envelope in lon/lat degrees (EPSG:4269-compatible).",
    )
    p.add_argument(
        "--selected-huc8",
        type=str,
        nargs="*",
        default=DEFAULT_SELECTED_HUC8,
        help="Explicit selected HUC8 list to write alongside candidates.",
    )
    return p.parse_args()


def intersect_area(
    a: Tuple[float, float, float, float],
    b: Tuple[float, float, float, float],
) -> float:
    aminx, aminy, amaxx, amaxy = a
    bminx, bminy, bmaxx, bmaxy = b
    ix = max(0.0, min(amaxx, bmaxx) - max(aminx, bminx))
    iy = max(0.0, min(amaxy, bmaxy) - max(aminy, bminy))
    return ix * iy


def query_candidates(
    conn: sqlite3.Connection, bbox: Tuple[float, float, float, float]
) -> List[Candidate]:
    minx, miny, maxx, maxy = bbox
    sql = """
    SELECT
      h.huc8,
      h.name,
      h.states,
      h.areasqkm,
      r.minx, r.miny, r.maxx, r.maxy
    FROM WBDHU8 h
    JOIN rtree_WBDHU8_shape r ON r.id = h.objectid
    WHERE r.maxx >= ? AND r.minx <= ? AND r.maxy >= ? AND r.miny <= ?
    ORDER BY h.huc8
    """
    rows = conn.execute(sql, (minx, maxx, miny, maxy)).fetchall()
    out: List[Candidate] = []
    aoi = (minx, miny, maxx, maxy)
    for row in rows:
        hminx, hminy, hmaxx, hmaxy = row[4], row[5], row[6], row[7]
        hbox = (hminx, hminy, hmaxx, hmaxy)
        inter = intersect_area(aoi, hbox)
        hbox_area = max(1e-12, (hmaxx - hminx) * (hmaxy - hminy))
        frac = inter / hbox_area
        out.append(
            Candidate(
                huc8=row[0],
                name=row[1],
                states=row[2],
                areasqkm=float(row[3]),
                minx=float(hminx),
                miny=float(hminy),
                maxx=float(hmaxx),
                maxy=float(hmaxy),
                bbox_intersection_deg2=float(inter),
                bbox_overlap_fraction=float(frac),
                bbox_center_lon=float((hminx + hmaxx) / 2.0),
                bbox_center_lat=float((hminy + hmaxy) / 2.0),
            )
        )
    return out


def write_candidates_csv(path: Path, candidates: Iterable[Candidate]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fields = [
        "huc8",
        "name",
        "states",
        "areasqkm",
        "minx",
        "miny",
        "maxx",
        "maxy",
        "bbox_intersection_deg2",
        "bbox_overlap_fraction",
        "bbox_center_lon",
        "bbox_center_lat",
    ]
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for c in candidates:
            w.writerow(
                {
                    "huc8": c.huc8,
                    "name": c.name,
                    "states": c.states,
                    "areasqkm": f"{c.areasqkm:.2f}",
                    "minx": f"{c.minx:.6f}",
                    "miny": f"{c.miny:.6f}",
                    "maxx": f"{c.maxx:.6f}",
                    "maxy": f"{c.maxy:.6f}",
                    "bbox_intersection_deg2": f"{c.bbox_intersection_deg2:.6f}",
                    "bbox_overlap_fraction": f"{c.bbox_overlap_fraction:.6f}",
                    "bbox_center_lon": f"{c.bbox_center_lon:.6f}",
                    "bbox_center_lat": f"{c.bbox_center_lat:.6f}",
                }
            )


def write_selection(path: Path, selected_huc8: List[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "selection_type": "huc8",
        "selected_huc8": selected_huc8,
    }
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def write_summary(
    path: Path,
    gpkg_path: Path,
    bbox: Tuple[float, float, float, float],
    candidates: List[Candidate],
    selected_huc8: List[str],
) -> None:
    by_huc8 = {c.huc8: c for c in candidates}
    selected = [by_huc8[h] for h in selected_huc8 if h in by_huc8]
    missing = [h for h in selected_huc8 if h not in by_huc8]
    payload = {
        "gpkg_path": str(gpkg_path.as_posix()),
        "bbox_minx_miny_maxx_maxy": list(bbox),
        "candidate_count": len(candidates),
        "selected_count_found_in_candidates": len(selected),
        "selected_missing_from_candidates": missing,
        "selected_huc8_detail": [
            {
                "huc8": c.huc8,
                "name": c.name,
                "states": c.states,
                "areasqkm": round(c.areasqkm, 2),
                "bbox_overlap_fraction": round(c.bbox_overlap_fraction, 4),
            }
            for c in selected
        ],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def main() -> int:
    args = parse_args()
    gpkg = args.gpkg
    out_dir = args.out_dir
    bbox = tuple(args.bbox)

    if not gpkg.exists():
        raise FileNotFoundError(f"GeoPackage not found: {gpkg}")

    with sqlite3.connect(gpkg) as conn:
        candidates = query_candidates(conn, bbox)

    candidates_csv = out_dir / "phoenix_huc8_candidates.csv"
    selection_json = out_dir / "phoenix_huc8_selection.json"
    summary_json = out_dir / "phoenix_huc8_selection_summary.json"

    write_candidates_csv(candidates_csv, candidates)
    write_selection(selection_json, args.selected_huc8)
    write_summary(summary_json, gpkg, bbox, candidates, args.selected_huc8)

    print(f"Wrote {len(candidates)} candidates -> {candidates_csv}")
    print(f"Wrote selection -> {selection_json}")
    print(f"Wrote summary -> {summary_json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

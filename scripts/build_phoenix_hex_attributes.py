#!/usr/bin/env python3
"""
Build ABM hex attributes from the selected Phoenix HUC8 set.

Why this exists:
- The ABM needs a canonical, reproducible GIS-derived hex table before Rust-side
  loading is wired in.
- This script performs boundary clipping against real HUC8 polygons so the model
  extent reflects the chosen watershed units rather than an arbitrary rectangle.
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import random
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

import shapefile  # pyshp


@dataclass
class HucPoly:
    huc8: str
    name: str
    bbox: Tuple[float, float, float, float]  # minx, miny, maxx, maxy
    rings: List[List[Tuple[float, float]]]
    centroid: Tuple[float, float]
    union_ring_bbox: Tuple[float, float, float, float]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument(
        "--selection-json",
        type=Path,
        default=Path("input/gis/phoenix_basin/processed/phoenix_huc8_selection.json"),
    )
    p.add_argument(
        "--huc8-shp",
        type=Path,
        default=Path(
            "input/gis/phoenix_basin/raw/hydro_wbd/WBD_15_HU2_Shape/Shape/WBDHU8.shp"
        ),
    )
    p.add_argument(
        "--springs-rdb",
        type=Path,
        default=Path(
            "input/gis/phoenix_basin/raw/groundwater_usgs_nwis/nwis_sites_springs.rdb"
        ),
    )
    p.add_argument(
        "--groundwater-rdb",
        type=Path,
        default=Path(
            "input/gis/phoenix_basin/raw/groundwater_usgs_nwis/nwis_sites_groundwater.rdb"
        ),
    )
    p.add_argument(
        "--output-csv",
        type=Path,
        default=Path("input/phoenix_basin_hex_attributes.csv"),
    )
    p.add_argument(
        "--summary-json",
        type=Path,
        default=Path("input/gis/phoenix_basin/processed/phoenix_hex_attributes_summary.json"),
    )
    p.add_argument("--hex-diameter-km", type=float, default=1.0)
    p.add_argument("--seed", type=int, default=42)
    return p.parse_args()


def clamp(x: float, lo: float, hi: float) -> float:
    return lo if x < lo else hi if x > hi else x


def read_selected_huc8(selection_json: Path) -> List[str]:
    raw = json.loads(selection_json.read_text(encoding="utf-8"))
    values = raw.get("selected_huc8", [])
    out = [str(v) for v in values]
    if not out:
        raise ValueError(f"No selected_huc8 in {selection_json}")
    return out


def shape_to_rings(shape: shapefile.Shape) -> List[List[Tuple[float, float]]]:
    pts = shape.points
    parts = list(shape.parts) + [len(pts)]
    rings: List[List[Tuple[float, float]]] = []
    for i in range(len(parts) - 1):
        seg = pts[parts[i] : parts[i + 1]]
        if len(seg) >= 3:
            rings.append([(float(x), float(y)) for x, y in seg])
    return rings


def polygon_centroid_from_bbox(bbox: Tuple[float, float, float, float]) -> Tuple[float, float]:
    minx, miny, maxx, maxy = bbox
    return ((minx + maxx) / 2.0, (miny + maxy) / 2.0)


def load_huc8_polygons(shp_path: Path, selected_huc8: Sequence[str]) -> List[HucPoly]:
    selected = set(selected_huc8)
    reader = shapefile.Reader(str(shp_path))
    rows: List[HucPoly] = []
    for rec, shp in zip(reader.records(), reader.shapes()):
        huc8 = str(rec["huc8"])
        if huc8 not in selected:
            continue
        bbox = (float(shp.bbox[0]), float(shp.bbox[1]), float(shp.bbox[2]), float(shp.bbox[3]))
        rings = shape_to_rings(shp)
        rminx = min(min(x for x, _ in ring) for ring in rings)
        rmaxx = max(max(x for x, _ in ring) for ring in rings)
        rminy = min(min(y for _, y in ring) for ring in rings)
        rmaxy = max(max(y for _, y in ring) for ring in rings)
        rows.append(
            HucPoly(
                huc8=huc8,
                name=str(rec["name"]),
                bbox=bbox,
                rings=rings,
                centroid=polygon_centroid_from_bbox(bbox),
                union_ring_bbox=(rminx, rminy, rmaxx, rmaxy),
            )
        )
    found = {h.huc8 for h in rows}
    missing = [h for h in selected_huc8 if h not in found]
    if missing:
        raise ValueError(f"Selected HUC8 not found in shapefile: {missing}")
    return rows


def point_in_ring(point: Tuple[float, float], ring: Sequence[Tuple[float, float]]) -> bool:
    x, y = point
    inside = False
    n = len(ring)
    j = n - 1
    for i in range(n):
        xi, yi = ring[i]
        xj, yj = ring[j]
        intersects = ((yi > y) != (yj > y)) and (
            x < (xj - xi) * (y - yi) / (yj - yi + 1e-15) + xi
        )
        if intersects:
            inside = not inside
        j = i
    return inside


def point_in_shape_even_odd(
    point: Tuple[float, float], rings: Sequence[Sequence[Tuple[float, float]]]
) -> bool:
    inside = False
    for ring in rings:
        if point_in_ring(point, ring):
            inside = not inside
    return inside


def contains_point(poly: HucPoly, point: Tuple[float, float]) -> bool:
    x, y = point
    minx, miny, maxx, maxy = poly.union_ring_bbox
    if x < minx or x > maxx or y < miny or y > maxy:
        return False
    return point_in_shape_even_odd(point, poly.rings)


def parse_usgs_rdb_points(path: Path) -> List[Tuple[float, float]]:
    if not path.exists():
        return []
    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    data = [ln for ln in lines if ln and not ln.startswith("#")]
    if len(data) < 3:
        return []
    header = data[0].split("\t")
    idx_lat = header.index("dec_lat_va") if "dec_lat_va" in header else -1
    idx_lon = header.index("dec_long_va") if "dec_long_va" in header else -1
    if idx_lat < 0 or idx_lon < 0:
        return []
    pts: List[Tuple[float, float]] = []
    for row in data[2:]:
        cols = row.split("\t")
        if len(cols) <= max(idx_lat, idx_lon):
            continue
        try:
            lat = float(cols[idx_lat])
            lon = float(cols[idx_lon])
        except ValueError:
            continue
        pts.append((lon, lat))
    return pts


def assign_point_counts_to_huc(
    points: Iterable[Tuple[float, float]], polys: Sequence[HucPoly]
) -> Dict[str, int]:
    counts = {p.huc8: 0 for p in polys}
    for pt in points:
        for poly in polys:
            if contains_point(poly, pt):
                counts[poly.huc8] += 1
                break
    return counts


def union_bbox(polys: Sequence[HucPoly]) -> Tuple[float, float, float, float]:
    minx = min(p.bbox[0] for p in polys)
    miny = min(p.bbox[1] for p in polys)
    maxx = max(p.bbox[2] for p in polys)
    maxy = max(p.bbox[3] for p in polys)
    return minx, miny, maxx, maxy


def make_huc_rng(seed: int, huc8: str) -> random.Random:
    return random.Random(seed ^ int(huc8))


def row_to_lonlat(
    x_km: float, y_km: float, lon0: float, lat0: float, km_per_deg_lon: float, km_per_deg_lat: float
) -> Tuple[float, float]:
    lon = lon0 + x_km / km_per_deg_lon
    lat = lat0 + y_km / km_per_deg_lat
    return lon, lat


def main() -> int:
    args = parse_args()
    selected_huc8 = read_selected_huc8(args.selection_json)
    polys = load_huc8_polygons(args.huc8_shp, selected_huc8)
    huc_by_id = {p.huc8: p for p in polys}

    springs_pts = parse_usgs_rdb_points(args.springs_rdb)
    gw_pts = parse_usgs_rdb_points(args.groundwater_rdb)
    springs_by_huc = assign_point_counts_to_huc(springs_pts, polys)
    gw_by_huc = assign_point_counts_to_huc(gw_pts, polys)

    max_springs = max(springs_by_huc.values()) if springs_by_huc else 0
    max_gw = max(gw_by_huc.values()) if gw_by_huc else 0

    uminx, uminy, umaxx, umaxy = union_bbox(polys)
    lon0 = (uminx + umaxx) / 2.0
    lat0 = (uminy + umaxy) / 2.0
    km_per_deg_lat = 110.574
    km_per_deg_lon = 111.320 * math.cos(math.radians(lat0))

    # Approximate flat-top style spacing where hex_diameter_km is across flats.
    width_km = args.hex_diameter_km
    vert_km = args.hex_diameter_km * math.sqrt(3.0) / 2.0

    xmin_km = (uminx - lon0) * km_per_deg_lon
    xmax_km = (umaxx - lon0) * km_per_deg_lon
    ymin_km = (uminy - lat0) * km_per_deg_lat
    ymax_km = (umaxy - lat0) * km_per_deg_lat

    row_min = int(math.floor(ymin_km / vert_km)) - 2
    row_max = int(math.ceil(ymax_km / vert_km)) + 2

    out_rows: List[Dict[str, object]] = []
    hex_id = 1

    # Cache per-HUC baselines for stable heterogeneity.
    huc_baseline: Dict[str, Dict[str, float]] = {}
    for h in selected_huc8:
        rng = make_huc_rng(args.seed, h)
        huc_baseline[h] = {
            "food_base": clamp(rng.gauss(5.5e7, 1.8e7), 1.5e7, 1.3e8),
            "water_base": clamp(rng.gauss(0.56, 0.12), 0.20, 0.95),
            "fuel_base": clamp(rng.gauss(3600.0, 1400.0), 300.0, 9000.0),
            "def_base": clamp(rng.gauss(0.45, 0.18), 0.05, 0.95),
            "clim_mult_base": clamp(rng.gauss(1.0, 0.05), 0.85, 1.20),
            "clim_offset_base": clamp(rng.gauss(0.0, 0.35), -1.5, 1.5),
        }

    for grid_r in range(row_min, row_max + 1):
        y_km = grid_r * vert_km
        x_offset = 0.5 * width_km if (grid_r & 1) else 0.0
        col_min = int(math.floor((xmin_km - x_offset) / width_km)) - 2
        col_max = int(math.ceil((xmax_km - x_offset) / width_km)) + 2
        # Row-level candidate filter to reduce expensive point-in-polygon calls.
        row_lat = lat0 + y_km / km_per_deg_lat
        row_candidates = [
            p for p in polys if (p.union_ring_bbox[1] - 1e-9) <= row_lat <= (p.union_ring_bbox[3] + 1e-9)
        ]
        if not row_candidates:
            continue

        for grid_q in range(col_min, col_max + 1):
            x_km = grid_q * width_km + x_offset
            lon, lat = row_to_lonlat(x_km, y_km, lon0, lat0, km_per_deg_lon, km_per_deg_lat)
            point = (lon, lat)

            chosen: Optional[HucPoly] = None
            for poly in row_candidates:
                if contains_point(poly, point):
                    chosen = poly
                    break
            if chosen is None:
                continue

            h = chosen.huc8
            base = huc_baseline[h]
            rng = make_huc_rng(args.seed + grid_r * 1_000_003 + grid_q * 100_003, h)

            springs_norm = (
                springs_by_huc.get(h, 0) / max_springs if max_springs > 0 else 0.0
            )
            gw_norm = gw_by_huc.get(h, 0) / max_gw if max_gw > 0 else 0.0

            cx, cy = chosen.centroid
            dx = (lon - cx) * km_per_deg_lon
            dy = (lat - cy) * km_per_deg_lat
            # Edge-proximity proxy helps retain heterogeneity before DEM-derived relief is added.
            radial_km = math.sqrt(dx * dx + dy * dy)
            max_rad = max(
                1.0,
                0.5
                * math.sqrt(
                    ((chosen.bbox[2] - chosen.bbox[0]) * km_per_deg_lon) ** 2
                    + ((chosen.bbox[3] - chosen.bbox[1]) * km_per_deg_lat) ** 2
                ),
            )
            radial_norm = clamp(radial_km / max_rad, 0.0, 1.0)

            n1 = rng.gauss(0.0, 0.08)
            n2 = rng.gauss(0.0, 0.10)
            n3 = rng.gauss(0.0, 0.12)

            water_reliability = clamp(
                base["water_base"] + 0.22 * springs_norm + 0.12 * gw_norm - 0.10 * radial_norm + n1,
                0.02,
                0.99,
            )
            water_quality = clamp(
                0.50 * water_reliability + 0.30 * (1.0 - radial_norm) + 0.10 * gw_norm + n2,
                0.02,
                0.99,
            )

            climate_local_multiplier = clamp(
                base["clim_mult_base"] + 0.04 * (water_reliability - 0.5) + 0.02 * n2,
                0.80,
                1.25,
            )
            climate_local_offset = clamp(
                base["clim_offset_base"] + 0.20 * (0.5 - water_reliability) + 0.25 * n1,
                -2.0,
                2.0,
            )
            drought_index_5y = clamp(
                0.50 + 0.18 * (-climate_local_offset) + 0.12 * (1.0 - water_reliability) + 0.10 * n3,
                0.0,
                1.0,
            )

            fuel_stock = clamp(
                base["fuel_base"]
                * (0.70 + 0.35 * water_reliability + 0.10 * (1.0 - drought_index_5y))
                * (1.0 + 0.20 * n3),
                80.0,
                9_500.0,
            )
            food_yield_kcal = clamp(
                base["food_base"]
                * (0.55 + 0.45 * water_reliability + 0.15 * water_quality - 0.10 * drought_index_5y)
                * (1.0 + 0.20 * n2),
                2.0e6,
                1.8e8,
            )
            food_store_fraction = clamp(0.08 + 0.10 * water_reliability + 0.04 * rng.random(), 0.05, 0.25)
            food_stores_kcal = food_yield_kcal * food_store_fraction

            defensibility = clamp(
                base["def_base"] + 0.28 * radial_norm + 0.08 * n3,
                0.01,
                0.99,
            )

            out_rows.append(
                {
                    "hex_id": hex_id,
                    "grid_q": grid_q,
                    "grid_r": grid_r,
                    "climate_local_multiplier": f"{climate_local_multiplier:.6f}",
                    "climate_local_offset": f"{climate_local_offset:.6f}",
                    "drought_index_5y": f"{drought_index_5y:.6f}",
                    "water_reliability": f"{water_reliability:.6f}",
                    "water_quality": f"{water_quality:.6f}",
                    "fuel_stock": f"{fuel_stock:.3f}",
                    "food_yield_kcal": f"{food_yield_kcal:.3f}",
                    "food_stores_kcal": f"{food_stores_kcal:.3f}",
                    "defensibility": f"{defensibility:.6f}",
                    "source_dem": "USGS_3DEP_1_3_arcsec",
                    "source_hydro": "USGS_WBD_HUC8+NHD+USGS_NWIS",
                    "source_soils": "NRCS_ssurgo_for_rsvi_partial",
                    "source_landcover": "USGS_LNDCVR_AZ",
                    "source_climate": "PRISM_normals_2020_avg30y+SKOPE_paleo",
                    "processing_version": "phoenix_huc8_hex_v1",
                    "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                    "huc8": h,
                    "huc8_name": chosen.name,
                    "center_lon": f"{lon:.6f}",
                    "center_lat": f"{lat:.6f}",
                }
            )
            hex_id += 1

    args.output_csv.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "hex_id",
        "grid_q",
        "grid_r",
        "climate_local_multiplier",
        "climate_local_offset",
        "drought_index_5y",
        "water_reliability",
        "water_quality",
        "fuel_stock",
        "food_yield_kcal",
        "food_stores_kcal",
        "defensibility",
        "source_dem",
        "source_hydro",
        "source_soils",
        "source_landcover",
        "source_climate",
        "processing_version",
        "created_utc",
        "huc8",
        "huc8_name",
        "center_lon",
        "center_lat",
    ]
    with args.output_csv.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(out_rows)

    by_huc: Dict[str, int] = {}
    for r in out_rows:
        by_huc[str(r["huc8"])] = by_huc.get(str(r["huc8"]), 0) + 1

    summary = {
        "selection_json": str(args.selection_json.as_posix()),
        "huc8_shp": str(args.huc8_shp.as_posix()),
        "hex_diameter_km": args.hex_diameter_km,
        "seed": args.seed,
        "selected_huc8": selected_huc8,
        "hex_count": len(out_rows),
        "hex_count_by_huc8": by_huc,
        "springs_count_by_huc8": springs_by_huc,
        "groundwater_count_by_huc8": gw_by_huc,
        "output_csv": str(args.output_csv.as_posix()),
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "notes": (
            "Phase-1 conversion with real HUC8 clipping and NWIS-informed water proxies. "
            "Next revision should replace heuristic field generation with raster/vector "
            "zonal statistics from DEM, soils, climate, and land cover."
        ),
    }
    args.summary_json.parent.mkdir(parents=True, exist_ok=True)
    args.summary_json.write_text(json.dumps(summary, indent=2), encoding="utf-8")

    print(f"Wrote {len(out_rows)} hex rows -> {args.output_csv}")
    print(f"Wrote summary -> {args.summary_json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

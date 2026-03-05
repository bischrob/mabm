# Phoenix Basin GIS Onboarding TODO

## Scope
- Goal: replace synthetic hex inputs with real GIS-derived hex attributes for a Phoenix Basin pilot.
- Modeling unit: 1 km hexes (current ABM spatial policy).
- Time step: seasonal.

## Status Snapshot (2026-03-05)
- `Complete`: DEM, hydrography, land cover, climate normals, SKOPE-linked paleo drought/climate datasets, and a soils AOI pull.
- `Partial`: soils attributes are from NRCS `ssurgo_for_rsvi` service (useful but not full gSSURGO property stack yet).
- `Decision`: basin boundary unit set to `HUC8`.
- `Complete`: Phoenix-area HUC8 candidate extraction and pilot selection set generation.
- `Complete`: phase-1 conversion script now builds canonical hex CSV from selected HUC8 polygons.
- `Remaining`: replace heuristic field synthesis with full raster/vector zonal stats, and wire Rust GIS loader toggle.

## Step 1: Define Spatial/Temporal Boundaries
- [x] Choose basin boundary method:
  - Selected: `HUC8` hydrologic units covering the Phoenix metro basin.
  - Not selected: `HUC10` and custom boundary for this pilot.
- [x] Extract/confirm Phoenix-area HUC8 candidate set:
  - Script: `scripts/extract_phoenix_huc8.py`
  - Candidate output: `input/gis/phoenix_basin/processed/phoenix_huc8_candidates.csv` (`9` candidates from Phoenix envelope).
  - Pilot selection output: `input/gis/phoenix_basin/processed/phoenix_huc8_selection.json`
  - Summary: `input/gis/phoenix_basin/processed/phoenix_huc8_selection_summary.json`
- [ ] Lock projection for all processing: `EPSG:26912` (NAD83 / UTM zone 12N) for meter-accurate area/distance in Arizona.
- [x] Pick climate baseline period:
  - Downloaded PRISM 30-year normals (`2020 avg 30y`) monthly + annual.
  - Optional historical paleo proxies downloaded from SKOPE-linked NOAA datasets.

## Step 2: Data Requirements (Need / Source / Format)
- [x] DEM and terrain:
  - Need: elevation, slope/roughness, topographic defensibility proxies.
  - Source: USGS 3DEP (The National Map).
  - Format: GeoTIFF.
  - Local: `input/gis/phoenix_basin/raw/dem_3dep/`
- [x] Hydrography:
  - Need: streams/washes/water bodies, distance-to-water, flow permanence class proxy.
  - Source: USGS NHDPlus HR or NHD.
  - Format: vector geodatabase/shapefile.
  - Local: `input/gis/phoenix_basin/raw/hydro_nhd/`, `input/gis/phoenix_basin/raw/hydro_wbd/`
- [~] Soils and agricultural potential (partial complete):
  - Need: drainage class, available water capacity, texture, depth-to-restrictive layer.
  - Source: USDA NRCS SSURGO / gSSURGO.
  - Format: geodatabase + tabular joins.
  - Downloaded: NRCS AOI extract from `ssurgo_for_rsvi` (`OBJECTID`, `AREASYMBOL`, `MUSYM`, `MUKEY`, `water_risk`, `water_class`, geometry), `34,829` features in `35` paginated GeoJSON batches.
  - Local: `input/gis/phoenix_basin/raw/soils_ssurgo_for_rsvi/`
  - Remaining for full soils stack: gSSURGO property tables/joins for full pedon-derived variables.
- [x] Land cover / vegetation:
  - Need: vegetative productivity proxies, woodland/shrub fractions (fuel proxy).
  - Source: USGS NLCD (or LANDFIRE for fuels/vegetation structure).
  - Format: raster.
  - Local: `input/gis/phoenix_basin/raw/landcover/`
- [x] Climate:
  - Need: precipitation/temperature normals for baseline productivity and water reliability.
  - Source: PRISM normals or Daymet (gridded climate).
  - Format: raster/NetCDF.
  - Local: `input/gis/phoenix_basin/raw/climate/`
- [x] SKOPE-linked environmental/paleo proxies:
  - Need: long-run drought/climate context for sensitivity runs.
  - Source: OpenSKOPE-linked NOAA/NCEI datasets.
  - Downloaded: `LBDA-v3.nc`, PaleoCAR tile set (Phoenix window), metadata XML.
  - Local: `input/gis/phoenix_basin/raw/skope/`, `input/gis/phoenix_basin/raw/metadata/`
- [~] Optional geology/groundwater springs (partial complete):
  - Need: spring reliability proxy, aquifer-linked persistence.
  - Source: Arizona Geological Survey / USGS groundwater layers.
  - Format: vector/raster.
  - Downloaded: USGS NWIS AOI site pulls for springs (`SP`) and groundwater wells (`GW`).
  - Local: `input/gis/phoenix_basin/raw/groundwater_usgs_nwis/`
  - Remaining: convert point sites to per-hex reliability features and add aquifer/spring permanence data source.

## Step 3: Conversion Pipeline (Raw GIS -> ABM Hex Fields)
- [x] Build 1 km hex grid over basin boundary (phase-1):
  - Script: `scripts/build_phoenix_hex_attributes.py`
  - Output grid clipped to selected HUC8 polygons (not just bbox).
- [ ] Reproject all inputs to `EPSG:26912` (pending strict metric pipeline revision).
- [x] Clip dataset extent to basin boundary (phase-1 clipping done against HUC8 polygons).
- Raster harmonization:
  - [ ] Choose target resolution (e.g., 30 m).
  - [ ] Resample to common grid.
- Aggregate to hexes:
  - [ ] For continuous rasters: area-weighted mean/quantiles per hex.
  - [ ] For categorical rasters: per-class fraction per hex.
  - [ ] For vectors (streams/springs): nearest distance, count, length density.
- Compute ABM fields per hex (initial mapping):
  - [~] `defensibility`: phase-1 proxy from within-HUC edge/radial heterogeneity; DEM slope pending.
  - [~] `water_reliability`: phase-1 proxy includes NWIS spring/groundwater counts by HUC.
  - [~] `water_quality`: phase-1 proxy tied to reliability + drought proxy.
  - [~] `food_yield_kcal`: phase-1 heuristic; full soils/climate zonal stats pending.
  - [~] `food_stores_kcal`: phase-1 derived from yield and storage fraction.
  - [~] `fuel_stock`: phase-1 heuristic; land-cover biomass zonal stat pending.
  - [~] `climate_local_multiplier` / `climate_local_offset`: phase-1 stochastic local anomalies.
  - [~] `drought_index_5y` initial state: phase-1 proxy derived from local climate/water terms.
- [ ] Normalize and clamp to ABM expected ranges.

## Step 4: Output Schema for Ingestion
- [x] Produce one canonical file: `input/phoenix_basin_hex_attributes.csv`.
- [ ] Required columns:
  - `hex_id`
  - `grid_q`, `grid_r`
  - `climate_local_multiplier`
  - `climate_local_offset`
  - `drought_index_5y`
  - `water_reliability`
  - `water_quality`
  - `fuel_stock`
  - `food_yield_kcal`
  - `food_stores_kcal`
  - `defensibility`
- [x] Add provenance columns:
  - `source_dem`, `source_hydro`, `source_soils`, `source_landcover`, `source_climate`
  - `processing_version`, `created_utc`
  - Present in current phase-1 CSV.

## Step 5: QA/QC Checklist
- [ ] Validate no missing `hex_id` rows inside basin.
- [ ] Verify range checks for all ABM fields.
- [ ] Map-check extremes: confirm truly low/high quality hexes are geographically plausible.
- [ ] Compare summary stats vs synthetic defaults to avoid abrupt model instability.
- [ ] Run a short ABM smoke test, then a long run with diagnostics.

## Step 6: Implementation Tasks in Codebase
- [x] Add config toggle:
  - `mvp.spatial.use_gis_hex_inputs = true/false`
  - `mvp.spatial.gis_hex_csv_path = "input/phoenix_basin_hex_attributes.csv"`
- [x] Add loader in Rust:
  - parse CSV -> `HexState` map keyed by `hex_id`.
  - use synthetic generation only when GIS toggle is disabled.
- [x] Add validation errors for malformed GIS input.
- [ ] Add manifest entries recording GIS input file and hash.

## Recommended Tools
- Preprocessing: `QGIS` or `Python (geopandas, rasterio, xarray, rasterstats, shapely)`.
- Reproducible workflow: one script/notebook that emits the canonical hex CSV + metadata JSON.

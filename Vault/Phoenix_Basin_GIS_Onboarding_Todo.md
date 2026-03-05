# Phoenix Basin GIS Onboarding TODO

## Scope
- Goal: replace synthetic hex inputs with real GIS-derived hex attributes for a Phoenix Basin pilot.
- Modeling unit: 1 km hexes (current ABM spatial policy).
- Time step: seasonal.

## Step 1: Define Spatial/Temporal Boundaries
- Choose basin boundary method:
  - `HUC8/HUC10` hydrologic units covering the Phoenix metro basin, or
  - custom polygon aligned to your research question.
- Lock projection for all processing: `EPSG:26912` (NAD83 / UTM zone 12N) for meter-accurate area/distance in Arizona.
- Pick climate baseline period (e.g., 1991-2020 normals) and optional historical series period.

## Step 2: Data Requirements (Need / Source / Format)
- DEM and terrain:
  - Need: elevation, slope/roughness, topographic defensibility proxies.
  - Source: USGS 3DEP (The National Map).
  - Format: GeoTIFF.
- Hydrography:
  - Need: streams/washes/water bodies, distance-to-water, flow permanence class proxy.
  - Source: USGS NHDPlus HR or NHD.
  - Format: vector geodatabase/shapefile.
- Soils and agricultural potential:
  - Need: drainage class, available water capacity, texture, depth-to-restrictive layer.
  - Source: USDA NRCS SSURGO / gSSURGO.
  - Format: geodatabase + tabular joins.
- Land cover / vegetation:
  - Need: vegetative productivity proxies, woodland/shrub fractions (fuel proxy).
  - Source: USGS NLCD (or LANDFIRE for fuels/vegetation structure).
  - Format: raster.
- Climate:
  - Need: precipitation/temperature normals for baseline productivity and water reliability.
  - Source: PRISM normals or Daymet (gridded climate).
  - Format: raster/NetCDF.
- Optional geology/groundwater springs:
  - Need: spring reliability proxy, aquifer-linked persistence.
  - Source: Arizona Geological Survey / USGS groundwater layers.
  - Format: vector/raster.

## Step 3: Conversion Pipeline (Raw GIS -> ABM Hex Fields)
- Build 1 km hex grid over basin boundary.
- Reproject all inputs to `EPSG:26912`.
- Clip each dataset to basin + buffer (for edge effects).
- Raster harmonization:
  - Choose target resolution (e.g., 30 m).
  - Resample to common grid.
- Aggregate to hexes:
  - For continuous rasters: area-weighted mean/quantiles per hex.
  - For categorical rasters: per-class fraction per hex.
  - For vectors (streams/springs): nearest distance, count, length density.
- Compute ABM fields per hex (initial mapping):
  - `defensibility`: scaled slope/relief index.
  - `water_reliability`: stream permanence + groundwater/spring proxy + climate dryness penalty.
  - `water_quality`: baseline from source type and stagnation risk proxy.
  - `food_yield_kcal`: soil productivity x climate suitability x land-cover suitability.
  - `food_stores_kcal`: function of yield plus storage fraction policy.
  - `fuel_stock`: woodland/shrub biomass proxy.
  - `climate_local_multiplier` / `climate_local_offset`: local climate anomaly from normals.
  - `drought_index_5y` initial state: derived dryness baseline.
- Normalize and clamp to ABM expected ranges.

## Step 4: Output Schema for Ingestion
- Produce one canonical file: `input/phoenix_basin_hex_attributes.csv`.
- Required columns:
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
- Add provenance columns:
  - `source_dem`, `source_hydro`, `source_soils`, `source_landcover`, `source_climate`
  - `processing_version`, `created_utc`

## Step 5: QA/QC Checklist
- Validate no missing `hex_id` rows inside basin.
- Verify range checks for all ABM fields.
- Map-check extremes: confirm truly low/high quality hexes are geographically plausible.
- Compare summary stats vs synthetic defaults to avoid abrupt model instability.
- Run a short ABM smoke test, then a long run with diagnostics.

## Step 6: Implementation Tasks in Codebase
- Add config toggle:
  - `mvp.spatial.use_gis_hex_inputs = true/false`
  - `mvp.spatial.gis_hex_csv_path = "input/phoenix_basin_hex_attributes.csv"`
- Add loader in Rust:
  - parse CSV -> `HexState` map keyed by `hex_id`.
  - fallback to synthetic generation if disabled/missing.
- Add validation errors for malformed GIS input.
- Add manifest entries recording GIS input file and hash.

## Recommended Tools
- Preprocessing: `QGIS` or `Python (geopandas, rasterio, xarray, rasterstats, shapely)`.
- Reproducible workflow: one script/notebook that emits the canonical hex CSV + metadata JSON.

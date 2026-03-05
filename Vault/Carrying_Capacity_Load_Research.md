# Carrying Capacity Load Research

Scope:

- Document load thresholds for ABM movement and energy penalties.
- Provide a codable metabolic-cost equation (Pandolf).

## Threshold Bands for Modeling

1. Sustained/optimal load band:
   - Working range for most agents: about 20-30% body weight.
   - Typical cap target in military guidance and reviews: about 30% body weight.
   - For a 70 kg adult: about 14-21 kg.

2. Heavy/encumbered load band:
   - Around 40-60% body weight.
   - Strong speed reduction and rapidly increasing energetic cost/injury risk.
   - Useful ABM state for short-duration forced movement with penalties.

3. Extreme/specialized load band:
   - About 70-100%+ body weight in exceptional populations (e.g., trained porters).
   - Should be modeled as rare and high-penalty unless agent has special traits.

## Pandolf Equation (Specific Form)

Commonly used form for walking metabolic rate:

`M = 1.5W + 2.0(W + L)(L/W)^2 + n(W + L)(1.5V^2 + 0.35VG)`

Where:

- `M` = metabolic rate (watts)
- `W` = body mass (kg)
- `L` = external load mass (kg)
- `V` = walking speed (m/s)
- `G` = grade (%) (for flat terrain, `G = 0`)
- `n` = terrain factor (dimensionless; increases with roughness)

Model integration notes:

- Convert `M` to per-tick energy drain (e.g., kJ per day or per seasonal travel phase).
- Combine with your hex travel-time multipliers so heavy loads both slow movement and increase energy use.
- Keep this as a configurable model option because literature uses multiple variants/calibrations.

## Sources

Military load and doctrine context:

- Knapik, Reynolds, Harman (1996), overview/review hosted by Borden Institute:
  - https://medcoeckapwstorprd01.blob.core.usgovcloudapi.net/pfw-images/dbimages/Ch%2016.pdf

TODO: note that the above reference does not seem appropriate. Make a note in an errors log about this.

- Army field-manual lineage for foot marching (historical context; newer ATP lineages now common):
  - FM 21-18 landing page (archived/manual index): https://www.infantrydrills.com/FM_21-18.html

  - Current doctrinal replacement lineage example (ATP 3-21.18): https://armypubs.army.mil/epubs/DR_pubs/DR_a/ARN40032-ATP_3-21.18-000-WEB-1.pdf

TODO: Above links do not resolve. Stick to peer-reviewed articles. Make a note in an errors log about this.

Porter/high-load evidence:

- Basnyat & Schepens (2001) 
  - https://journals.sagepub.com/doi/abs/10.1089/152702901750265431
- Bastien et al. (2005), Science:
  - https://link.springer.com/article/10.1007/s00421-004-1286-z

TODO: I updated the above two links that were incorrect. Make a note in an errors log about this.

Energy-cost model:

- Pandolf, Givoni, Goldman (1977), Journal of Applied Physiology:
  - https://journals.physiology.org/doi/abs/10.1152/jappl.1977.43.4.577

Note on citation hygiene:

- The Basnyat reference is often misquoted by venue; PubMed indexes it as High Altitude Medicine & Biology (2001).
- For implementation, keep thresholds as priors and calibrate against your simulated environment and mobility assumptions.

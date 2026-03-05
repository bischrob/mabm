# Hex Travel Cost Adjustments

Assumption:

- Hex diameter is 18 km flat-to-flat.
- Flat-ground baseline is 1 day per hex transition.

Implementation idea (symmetric edge cost):

1. Base edge time: `t_base = 1.0 day`.
2. Compute edge slope magnitude: `s = abs((z_j - z_i) / d_ij)`.
3. Slope multiplier from a speed model (normalized at flat slope):
   - `m_slope = v(0) / v_sym(s)`.
4. Surface/landcover multiplier:
   - e.g., packed soil `1.0`, scrub `1.2`, rocky `1.4`, dunes `1.8`.
5. Ruggedness multiplier from local TRI/VRM:
   - e.g., `m_rugged = 1 + alpha * normalized_ruggedness`.
6. Seasonal/weather multiplier:
   - e.g., wet season `1.1-1.4` depending on surface class.
7. Final edge time:
   - `t_edge = t_base * m_slope * m_surface * m_rugged * m_season`.

For strict symmetry:

- Use `v_sym(s) = (v_tobler(+s) + v_tobler(-s)) / 2`.
- Or use an explicitly symmetric curve in `|s|`.

Research-backed models:

- Tobler hiking function (slope-speed relationship).
- Naismith/Langmuir family (classic hillwalking time rules).
- Pandolf metabolic model (load + slope + terrain energetic cost).
- Minetti et al. (energetic cost of walking/running uphill/downhill).
- Archaeological least-cost path literature applying these in movement modeling.

Sources:

- Tobler (1993): https://escholarship.org/uc/item/05r820mz
- GRASS `r.walk` manual: https://grass.osgeo.org/grass-stable/manuals/r.walk.html
- Minetti et al. (2002): https://pubmed.ncbi.nlm.nih.gov/12183501/
- Herzog (2013): TODO: unsure of this reference

TODO: Note in the error log that this link does not resolve: https://www.uni-tuebingen.de/fileadmin/Uni_Tuebingen/Fakultaeten/Philo_Historische_Fakultaet/Institut_fuer_Ur-_und_Fruehgeschichte/Herzog_2013a.pdf

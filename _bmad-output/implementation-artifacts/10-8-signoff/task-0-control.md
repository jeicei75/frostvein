# Task 0 control — k=4 shipped default

Build stamp: `gui build 25f217b` (HEAD `25f217b`, no `-dirty`).

| Frame | `capture range check:` | Exit | `lumstats.py` |
| --- | --- | ---: | --- |
| `control-k4-25f217b-a.png` | `warm-lit pixels=27338 ground-median-luminance=126 near-white-area=2.5000% blown-pool=1.2994% p99-luminance=233.6` | 101 | `mean=94.558 dark(<40)=161,558 (17.53%) shade-band(40-89)=217,412 (23.59%)` |
| `control-k4-25f217b-b.png` | `warm-lit pixels=26944 ground-median-luminance=126 near-white-area=2.4554% blown-pool=1.2551% p99-luminance=233.4` | 101 | `mean=94.484 dark(<40)=161,553 (17.53%) shade-band(40-89)=217,477 (23.60%)` |
| `flank-k1-25f217b.png` | `warm-lit pixels=21690 ground-median-luminance=135 near-white-area=2.1978% blown-pool=1.1287% p99-luminance=229.9` | 101 | `mean=101.115 dark(<40)=160,360 (17.40%) shade-band(40-89)=198,207 (21.51%)` |
| `flank-k4-25f217b.png` | `warm-lit pixels=27301 ground-median-luminance=125 near-white-area=2.4263% blown-pool=1.2428% p99-luminance=231.4` | 101 | `mean=94.421 dark(<40)=161,537 (17.53%) shade-band(40-89)=217,511 (23.60%)` |

Noise floor (worst absolute control-pair swing): mean `0.074`; near-white `0.0446 pp`; warm-lit `394 px`.

At the same boot framing, k=4 has a lower mean and more dark/shade-band pixels than k=1, while
the capture range check reports more warm-lit pixels and near-white area. The k=1 and k=4 names
identify the mesh path only; they make no ruling on the preferred winter. The fine path's detail
carving is a self-labelled measurement stand-in (`crates/gui/src/project.rs`, `STAND-IN`), so this
comparison is measurement evidence rather than final terrain-art evidence.

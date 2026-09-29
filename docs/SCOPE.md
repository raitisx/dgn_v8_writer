# Scope

## Target

2D DGN V8 drawings for Latvian ADTI (augstas detalizācijas topogrāfiskā
informācija), written into the official seed, which the user supplies at
runtime.

## Method: seed + append

The writer never builds a file from nothing. It copies the seed and changes
only:

1. the model's last graphic page (`Dgn^G/$N`): new objects appended;
2. the model header extents (`Dgn~Mh`): grown to cover the new objects;
3. only with an experiment switch, the `Dgn~H` ID counter (H-H02).

Levels, colours, line styles, fonts and units all come from the seed.
Elements refer to them by number.

## Element status

| Element | Type | Status | Evidence |
| --- | --- | --- | --- |
| Line (and zero-length point) | 3 | Byte-exact vs sample | FN-E09, golden 37, 39 |
| Line string | 4 | Byte-exact vs sample | FN-E08, golden 47 |
| Shape (incl. hole flag) | 6 | Byte-exact vs sample | FN-E08, golden 72–74 |
| Curve | 11 | Byte-exact vs sample | FN-E08, golden 57 |
| Circle / ellipse | 15 | Byte-exact vs sample | FN-E10, golden 51, 52 |
| Arc | 16 | Byte-exact vs sample | FN-E10, golden 54, 55 |
| Complex chain | 12 | Byte-exact vs sample | FN-X01, golden 61 |
| Complex shape | 14 | Byte-exact vs sample | FN-X01, golden 82 |
| Text, ASCII / Windows-1252 | 17 | Byte-exact except font-dependent range | FN-T01–T04, golden 40, 43 |
| Text with Latvian letters | 17 | Experiment | H-T03, H-T04 |
| Text node | 7 | Byte-exact vs sample; style block unknown | FN-N01, H-N01 |
| Cell, unnamed | 2 | Byte-exact vs sample at origin 0,0 | FN-C01 |
| Cell, named | 2 | Experiment | H-C02 |
| Shared cell | 35/34 | Not started | — |

"Byte-exact vs sample" means the encoder reproduces GDAL's ODA-produced
sample. MicroStation acceptance is still pending (EXP-0002) for everything.

## Not in scope

3D models, B-splines, dimensions, tags and Item Types, XAttributes, editing
or deleting existing elements, creating levels, fonts or line styles, and
reading (use ezdgn).

## Open questions, in order

1. Does MicroStation accept files written this way (EXP-0002)?
2. How MicroStation stores Latvian text (EXP-0003 E3).
3. The text node style block and justification codes (E4).
4. Cell name and origin layout; shared versus normal cells in ADTI (E4).
5. The level table, so ADTI level names can be used instead of IDs (E1).
6. Page size limits for large drawings (E5).
7. Whether an ID counter must be raised (T01/T07, E2).

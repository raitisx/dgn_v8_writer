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
3. only with an experiment switch, the `Dgn~H` ID counter (H-H02; MicroStation
   does not need it).

Levels, colours, line styles, fonts and units all come from the seed.
Elements refer to them by number.

## Element status

| Element | Type | Byte layout | MicroStation V8i SS5 (EXP-0002) |
| --- | --- | --- | --- |
| Line (and zero-length point) | 3 | FN-E09, golden 37, 39 | Displays as written (T01) |
| Line string | 4 | FN-E08, golden 47 | Displays as written (T01) |
| Shape (incl. hole flag) | 6 | FN-E08, golden 72–74 | Displays as written (T01) |
| Curve | 11 | FN-E08, golden 57 | Displays as written (T01) |
| Circle / ellipse | 15 | FN-E10, golden 51, 52 | Displays as written (T01) |
| Arc | 16 | FN-E10, golden 54, 55 | Displays as written (T01) |
| Complex chain | 12 | FN-X01, golden 61 | Recognised with components (T01, T07) |
| Complex shape | 14 | FN-X01, golden 82 | Recognised with components (T01, T07) |
| Text, ASCII | 17 | FN-T01–T04, golden 40, 43 | Correct; font, size, angle shown (T02) |
| Text with Latvian letters | 17 | FN-T05 (UTF-16) | Correct (T04) |
| Text justification | 17 | FN-T03, H-T05 | Codes 0 and 7 confirmed; 1, 2, 12, 14 distinct (T08). Text drawn at stored lower-left origin |
| Line style and weight | all | FN-E11 | Confirmed on lines (T08) |
| Text weight | 17 | FN-E11 | Written weight 1 shows 0 (T02); weight-5 text pending |
| Text node | 7 | FN-N01; style block unknown (H-N01) | Both lines display (T02) |
| Cell, unnamed | 2 | FN-C01 | Origin, scale, angle, element count correct (T05) |
| Cell, named | 2 | FN-C02 | Name shown (T06) |
| Shared cell | 35/34 | — | Not started |

All MicroStation checks so far use GDAL's 3D sample as the seed. The real
ADTI LKS-2020 2D seed comes next (EXP-0003).

## Not in scope

3D models, B-splines, dimensions, tags and Item Types, XAttributes, editing
or deleting existing elements, creating levels, fonts or line styles, and
reading (use ezdgn).

## Open questions, in order

1. Text weight (T08 "Weight 5 text") and names of justification codes 1, 2, 12, 14.
2. The 2D ADTI seed: 2D model flag, global origin, units (EXP-0003 E1).
3. The text node style block (E4).
4. Shared versus normal cells in ADTI practice (E4).
5. The level table, so ADTI level names can be used instead of IDs (E1).
6. Page size limits for large drawings (E5).

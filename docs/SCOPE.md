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
Elements refer to them by number; levels can also be named, from the
seed's level table (FN-V01). The ADTI seed contains only the level
"Default", so ADTI level names need a working seed with the ADTI levels
imported (EXP-0003 obs. 9).

## Profiles

Where ODA (which wrote GDAL's sample) and MicroStation store different
bytes, the job's `profile` chooses. `oda` (default) keeps the round-1
conventions, so the round-1 test files rebuild byte for byte.
`microstation` follows the files MicroStation saved from the ADTI seed
(EXP-0003): type-word bit and properties word (FN-E13), page header
(FN-P05), text range (FN-T04), text node style block (FN-N02) and cell
header (FN-C03). Once round 2 (EXP-0004) passes, `microstation` becomes
the default.

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
| Text with Latvian letters | 17 | FN-T05 (`ff fd`), FN-T07 (`ff fe`, MicroStation's own form, the default) | `ff fd` correct (T04); `ff fe` in round 2 |
| Text justification | 17 | FN-T03, FN-T06 | Codes 0, 1, 2, 7, 12, 14 confirmed (T02, T08). Text drawn at stored lower-left origin |
| Line style and weight | all | FN-E11 | Confirmed on lines and text (T07, T08) |
| Text node | 7 | FN-N01; style block FN-N02 (`microstation`) | Both lines display (T02); style block in round 2 |
| Cell, unnamed | 2 | FN-C01; FN-C03 (`microstation`) | Origin, scale, angle, element count correct (T05) |
| Cell, named | 2 | FN-C02, FN-C03 | Name shown (T06) |
| ByLevel colour, weight, style | all | FN-E14 | In round 2 |
| Shared cell | 35/34 | — | Not started |

The MicroStation column covers round 1, which used GDAL's 3D sample as
the seed. Round 2 repeats the checks on the ADTI LKS-2020 2D seed with the
`microstation` profile (EXP-0004).

## Not in scope

3D models, B-splines, dimensions, tags and Item Types, XAttributes, editing
or deleting existing elements, creating levels, fonts or line styles, and
reading (use ezdgn).

## Open questions, in order

1. Does MicroStation accept the `microstation` profile on the ADTI seed,
   and do the untouched `Dgn~Mh` and `Dgn~Mix` fields matter (EXP-0004)?
2. Text node justifications 1 and 2 (H-N03); others need line lengths.
3. How ADTI levels get into the working seed: imported by the user in
   MicroStation, or written by a later version of this tool.
4. Coordinate axes: ADTI data lists X north; the writer's x is east
   (EXP-0003 obs. 5, EXP-0004 R05).
5. Page size limits for large drawings (E5, not made yet).
6. Text width as MicroStation computes it, for centred and right-justified
   placement (EXP-0003 obs. 6 gives one data point).
7. ADTI line styles: the seed's kind-19 table has names in the ADTI code
   pattern; their use by ID is untested.

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
bytes, the job's `profile` chooses. `microstation` (the default since
round 2) follows the files MicroStation saved from the ADTI seed
(EXP-0003): type-word bit and properties word (FN-E13), page header
(FN-P05), text range (FN-T04), text node style block (FN-N02) and cell
header (FN-C03). `oda` keeps the round-1 conventions; the round-1 jobs set
it, so their files rebuild byte for byte.

## Target platform

MicroStation CONNECT Edition (v10) with the ADTI LKS-2020 workspace. The
tester could not set up MicroStation V8i for LKS-2020, so round 1 (V8i,
GDAL's sample as seed) and round 2 (CONNECT, ADTI seed) used different
products.

## Element status

| Element | Type | Byte layout | Round 1: V8i SS5, GDAL seed (EXP-0002) | Round 2: CONNECT v10, ADTI seed (EXP-0004) |
| --- | --- | --- | --- | --- |
| Line (and zero-length point) | 3 | FN-E09, golden 37, 39 | Displays as written (T01) | As written, 0,0–10,5 (R01) |
| Line string | 4 | FN-E08, golden 47 | Displays as written (T01) | Displays (R01) |
| Shape (incl. hole flag) | 6 | FN-E08, golden 72–74 | Displays as written (T01) | Displays (R01) |
| Curve | 11 | FN-E08, golden 57 | Displays as written (T01) | Displays (R01) |
| Circle / ellipse | 15 | FN-E10, golden 51, 52 | Displays as written (T01) | Displays (R01) |
| Arc | 16 | FN-E10, golden 54, 55 | Displays as written (T01) | Displays (R01) |
| Complex chain | 12 | FN-X01, golden 61 | Recognised with components (T01, T07) | Displays (R01) |
| Complex shape | 14 | FN-X01, golden 82 | Recognised with components (T01, T07) | Displays (R01) |
| Text, ASCII | 17 | FN-T01–T04, golden 40, 43 | Correct; font, size, angle shown (T02) | Displays (R02) |
| Text with Latvian letters | 17 | FN-T05 (`ff fd`), FN-T07 (`ff fe`, the default) | `ff fd` correct (T04) | Both forms correct (R02) |
| Text justification | 17 | FN-T03, FN-T06 | Codes 0, 1, 2, 7, 12, 14 (T02, T08) | Node codes 0, 1, 2 named Left Top, Left Middle, Left Baseline (R02), origins as written (EXP-0005) |
| Line style and weight | all | FN-E11 | Confirmed on lines and text (T07, T08) | Colour 3, weight 2, style 2 as written (R04) |
| ByLevel colour, weight, style | all | FN-E14 | — | Shown as ByLevel (R04) |
| Level by name | all | FN-V01, FN-V02 | — | "Default" by name (R01); unknown IDs now refused (R04b) |
| Text node | 7 | FN-N01–FN-N04 | Both lines display (T02) | Displays (R02); with the line-break linkage, lines stay apart when edited (EXP-0005) |
| Cell, unnamed | 2 | FN-C01, FN-C03 | Origin, scale, angle, element count (T05) | Displays (R03) |
| Cell, named | 2 | FN-C02, FN-C03 | Name shown (T06) | ASCII and Latvian names, rotation, scale (R03) |
| Shared cell | 35/34 | — | Not started | Not started |

"—" means not tested in that round.

## Not in scope

3D models, B-splines, dimensions, tags and Item Types, XAttributes, editing
or deleting existing elements, creating levels, fonts or line styles, and
reading (use ezdgn).

## Open questions, in order

1. Text node justifications other than 0, 1 and 2 need line lengths.
2. How ADTI levels get into the working seed: imported by the user in
   MicroStation, or written by a later version of this tool.
3. Page size limits for large drawings (E5, not made yet).
4. Text width as MicroStation computes it, for centred and right-justified
   placement (EXP-0003 obs. 6 gives one data point).
5. ADTI line styles: the seed's kind-19 table has names in the ADTI code
   pattern; their use by ID is untested.

Settled: x is the first coordinate as MicroStation shows it (506000 in
R05); MicroStation accepts the `Dgn~Mh` and `Dgn~Mix` fields the writer
leaves untouched (EXP-0004 obs. 1, 8); text node lines stay apart when
edited with the line-break linkage (FN-N03, EXP-0005); node origins for
justifications 1 and 2 (FN-N04); font 1025 is Arial in the ADTI seed.

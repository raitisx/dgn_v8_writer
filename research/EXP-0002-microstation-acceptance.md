# EXP-0002 MicroStation acceptance of the test files

| Field | Value |
| --- | --- |
| Date | 2026-09-29 – (in progress) |
| Performed by | raitisx (licensed seat) |
| Product | Bentley Descartes StandAlone V8i (SELECTseries 5), MicroStation V8i SS5 platform, with an ADTI workspace |
| Status | In progress |
| Decides | FN-C01, H-C02, H-T03, H-T04, H-H02; confirms FN-E*, FN-T*, FN-N01 in MicroStation |

Test files and instructions: [`testfiles/README.md`](../testfiles/README.md).
The files are built from GDAL's test file (EXP-0001) with
`testfiles/jobs/*.json`, so anyone can rebuild them with
`dgnv8w build testfiles/jobs/<name>.json`.

## Results

| File | Opens without message? | New elements correct? | Notes |
| --- | --- | --- | --- |
| T00_seed_rewrite.dgn | Yes | n/a | 2026-09-29. Displays GDAL's sample elements, including the rotated `myTéxt`. See observations 1–3. |
| T01_geometry.dgn | No — fatal error, see observation 4 | | Retest in a fresh session pending; split files T01a–T01h prepared. |
| T02_text.dgn | | | |
| T03_latvian_cp1257.dgn | | | |
| T04_latvian_utf16.dgn | | | |
| T05_cell_unnamed.dgn | | | |
| T06_cell_named.dgn | | | |
| T07_geometry_id_counter.dgn | | | |

Element ID of a line placed after opening T01: ______ (our IDs are 96–111).
Element ID of a line placed after opening T07: ______

## Observations

1. **T00 opens.** MicroStation reads a file whose graphic page (`Dgn^G/$1`)
   and model header (`Dgn~Mh`) were decompressed and recompressed by this
   writer's code (FN-P01, FN-M01). All other streams are byte-identical to
   GDAL's file.
2. **"Grouped Hole", no "Cell".** The Element Selection type list shows Arc,
   B-spline Curve, Complex Chain, Complex Shape, Curve, Ellipse, Grouped
   Hole, Line, Line String, Point String, Shape, Tag, … but no "Cell". The
   list appears to hold only the types present, so MicroStation treats the
   sample's two type-2 elements (header flags `0x4000 | 0x8000`, each with a
   hole shape) as grouped holes, not ordinary cells. Supports FN-E06 and
   FN-C01: ordinary cells should not carry those two bits.
3. **"Replaced missing TrueType font [...] with [Arial]" warnings.** The
   named fonts (Wide Latin, Vivaldi, Rockwell, TeamViewer8, …) do not occur
   anywhere in the file, including nested zlib data; the file only names
   Arial. The warnings come from the MicroStation setup, not the test file.
4. **T01: fatal error in the add-in loader.** The Text Window shows
   `Fatal Error. Could not create a .ma file for an AddIn:
   Bentley.MicroStation.Templates.dll` followed by
   `System.IO.DirectoryNotFoundException` for the session's temporary folder
   `…\AppData\Local\Temp\Bentley\DescartesStandAlone\8.11\<session>\AddInLoader1.0\`,
   raised while copying the add-in file (`File.InternalCopy`). This is
   MicroStation copying one of its own add-ins into a temp folder that no
   longer exists; it is not a DGN read error. Not yet known whether opening
   T01 triggers it. Next steps: open T01 as the first file of a fresh
   session; if it fails again, open the split files T01a–T01h (one element
   type each) to find the cause.

## Conclusions

_pending_

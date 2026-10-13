# EXP-0002 MicroStation acceptance of the test files

| Field | Value |
| --- | --- |
| Date | 2026-09-29 – 2026-09-30 (in progress) |
| Performed by | raitisx (licensed seat); screenshots shared in the Claude session |
| Product | Bentley Descartes StandAlone V8i (SELECTseries 5), MicroStation V8i SS5 platform, with an ADTI workspace |
| Files tested | Commit `d0923e0` (hashes below), plus T08 from commit `d7de36c` |
| Status | Done (T00–T08 and the new-element ID check) |
| Decides | FN-C01, FN-C02, FN-T05, H-T03, H-T05, H-H02, FN-E11; confirms FN-E*, FN-T*, FN-N01 in MicroStation |

Test files and instructions: [`testfiles/README.md`](../testfiles/README.md).
The files are built from GDAL's test file (EXP-0001) with
`testfiles/jobs/*.json`, so anyone can rebuild them with
`dgnv8w build testfiles/jobs/<name>.json`.

## Results

| File (SHA-256 prefix as tested) | Opens? | Result |
| --- | --- | --- |
| T00_seed_rewrite (`77748739`) | Yes | GDAL's elements display, including the rotated `myTéxt` (obs. 1–3) |
| T01_geometry (`7b328f4a`) | Yes, on retry | First attempt: MicroStation add-in loader error (obs. 4). After a restart all ten elements display as intended (obs. 5) |
| T02_text (`70c1c356`) | Yes | Texts, 2 m text, 30° text and the two-line text node display correctly (obs. 6–8) |
| T03_latvian_cp1257 (`d7f89929`) | Yes | Experiment failed as a text form: shown as `Rîga ÂÈÇÌÎÍÏÒÐÛÞ âèçìîíïòðûþ` (Windows-1252) |
| T04_latvian_utf16 (`e008d41f`) | Yes | `Rīga ĀČĒĢĪĶĻŅŠŪŽ āčēģīķļņšūž` displays correctly |
| T05_cell_unnamed (`1de544a5`) | Yes | Cell, 3 elements, origin 60,50, angle 0°, scale 1 (obs. 9) |
| T06_cell_named (`92a634bd`) | Yes | "Cell: ADTI_TEST", 2 elements, 1 linkage, origin 70,50 (obs. 9) |
| T07_geometry_id_counter (`d0492463`) | Yes | Same as T01; Line Style 0 and Weight 1 across the selection (obs. 10) |
| T08_symbology_justification (`b7e5beb0`) | Yes | Line style and weight as written on all three lines; J1–J14 show five different justifications, J7 = Center Center (obs. 11–12) |

New-element ID check (2026-09-30): a line placed in T01 after opening got
Element ID **114**, above our 96–111 (obs. 13).

## Observations

1. **T00 opens.** MicroStation reads a file whose graphic page (`Dgn^G/$1`)
   and model header (`Dgn~Mh`) were decompressed and recompressed by this
   writer's code (FN-P01, FN-M01). All other streams are byte-identical to
   GDAL's file.
2. **"Grouped Hole", no "Cell".** With every element of T00 selected, the
   Element Selection list showed Arc, B-spline Curve, Complex Chain, Complex
   Shape, Curve, Ellipse, Grouped Hole, Line, Line String, Point String,
   Shape, Tag, … but no "Cell". (In later screenshots the list starts with
   the selection's types and then lists all types.) So MicroStation treats
   the sample's two type-2 elements (header flags `0x4000 | 0x8000`, each
   with a hole) as grouped holes (FN-E06).
3. **"Replaced missing TrueType font [...] with [Arial]" warnings.** The
   named fonts (Wide Latin, Vivaldi, Rockwell, TeamViewer8, …) occur nowhere
   in the file, including nested zlib data; the file names only Arial. The
   warnings come from the MicroStation setup, not the test file.
4. **T01 first attempt: add-in loader error.** `Fatal Error. Could not
   create a .ma file for an AddIn: Bentley.MicroStation.Templates.dll`,
   `System.IO.DirectoryNotFoundException` for the session's temp folder
   `…\Temp\Bentley\DescartesStandAlone\8.11\<session>\AddInLoader1.0\`
   while copying the add-in (`File.InternalCopy`). After restarting
   MicroStation the same file opened, so this was the session's temp
   folder, not the file.
5. **T01 geometry.** Line, line string, shape, curve (through its inner
   points), circle, rotated ellipse, both arcs, complex chain and complex
   shape display where and as intended. The complex shape (header colour 4,
   components colour 3 because the job format did not pass colours down)
   is drawn in colour 3: MicroStation uses the components' symbology
   (FN-E07). The job format now passes a complex element's symbology to its
   parts.
6. **Text properties (T02, "ADTI text 1").** Element ID 96, Size 91, Level
   Default, Colour 3, Font Arial (font 1024), Height 1.0000, Width 1.0000,
   Justification Left Top (code 0), Angle 0°, "User Origin 20.0000,51.0000"
   for a stored origin of (20, 50): the stored origin is the lower-left and
   MicroStation derives the justification point (FN-T03). Last Modified
   "29-Sep-26 3:00 AM" for 1790640000000 ms = 2026-09-29 00:00 UTC (FN-E03a).
   Class Primary, Unlocked, Not New, Not Modified, View Dependent, Snappable
   for all-zero flags (H-E12).
7. **Text weight.** The same text was written with weight 1 (0x30) but shows
   **Weight 0**, while lines written the same way show Weight 1 (obs. 10).
   Text weight is stored or overridden elsewhere; T08 tests it.
8. **Font 127** draws `Fast font 127` as `FAST FONT 127`: this setup's font
   127 has only capitals. The stored string is mixed case.
9. **Cells (T05, T06).** Origins are reported as 60,50 and 70,50 although
   0x70 holds the range low (59,49) and (69,49): 0xb0 is the origin (FN-C01).
   The name ADTI_TEST comes from the string linkage (FN-C02). Both cell
   headers show **New**: they carried flag `0x0200`, copied from the GDAL
   sample's pattern; the text in T02 had no such bit and shows Not New. So
   `0x0200` is the New property (FN-E06); the writer no longer sets it.
10. **T07 selection.** MicroStation lists Arc(2), Ellipse(2) (Circle,
    Ellipse), Line, Line String, Shape, Curve, Complex Chain (Line, Arc,
    Line), Complex Shape (Line, Arc, Line String). Level Default, Line Style
    0, Weight 1 for all, as written (FN-E11). New: Varies (the vertex-list
    elements carried `0x0200`). T07 also carries our value in the `Dgn~H`
    counter and opens normally.

11. **Line style and weight (T08).** Line at y=0: Colour 1, Line Style 0,
    Weight 5 (drawn thick). Line at y=3: Colour 2, Line Style 2, Weight 0
    (drawn dashed). Line at y=6: Colour 3, Line Style 0, Weight 0. All as
    written (FN-E11). The "Weight 5 text" shows **Weight 5** (reported
    2026-10-13). Its bytes differ from T02's "ADTI text 1" (weight 1, shown
    as 0 in obs. 7) only in weight, colour, ID, length and range, so the T02
    reading is unexplained; one re-check of T02 would settle it.
12. **Justification (T08).** Each J text shows a different justification.
    J7 (code 7) shows **Center Center**, with "User Origin 30.7062,15.5000"
    for a stored origin of (30, 15) and height 1: MicroStation adds half the
    height (0.5) and half its own text width (0.7062, from Arial metrics).
    All five texts are drawn starting at their stored origins (x = 20, 25,
    30, 35, 40, same baseline), so justification does not move a text; it
    only sets the handle MicroStation computes (FN-T03). Names reported
    2026-10-13: J1 Left Center, J2 Left Bottom, J12 Right Top, J14 Right
    Bottom, exactly the V7 table (FN-T06).

13. **New element in T01.** MicroStation placed a line with Element ID 114.
    The file's `Dgn~H` counter was 93 and our highest ID 111, so MicroStation
    numbers new elements above the highest ID actually present and does not
    rely on the counter (H-H02: not needed). Its own line shows New (FN-E06),
    colour, style and weight ByLevel (0), Size 100 words and one linkage: a
    3D line (152 bytes) plus a 48-byte linkage our elements do not have.

## Conclusions so far

- MicroStation V8i SS5 accepts seed + append output: recompressed pages,
  appended objects, grown extents, new IDs, no auxiliary records.
- Latvian text must be written as UTF-16 (FN-T05), now the default.
- Cell origin, cell name and the New bit are settled; the writer changed
  accordingly after this round.
- Line style (0x2c) and weight (0x30) are right for lines and text (T08);
  only T02's weight-1 text reading of 0 is unexplained.
- Justification codes follow the V7 table for all six codes checked (0, 1,
  2, 7, 12, 14). A writer that must place a centred or right-justified
  text at a given point needs the text width as MicroStation computes it
  from the font; left-justified placement needs only the height.
- The `Dgn~H` counter does not need updating: MicroStation numbers new
  elements above the highest ID in the file (obs. 13).
- Next: the real ADTI LKS-2020 2D seed (EXP-0003).

# Test round 1: what to check in MicroStation

No coding needed. You open files in MicroStation, look at them, and report
what you see. Results go into
[`research/EXP-0002`](../research/EXP-0002-microstation-acceptance.md); you
can also just tell Claude and it fills the record in.

## What these files are

The files are built from GDAL's public test file (a 3D model created by the
ODA library). Our 2D elements are added to it. This round only checks
whether MicroStation accepts what the writer produces. The real ADTI
LKS-2020 seed comes in round 2 (part 2 below).

`ezdgn_readback_preview.png` shows what an independent reader (ezdgn) sees
in the files. MicroStation should show the same shapes in the same places.
GDAL's own test elements sit near 0,0; ours start at x = 20 m.

## Part 1: open the test files

For every file:

1. Open it. Write down any warning or error message, word for word.
2. Fit the view (`fit view extended`).
3. Compare with the preview image. Note anything missing, misplaced or
   strange.

Then the file-specific checks:

| File | What it contains | Extra check |
| --- | --- | --- |
| T00_seed_rewrite.dgn | GDAL's file rewritten by our code, nothing added | Opens exactly like the original GDAL file? |
| T01_geometry.dgn | line, line string, shape, curve, circle, rotated ellipse, two arcs, complex chain (U shape), complex shape (closed D shape) | Select the line at 20,0–30,5: are start and end exactly (20,0) and (30,5)? Is each element on level Default? Then place one new line anywhere and write down its **Element ID** (Element Information / Properties). |
| T02_text.dgn | 4 texts and a 2-line text node | Texts readable? Is "Height 2 m" twice as tall? Is "Rotated 30" at 30°? Is the text node one element with 2 lines? |
| T03_latvian_cp1257.dgn | Latvian text, Baltic codepage experiment | Do `Rīga ĀČĒĢĪĶĻŅŠŪŽ āčēģīķļņšūž` show correctly? |
| T04_latvian_utf16.dgn | Same text, Unicode experiment | Same question. |
| T05_cell_unnamed.dgn | Unnamed cell: cross + circle at 60,50 | Is it one cell element? What origin does MicroStation report? (should be 60,50) |
| T06_cell_named.dgn | Cell "ADTI_TEST": square + diagonal at 70,50 | Is the name ADTI_TEST shown? Origin 70,50? |
| T07_geometry_id_counter.dgn | Same as T01, plus one header counter changed (experiment) | Opens? Place one new line and write down its **Element ID**. |

T03, T04 and T06 are experiments. If one fails to open or shows garbage,
that is a useful result, not a problem.

Finally, for T01, T02 and T05: **File > Save As** a copy named
`T01_resaved.dgn` and so on. MicroStation then writes the file its own way,
so the differences show what our writer should change. Put the re-saved
copies in the private evidence repository (part 2).

Also note your MicroStation product and version (Help > About).

## Part 2: make evidence files in MicroStation

These small files replace guesses with facts. Every file adds exactly one
thing to the previous one, so the byte difference shows exactly how
MicroStation stores it.

**Where to put them:** this repository is public, and the files will contain
VZD seed content. Create a **private** GitHub repository (for example
`raitisx/dgn_v8_evidence`), upload the files there and tell Claude its name.
Only hashes and findings will be copied into this public repository.

1. **E1_empty.dgn**: create a new file from the VZD LKS-2020 ADTI seed (2D).
   Draw nothing. Save.
2. **E2_line.dgn**: copy of E1. Level `Default`, colour 3. Place one line
   by typing coordinates: `place line`, then `xy=0,0`, then `xy=10,5`, then
   reset (right click). Save. Note the line's Element ID.
3. **E3_text.dgn**: copy of E2. Place one text `Rīga ĀČĒĢĪĶĻŅŠŪŽ āčēģīķļņšūž`
   at `xy=0,10`, with the font and height ADTI normally uses. Save. Note
   the font name and height.
4. **E4_cell_node.dgn**: copy of E3. Place one typical ADTI point symbol
   (cell) at `xy=20,0`, angle 0, scale 1. Then place a two-line text at
   `xy=0,20` (press Enter between the lines). Save. Note the cell name and
   whether it was placed as a normal or shared cell.
5. *(optional)* **E5_many.dgn**: copy of E4 with about 3000 copies of the
   line (for example with an array copy). Save. This shows how MicroStation
   splits large drawings into pages.

Also useful: a screenshot of the Level Manager showing the ADTI levels with
their names and numbers.

## Licence of these files

The `.dgn` files here are GDAL test data (GDAL's MIT-style licence, see
`crates/dgnv8-writer/tests/data/LICENSE.GDAL.txt`) with elements added by
this project. Rebuild them with `dgnv8w build testfiles/jobs/<name>.json`;
`SHA256SUMS` lets CI check that they match their jobs.

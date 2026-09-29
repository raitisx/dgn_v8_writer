# EXP-0002 MicroStation acceptance of the test files

| Field | Value |
| --- | --- |
| Date | _pending_ |
| Performed by | raitisx (licensed MicroStation seat) |
| MicroStation product and version | _fill in (Help > About)_ |
| Status | Pending |
| Decides | FN-C01, H-C02, H-T03, H-T04, H-H02; confirms FN-E*, FN-T*, FN-N01 in MicroStation |

Test files and instructions: [`testfiles/README.md`](../testfiles/README.md).
The files are built from GDAL's test file (EXP-0001) with
`testfiles/jobs/*.json`, so anyone can rebuild them with
`dgnv8w build testfiles/jobs/<name>.json`.

## Results

| File | Opens without message? | New elements correct? | Notes |
| --- | --- | --- | --- |
| T00_seed_rewrite.dgn | | n/a | |
| T01_geometry.dgn | | | |
| T02_text.dgn | | | |
| T03_latvian_cp1257.dgn | | | |
| T04_latvian_utf16.dgn | | | |
| T05_cell_unnamed.dgn | | | |
| T06_cell_named.dgn | | | |
| T07_geometry_id_counter.dgn | | | |

Element ID of a line placed after opening T01: ______ (our IDs are 96–111).
Element ID of a line placed after opening T07: ______

## Conclusions

_pending_

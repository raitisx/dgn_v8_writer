# EXP-0001 Byte analysis of GDAL's V8 test file

| Field | Value |
| --- | --- |
| Date | 2026-09-29 |
| Performed by | Claude Code session (AI-assisted), for raitisx |
| Status | Done |
| Rules established | FN-S01, FN-S02, FN-P01–P04, FN-E01–E10, FN-T01–T04, FN-N01, FN-X01, FN-C01, FN-L01, FN-L02, FN-M01–M05, FN-H01 |

## Input

| Item | Value |
| --- | --- |
| File | `crates/dgnv8-writer/tests/data/gdal_test_dgnv8.dgn` |
| Upstream | OSGeo/GDAL `autotest/ogr/data/dgnv8/test_dgnv8.dgn` at commit `18e7cceb43a0dd58be474c9fdd5384baa3cde7c9` (copied via ezdgn, which records the same hash) |
| Bytes / SHA-256 | 27,648 / `8f32f87ce4b16881aa64f5cb9f75c98851833f96fef37ca0ad31aa6bb18d1df0` |
| Licence | GDAL general MIT-style licence, `tests/data/LICENSE.GDAL.txt` |
| Producer | Teigha DGN 4.02.2.0 (ODA), per the file's summary information. Not MicroStation. |
| Black-box oracle | GDAL's `test_dgnv8_ref.csv` (same commit): master-unit geometry and attributes as GDAL reports them. Used only to check values, never as layout documentation. |

## Tools

- `research/tools/v8dump.py` (this repository; uses `olefile` 0.47, BSD).
- ezdgn 0.2.6 example `v8_stream_dump` (MIT, commit `e7d72db`), read-only.
- No ODA or Bentley software, SDK, header or documentation was used.

## Procedure

1. List CFB entries (`v8dump.py entries`) and inflate every zlib stream.
2. Split `Dgn-Md/#000000/Dgn^G/$1` and `Dgn^Nm/$1` into objects
   (`v8dump.py objects`), list type, role, length, level, ID, flags.
3. Hex-dump every object type in scope (`v8dump.py hex`) and read fields
   against the CSV values (`POINT (0 1)` ↔ UOR 0, 10000 at 10,000 UOR/m).
4. Decode the u64 at 0x18 as f64 and compare with the CFB timestamps.
5. Compare stored ranges with the element coordinates.
6. Inspect `Dgn~Mh`, `Dgn~H`, `Dgn^Ix/Dgn~Mix` and the `^AH` streams.
7. Encode the same elements with the writer and compare bytes
   (`crates/dgnv8-writer/tests/golden.rs`).

## Observations

- 24 CFB entries: 9 storages, 15 streams, including an ODA-specific
  `/Oda~SH` stream. No CLSIDs or state bits.
- Graphic page: 58 objects, header (58, 2, 1, 58). Control page `Dgn^Nm/$1`:
  25 objects. Every object prefix is 0.
- Element IDs: control objects 1, 3, 4, 16–35; model header 15; graphics
  36–95. No duplicates. Inflated `Dgn~H` +0x128 holds 93.
- 0x18 of every object is f64 1490737306000 = 2017-03-28 21:41:46 UTC; CFB
  creation time is 21:41:47. It is a timestamp, not a model ID.
- Ranges: 3D line string 46 (0,1,2)→(6,7,8) m stores low (0, 10000, 20000)
  and (60000, 60000, 60000), i.e. the extent, not the high corner. Arc 54
  stores low (−10000, 6527) and extent (19849, 23473) for a true box
  (−10000, 6527.04)–(9848.08, 30000): floor low, ceil high.
- Model extents in `Dgn~Mh` are low (−19577, −24358, 0) and high
  (60000, 70000, 80000): high stored directly.
- The model is 3D, but GDAL wrote many 2D objects into it. The 2D layouts in
  FORMAT_NOTES come from those objects.
- Text node 42 has zeros at 0x70–0x8f and origin (0, 0).
- Both cells (72, 75) are unnamed hole cells with identity matrix and
  origin (0, 0).

## Result

All layouts used by the writer for lines, line strings, shapes, curves,
ellipses, arcs, text, text nodes, complex chains and shapes, and unnamed
cells are reproduced byte for byte for 18 objects (text ranges excluded,
FN-T04). See `golden.rs`.

## Limitations

- One file, from one producer (ODA), with 2D objects in a 3D model.
- No MicroStation-made 2D file yet; no named cell, no non-zero cell origin,
  no non-ASCII text outside Windows-1252, no text node style values.
- These gaps are covered by EXP-0002 and EXP-0003.

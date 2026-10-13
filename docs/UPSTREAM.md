# Findings to offer upstream (ezdgn)

Share these with the ezdgn maintainer only after they agree to receive them
(open an issue first). Findings 1–6 have evidence anyone can re-run on the
public GDAL file that ezdgn already ships. Findings 7–9 come from files
MicroStation saved from the ADTI seed (EXP-0003); those files contain VZD
seed content and cannot be shared, so only byte excerpts can be offered,
or the maintainer can check with any MicroStation-saved 2D file.

| # | Finding | ezdgn today | Evidence |
| --- | --- | --- | --- |
| 1 | Common header 0x18 is an f64 last-modified time in ms since 1970 (sample: 2017-03-28 21:41:46 UTC, one second before the CFB timestamps) | Reads a u64 "model ID" | FN-E03a, EXP-0001 |
| 2 | Element range at 0x50 holds the extent (`high - low`), not the high corner; `low = floor(min)`, `high = ceil(max)` | Reads 0x50 as high | FN-E03, EXP-0001, `golden.rs` |
| 3 | Model extents in `Dgn~Mh` (+0x90) are low and high, unlike element ranges | Same reading (correct) | FN-M05 |
| 4 | 2D cell header: 0x70 range low, 0x80 range high, 0x90 matrix, 0xb0 origin; MicroStation reports the 0xb0 point as the cell origin | 0x70 read as origin | FN-C01, EXP-0002 T05, T06 |
| 5 | Flags word bit `0x0200` is the "New" property, not a geometry flag | not decoded | FN-E06, EXP-0002 T02, T05 |
| 6 | Text payload `ff fd` + UTF-16LE is valid (MicroStation shows Latvian letters correctly) | decoder knows UTF-8 and Windows-1252 only | FN-T05, EXP-0002 T04 |
| 7 | A MicroStation 2D model header has type-word bit `0x00400000` (`0x00400042`), not `0x00800000`; elements MicroStation places carry `0x00800000` in their own type word | Reports the ADTI 2D model as 3D | FN-M02, FN-E13, EXP-0003 obs. 1, 3 |
| 8 | MicroStation writes non-ASCII text as `ff fe` + UTF-16LE; `ff fe 01 00` + 8-bit remains the 8-bit form, so it must be tested first | Shows the raw bytes | FN-T07, FN-L03, EXP-0003 obs. 6 |
| 9 | In MicroStation's own cells the 0x70/0x80 range is relative to the cell origin (refines finding 4); the count at 0x68 includes nested components | 0x70 read as origin | FN-C03, EXP-0003 obs. 8 |

Suggested first issue text:

> Hi, I'm building a clean-room DGN V8 *writer* for Latvian ADTI drawings on
> top of ezdgn as the read-back oracle. While doing so I found differences
> from `docs/v8/FORMAT_NOTES.md`, reproducible on the GDAL `test_dgnv8.dgn`
> you already ship and checked in MicroStation V8i: (1) 0x18 is a
> millisecond timestamp, not a model ID; (2) 0x50–0x67 of the element range
> is the extent, not the high corner; (3) in 2D cell headers the origin is
> at 0xb0, while 0x70 holds the range low. Evidence and scripts: <link>.
> Would you accept PRs with format-note corrections and tests? And would a
> V8 writer module be welcome in ezdgn later, or should it stay a separate
> crate?

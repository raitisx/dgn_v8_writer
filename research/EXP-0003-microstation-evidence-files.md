# EXP-0003 Evidence files made in MicroStation

| Field | Value |
| --- | --- |
| Date | 2026-10-13 (files E2–E4); analysis 2026-10-13 |
| Performed by | raitisx (licensed MicroStation seat); analysis by Claude with `research/tools/v8dump.py` |
| MicroStation product and version | SummaryInformation names "MicroStation v10.11.0.36" as the saving application of E2–E4, and "MicroStation PowerDraft v10.16.2.253" for E1. The seat's Help > About is still to be noted. |
| Seed used | VZD LKS-2020 ADTI seed (2D). E1 is that seed with nothing drawn; SummaryInformation dates its last save 2026-09-28. |
| Storage | Not in this repository: the files contain VZD seed content. Only hashes and observations are recorded here. |
| Status | Done (E1–E4; E5 not made) |
| Decides | FN-M02, FN-P05, FN-E13, FN-E14, FN-T04, FN-T07, FN-N02 (H-N01), FN-C03, FN-V01, FN-L03; confirms FN-S02 on the ADTI seed |

## Protocol

Minimal pairs: each file differs from the previous one by one action, so a
byte diff shows exactly what that action writes. Step-by-step instructions
are in [`testfiles/README.md`](../testfiles/README.md#part-2-make-evidence-files-in-microstation).

| File | Content |
| --- | --- |
| E1 | New file from the ADTI seed, nothing drawn |
| E2 | E1 + one line |
| E3 | E2 + one Latvian text |
| E4 | E3 + one ADTI cell + one two-line text node |
| E5 (optional) | Not made |

## Records

| File | Bytes | SHA-256 | Element IDs written |
| --- | --- | --- | --- |
| E1 | 47,104 | `431af1fd0b043baaa90832bfa46dca3028796c8f6be5f4f556de2a44c7a757c8` | none; highest existing ID 1287 |
| E2 | 47,104 | `468a870f744fd89820d0a3988a0eb3cc9b6118da6f15af6322817c0845dc48a0` | 1288 (line) |
| E3 | 47,104 | `1c29b14fdcc452ae7bc40dc0ed7d36dd8e550811d0c5e2e6137f3d0c9eba2c09` | 1289 (text) |
| E4 | 52,736 | `a3ca5ac45652c607b3489743fd2bd16248eb3237c22d8846bef6b7edb8b2b7f1` | 1363–1374 (cell), 1375 (level), 1376–1378 (text node) |

The streams were compared pairwise (E1→E2, E2→E3, E3→E4) after inflating
every zlib stream. Offsets below are from the start of an object, as in
FORMAT_NOTES.

## Observations

1. **The seed (E1).** One model `/Dgn-Md/#000000`, UOR per master unit
   10000 (metres), global origin 0. The model header object is type-66
   word `0x00400042`; the GDAL 3D sample has `0x00000042`, and ezdgn reads
   bit `0x00800000` as 2D, so ezdgn would report this 2D model as 3D
   (FN-M02). Empty extents are stored as low (2^52−1, 2^52−1, 0) and high
   (−2^52, −2^52, 0). The graphic page `Dgn^G/$1` is a header without
   data: record count 0, version 3, page number 1, population 0. Highest
   element ID 1287. The writer's `Dgn~H` counter read (H-H02) finds no
   value at the GDAL offset; nothing depends on it.
2. **New IDs (E2).** MicroStation gave the new line ID 1288, the highest
   existing ID plus one, as the writer does (FN-S02).
3. **MicroStation's header style (E2–E4).** Every object MicroStation
   placed directly carries type-word bit `0x00800000` (line `0x10800003`,
   text `0x10800011`, cell header `0x30800002`, text node `0x30800007`,
   node lines `0x50800011`) and properties word 0 at 0x24. The cell's
   components, which come from the cell library, carry `0x00c00000` and
   properties `0x80000000` instead, as the ODA-written GDAL sample does.
   The flags word 0x28 is 0 on the line, the text and the text node, and
   `0x0200` on the library cell and its components (FN-E13).
4. **ByLevel (E2).** The line, placed with ByLevel symbology, stores line
   style `0x7fffffff`, weight `0xffffffff` and colour `0xffffffff`
   (FN-E14). The cell components store line style `0xffffffff` (meaning
   not checked).
5. **The line went elsewhere (E2).** It runs from (0, 0) to (0, 10.5) m,
   not to (10, 5). Likely cause: the coordinates were typed with a decimal
   comma (`10,5` read as 10.5) in an auxiliary coordinate system whose X
   axis points north, as in LKS-92 conventions. To be confirmed by the
   user; it does not affect the format rules.
6. **Text (E3).** Font 1025, height and width 0.25 m (multipliers
   416666.67), justification 0. The Latvian text is stored as `ff fe` +
   UTF-16LE without `01 00` and without terminator, 30 characters in 62
   bytes (FN-T07). 0x80 holds the text's length in UOR as MicroStation
   measured it (48395.6, about 0.645 × height per character) and 0x88 its
   height in UOR (2500); the GDAL sample and this writer store zeros there
   (FN-T03). Range: low x = origin x rounded down, low y = origin y −
   0.5 × height, extent y 5001 = 2 × height (FN-T04). Two 24-byte
   linkages follow, IDs `0x80d4` and `0x56f3`, meaning unknown; texts
   without them display (EXP-0002).
7. **Text node (E4).** Type 7: line count 2 at 0x68, node number 1 at
   0x6c, then a style block that is zero in the GDAL sample: font 1025
   (u32 at 0x70), 255 (u16 at 0x74), justification 0 (u16 at 0x76), line
   spacing 5000 UOR = 0.5 m (f64 at 0x78), width and height multipliers
   (0x80, 0x88), then rotation and origin (FN-N02). The node origin is the
   top-left: the first line's baseline is one text height below it, and
   baselines are height + spacing = 0.75 m apart. The node range is the
   union of the line ranges. The two lines store their ASCII text as
   `ff fe 01 00` + bytes, and carry the same two linkages as the text.
8. **Cell (E4).** A normal cell (type 2), level 0. Count 11 at 0x68 = all
   descendants: 6 direct components, one of them a complex shape (role
   `0x70`) with 5 parts of its own. 0x6c holds `01 00 01 00`. The range at
   0x70/0x80 is relative to the origin: (−0.4061, 0) to (0.4061, 1.5) m,
   while the origin at 0xb0 is (4.6722, 2.3994) m (FN-C03). Identity
   matrix at 0x90. The name is a string linkage with property 1 and plain
   ASCII payload (6 bytes, no marker) in a 24-byte linkage.
9. **Levels (E1, E4).** The level table lives in `Dgn^Nm/$4`: a type-96
   header and type-95 entries. Table objects store their table kind where
   graphic elements store the level (0x0c); kind 1 is the level table.
   Entries hold the level ID at 0x20, the name in string-linkage property
   1 and the description in property 2. E1 has only "Default" (ID 64).
   ADTI levels are **not** in the seed: when the cell was placed,
   MicroStation copied its level from the library into the table as ID 1,
   and also wrote a second kind-1 table of 60 entries (IDs 1–60) whose
   objects carry type-word bit `0x00800000`; the writer reads that one as
   a library copy and ignores it (FN-V01). Level names copied by
   MicroStation use the `ff fe 01 00` + 8-bit form (FN-L03).
10. **Other tables.** Kind 19 in `Dgn^Nm/$3` has 1076 entries (1078 in E4)
    with line-style-like names, including names in the ADTI code pattern;
    the writer does not use it yet. Kinds 2–8 also occur.
11. **Pages.** Every page MicroStation wrote has page number 0 and
    population 0 in its header, whatever the record count: graphic pages
    (1, 3, 0, 0), (2, 3, 0, 0), (17, 3, 0, 0), and the `Dgn^Nm` pages too.
    The GDAL sample (ODA) stores page number 1 and population = record
    count (FN-P05).
12. **Other streams.** At the first save with an element, the u32 before
    the model header object in `Dgn~Mh` changed from 1 to 2 and stayed 2;
    the GDAL sample has 1 although it has elements. `Dgn^Ix/Dgn~Mix` (model
    index) gained 16 bytes and a flag bit `0x00020000`. `Dgn~H` and
    SummaryInformation change at every save. The writer leaves all of
    these unchanged; round 2 (EXP-0004) shows whether MicroStation needs
    them.
13. **Last-modified time.** The f64 at 0x18 of each new element is the
    UTC moment of placement (line: 2026-10-13 13:20:41 UTC), as in FN-E03a.

## Conclusions

- The writer gets a `profile` switch. `oda` keeps the round-1 conventions,
  so the round-1 test files rebuild byte for byte. `microstation` follows
  observations 3, 6, 7, 8 and 11.
- Jobs can name levels from the seed's own level table and use
  `"by_level"` for colour, weight and line style.
- ADTI levels have to be in the seed before the writer can use them. Either
  the user imports them into a working seed in MicroStation, or a later
  version writes level entries (not in scope yet).
- Round 2 (EXP-0004) tests the `microstation` profile on the ADTI seed.

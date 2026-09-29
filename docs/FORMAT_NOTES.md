# DGN V8 format notes (writer evidence ledger)

Every byte-level rule the writer relies on is listed here with its evidence.
Code comments cite these IDs; `tools/check_trace.py` fails CI when code cites
an ID that is not in this file, or when a rule has no evidence line.

Status values:

- **Confirmed**: reproduced byte for byte by a golden test against an
  independently produced file, or defined by a public specification.
- **Provisional**: observed, but only in the one ODA-produced sample; needs a
  MicroStation-made file before it is trusted for ADTI output.
- **Hypothesis** (`H-` IDs): not observed. Used only behind an explicit
  experiment switch or in a dedicated test file.

Evidence sources are described in [PROVENANCE.md](PROVENANCE.md). The main
sample is GDAL's `test_dgnv8.dgn` (SHA-256 `8f32f87c…d1df0`), analysed in
[EXP-0001](../research/EXP-0001-gdal-fixture.md). Rules first published by
ezdgn cite its commit `e7d72db` and were re-observed here.

## Container and streams

### FN-S01 Container and model storages
Status: Confirmed. A DGN V8 file is a Compound File Binary container
([MS-CFB], public). The root holds `/Dgn~H`, `/Dgn~S` and `/Dgn-Md`; each model
is a storage `/Dgn-Md/#NNNNNN` containing `Dgn~Mh` (model header) and
`Dgn^G` (graphic pages). No CLSIDs or state bits are set.
Evidence: ezdgn `docs/v8/FORMAT_NOTES.md` @e7d72db; EXP-0001 (stream list).

### FN-S02 Element IDs are unique across the file
Status: Provisional. Every object in every object page, and the model header
object, carries a u64 element ID at 0x10. IDs 1–95 are used across `Dgn^Nm`,
`Dgn^G` and `Dgn~Mh` without duplicates. The writer allocates new IDs from
`max + 1`.
Evidence: EXP-0001 (ID inventory); test `roundtrip::appended_elements_read_back_with_ezdgn`.

## Pages

### FN-P01 Page stream layout
Status: Confirmed. A `$N` stream is a 16-byte header of four u32 values
(record count, format version, page number, population) followed by a zlib
stream (`78 9c`). An empty page is the header alone. Format version 2 in the
sample (ezdgn also saw 3). Count and population are equal in every sample.
Evidence: ezdgn @e7d72db; EXP-0001; unit tests in `page.rs`.

### FN-P02 Object prefix
Status: Provisional. Each object inside an inflated page is preceded by a
4-byte prefix. It is zero for all 83 objects of the sample; the writer writes
zero. Meaning unknown.
Evidence: EXP-0001.

### FN-P03 Object framing
Status: Confirmed. An object starts with the u32 type word, the u32 total
length in 16-bit words and the u32 attribute offset in words. Primary data
precedes linkages. Object length is always even.
Evidence: ezdgn @e7d72db; golden tests (18 objects).

### FN-P04 Auxiliary storages
Status: Provisional. Storages ending in `A` (`Dgn^GA`, `Dgn^CA`, `Dgn^NmA`)
hold auxiliary records (28-byte headers, magic `0xa11b`), not objects. Their
4-byte `^AH` stream is 1 when the storage has one page and 0 when it has none.
The writer never touches them and adds no auxiliary records.
Evidence: ezdgn @e7d72db; EXP-0001.

## Common element header (0x68 bytes)

### FN-E01 Type word and role
Status: Confirmed. Low byte = element type. Top byte = role: `0x10`
standalone, `0x30` complex header, `0x50` complex component.
Evidence: EXP-0001; golden tests (18 objects).

### FN-E02 Level and element ID
Status: Confirmed. 0x0c u32 level ID (64 is the default level in the sample);
0x10 u64 element ID.
Evidence: EXP-0001; golden tests.

### FN-E03 Range is low corner plus extent
Status: Confirmed. 0x38: low x, y, z as i64; 0x50: extent x, y, z as i64
(`high - low`), in UOR. `low = floor(min)`, `high = ceil(max)`. Example:
3D line string 46 spans y 10000–70000 and stores low y 10000, extent 60000.
**Differs from ezdgn**, which reads 0x50 as the high corner.
Evidence: EXP-0001; golden tests incl. rounded arcs 54 and 55
(`golden::arcs_54_and_55_including_rounded_ranges`).

### FN-E03a Last-modified time
Status: Confirmed. 0x18 is an f64: milliseconds since 1970-01-01 UTC. The
sample value 1490737306000 is 2017-03-28 21:41:46 UTC, one second before the
file's CFB timestamps. **Differs from ezdgn**, which reads it as a model ID.
Evidence: EXP-0001; golden tests.

### FN-E04 Symbology
Status: Confirmed. 0x20 u32 graphic group, 0x2c u32 line style, 0x30 u32
line weight, 0x34 u32 colour index. Object 36 stores 2, 4, 5, 3 and GDAL's
reference CSV reports GraphicGroup 2, Style 4, Weight 5, ColorIndex 3.
Evidence: EXP-0001.

### FN-E05 Properties word
Status: Provisional. 0x24 u32 is `0x80000000` on every graphical object.
Meaning unknown; the writer copies it.
Evidence: EXP-0001.

### FN-E06 Flags word
Status: Provisional. 0x28 u32: `0x800` = 3D, `0x8000` = hole. `0x200` is set
on 2D line strings, shapes, curves, point strings and cells and absent on 2D
lines, arcs, ellipses, text and text nodes. `0x4000` and `0x8000` are set on
both sample cell headers, and both cells contain a hole. The writer
reproduces this pattern and writes plain `0x200` for cells without holes;
meanings other than 3D and hole are unknown.
Evidence: EXP-0001; golden tests.

### FN-E07 Complex elements
Status: Confirmed. A complex header (types 2, 7, 12, 14) is followed
immediately by its components; u32 component count at 0x68. The header range
covers its components.
Evidence: ezdgn @e7d72db; golden tests 42, 61, 72, 82.

## Element payloads (2D)

### FN-E08 Line string, shape, curve
Status: Confirmed. Types 4, 6, 11: u32 vertex count at 0x68, u32 zero at
0x6c, then f64 x, y pairs in UOR. A shape repeats its first vertex.
Evidence: golden tests 47, 57, 72–74.

### FN-E09 Line
Status: Confirmed. Type 3: start x, y and end x, y (f64) at 0x68.
Evidence: golden tests 37, 39.

### FN-E10 Ellipse and arc
Status: Confirmed. Type 15: primary axis, secondary axis, rotation (rad),
centre x, y. Type 16: start angle, sweep angle (rad; parametric, measured from
the rotated primary axis; negative sweep runs clockwise), primary, secondary,
rotation, centre x, y. A circle is an ellipse with equal axes.
Evidence: golden tests 51, 52, 54, 55, 84.

## Text

### FN-T01 Text size
Status: Confirmed. Type 17: f64 width multiplier at 0x70 and height multiplier
at 0x78; size in UOR = multiplier × 6 / 1000. The writer computes
`uor × (1000 / 6)`, which reproduces the sample bits exactly.
Evidence: ezdgn @e7d72db; golden tests 40, 43.

### FN-T02 Text payload
Status: Confirmed. u16 byte length at 0x6e; u16 editable-field word at 0xa8;
payload from 0xaa; object padded to even length. ASCII is stored as-is.
`myTéxt` is stored as `ff fe 01 00` + Windows-1252 bytes.
Evidence: ezdgn @e7d72db; golden tests 40, 43; unit test `text::tests`.

### FN-T03 Font, justification, rotation, origin
Status: Provisional. 0x68 u32 font number (127 and 1024 in the sample; GDAL
reports 1024 as Arial). 0x6c u16 justification: 0 in every sample, where the
origin is the left end of the baseline. 0x80–0x8f zero. 0x90 rotation (rad).
0x98 origin x, y. Other justification codes and font-table lookup need
MicroStation evidence.
Evidence: golden tests 40, 43.

### FN-T04 Text range depends on font metrics
Status: Confirmed. Text 43 (`z`, 10000 UOR) stores a 6693 × 9922 range, a
glyph box. The writer cannot compute this and stores a conservative box:
`n × width` long, from −0.25 × height to height.
Evidence: EXP-0001 (objects 40, 43).

### H-T03 Latvian text as escaped Windows-1257
Status: Hypothesis. `ff fe 01 00` + Windows-1257 bytes (Baltic). Test file
T03.
Evidence: none yet (EXP-0002 pending).

### H-T04 Latvian text as UTF-16
Status: Hypothesis. `ff fd` + UTF-16LE, as in string linkages (FN-L02). Test
file T04.
Evidence: none yet (EXP-0002 pending).

## Text node

### FN-N01 Text node header
Status: Confirmed. Type 7 (2D, 0xa8 bytes): u32 line count at 0x68, u32 node
number at 0x6c, rotation at 0x90, origin at 0x98; lines follow as type-17
components.
Evidence: golden test 42/43.

### H-N01 Text node style block
Status: Hypothesis. 0x70–0x8f (32 bytes) are zero in the sample. They
probably hold font, justification, line spacing and size. The writer writes
zeros, as the sample does.
Evidence: none yet (needs a MicroStation text node, EXP-0003).

## Complex chain and complex shape

### FN-X01 Complex chain and complex shape header
Status: Confirmed. Types 12 and 14 (0x70 bytes): u32 component count at 0x68,
u32 zero at 0x6c. The sample headers carry the 3D bit while their components
are 2D; the writer leaves it clear for 2D.
Evidence: golden tests 61, 82.

## Cells

### FN-C01 Cell header
Status: Provisional. Type 2 (2D, 0xc0 bytes): u32 component count at 0x68,
u32 = 1 at 0x6c, four f64 at 0x70, 2×2 matrix at 0x90, two f64 at 0xb0. The
only sample has range (0,0)–(10000,10000), identity matrix and origin (0,0),
so the writer's reading — 0x70 range low, 0x80 range high, 0xb0 origin, the
same order as the public V7 cell layout — is not yet distinguished from
ezdgn's (0x70 = origin). Test file T05 places a cell away from the origin.
Both sample cell headers are on level 0 with colour, weight and style 0;
the job format defaults cell headers to level 0.
Evidence: golden test 72; EXP-0002 pending.

### H-C02 Cell name
Status: Hypothesis. The name is stored as a string linkage (FN-L02) with
property ID 1, as on the sample's shared cell (types 34/35, "Named
definition"). Test file T06.
Evidence: none yet (EXP-0002 pending).

## Linkages

### FN-L01 Linkage framing
Status: Confirmed. Linkages follow the primary data. Each starts with a u16
whose low byte is `length / 2 - 1` and high byte `0x10`, then a u16 linkage
ID. Lengths are multiples of 8 bytes.
Evidence: ezdgn @e7d72db; EXP-0001 (model header and shared cell linkages).

### FN-L02 String linkage
Status: Confirmed. Linkage ID `0x56d2`: u32 property ID, u32 payload length,
payload `ff fd` + UTF-16LE, zero padding. Model name and description use
property IDs 1 and 2; unit labels use 0x13 and 0x14.
Evidence: EXP-0001; unit test `element::tests::string_linkage_matches_fixture_layout`.

## Model header (`Dgn~Mh`)

### FN-M01 Model header stream
Status: Confirmed. zlib from offset 0; the inflated stream ends with one
complete type-66 object (sample: u32 1, 4096 zero bytes, object at 0x1004).
Evidence: ezdgn @e7d72db; EXP-0001.

### FN-M02 2D model flag
Status: Provisional. Type-66 word bit `0x00800000` set = 2D model; clear in
the (3D) sample.
Evidence: ezdgn @e7d72db (public V8 files). Needs a MicroStation 2D seed.

### FN-M03 Global origin
Status: Provisional. +0xc8: global origin, three f64 in UOR (zero in the
sample). `uor = master × scale + origin`.
Evidence: ezdgn @e7d72db.

### FN-M04 Units
Status: Confirmed. +0xe0: UOR per master unit, f64 (10000 in the sample,
matching GDAL's master-unit coordinates).
Evidence: ezdgn @e7d72db; EXP-0001.

### FN-M05 Model extents
Status: Confirmed. +0x90: low x, y, z as i64; +0xa8: high x, y, z as i64 —
unlike element ranges, high is stored directly. Sample high
(60000, 70000, 80000) is the largest coordinate in the model.
Evidence: EXP-0001; test `roundtrip::extents_grow_to_cover_new_elements`.

## File header (`Dgn~H`)

### FN-H01 File header stream
Status: Confirmed. 20-byte uncompressed prefix, zlib from 0x14, 1576 bytes
inflated in the sample.
Evidence: ezdgn @e7d72db; EXP-0001.

### H-H02 Element ID counter
Status: Hypothesis. Inflated +0x128 u64 is 93 in the sample (highest ID 95).
+0x130 holds the same f64 time as the objects. Test file T07 writes the new
highest ID there; T01 leaves it unchanged.
Evidence: none yet (EXP-0002 pending).

[MS-CFB]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/

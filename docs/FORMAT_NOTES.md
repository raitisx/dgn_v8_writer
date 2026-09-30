# DGN V8 format notes (writer evidence ledger)

Every byte-level rule the writer relies on is listed here with its evidence.
Code comments cite these IDs; `tools/check_trace.py` fails CI when code cites
an ID that is not in this file, or when a rule has no evidence line.

Status values:

- **Confirmed**: reproduced byte for byte by a golden test against an
  independently produced file, shown correctly by MicroStation, or defined by
  a public specification.
- **Provisional**: observed, but not yet checked in MicroStation or in a
  MicroStation-made file.
- **Hypothesis** (`H-` IDs): not observed. Used only behind an explicit
  experiment switch or in a dedicated test file.
- **Rejected** / **Promoted**: a hypothesis that MicroStation disproved, or
  that became an `FN-` rule. Kept so the history stays traceable.

A `MicroStation:` line records what MicroStation itself showed. So far that
is Bentley Descartes StandAlone V8i SS5 (MicroStation V8i SS5 platform), in
[EXP-0002](../research/EXP-0002-microstation-acceptance.md).

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
MicroStation: shows Element ID 96 for the first appended element (T02, T05,
T06). Whether MicroStation's own next ID avoids ours is open (H-H02).
Evidence: EXP-0001 (ID inventory); EXP-0002;
test `roundtrip::appended_elements_read_back_with_ezdgn`.

## Pages

### FN-P01 Page stream layout
Status: Confirmed. A `$N` stream is a 16-byte header of four u32 values
(record count, format version, page number, population) followed by a zlib
stream (`78 9c`). An empty page is the header alone. Format version 2 in the
sample (ezdgn also saw 3). Count and population are equal in every sample.
MicroStation: opens files whose graphic page this code recompressed and
extended (T00–T07).
Evidence: ezdgn @e7d72db; EXP-0001; unit tests in `page.rs`; EXP-0002.

### FN-P02 Object prefix
Status: Provisional. Each object inside an inflated page is preceded by a
4-byte prefix. It is zero for all 83 objects of the sample; the writer writes
zero, which MicroStation accepts. Meaning unknown.
Evidence: EXP-0001; EXP-0002.

### FN-P03 Object framing
Status: Confirmed. An object starts with the u32 type word, the u32 total
length in 16-bit words and the u32 attribute offset in words. Primary data
precedes linkages. Object length is always even.
MicroStation: reports Size 91 words for a 182-byte text (T02) and Linkages 1
for a cell with one linkage (T06).
Evidence: ezdgn @e7d72db; golden tests (18 objects); EXP-0002.

### FN-P04 Auxiliary storages
Status: Provisional. Storages ending in `A` (`Dgn^GA`, `Dgn^CA`, `Dgn^NmA`)
hold auxiliary records (28-byte headers, magic `0xa11b`), not objects. Their
4-byte `^AH` stream is 1 when the storage has one page and 0 when it has none.
The writer never touches them and adds no auxiliary records; MicroStation
accepts elements without them (T01–T07).
Evidence: ezdgn @e7d72db; EXP-0001; EXP-0002.

## Common element header (0x68 bytes)

### FN-E01 Type word and role
Status: Confirmed. Low byte = element type. Top byte = role: `0x10`
standalone, `0x30` complex header, `0x50` complex component.
Evidence: EXP-0001; golden tests (18 objects); EXP-0002 T01, T05, T07.

### FN-E02 Level and element ID
Status: Confirmed. 0x0c u32 level ID (64 is the default level in the sample);
0x10 u64 element ID.
MicroStation: level 64 shows as "Default"; IDs as written (T02, T07).
Evidence: EXP-0001; golden tests; EXP-0002.

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
MicroStation: our value 1790640000000 (2026-09-29 00:00 UTC) shows as
"Last Modified 29-Sep-26 3:00 AM", Riga summer time (T02, T05–T07).
Evidence: EXP-0001; golden tests; EXP-0002.

### FN-E04 Graphic group and colour
Status: Confirmed. 0x20 u32 graphic group; 0x34 u32 colour index. Object 36
stores graphic group 2 and colour 3; GDAL's reference CSV reports the same.
MicroStation: colour 3 shows as colour 3 (T02); colours 1–6 display as
written (T01).
Evidence: EXP-0001; EXP-0002.

### FN-E11 Line style and weight
Status: Provisional. 0x2c u32 line style, 0x30 u32 line weight. Object 36
stores 4 and 5; GDAL's CSV reports Style 4, Weight 5.
MicroStation: lines, arcs, ellipses and shapes written with style 0,
weight 1 show Line Style 0, Weight 1 (T07). A text written with weight 1
shows Weight 0 (T02), so text weight lives elsewhere or is overridden;
test file T08 checks this.
Evidence: EXP-0001; EXP-0002 T02, T07.

### FN-E05 Properties word
Status: Provisional. 0x24 u32 is `0x80000000` on every graphical object.
Meaning unknown; the writer copies it, and MicroStation accepts it.
Evidence: EXP-0001; EXP-0002.

### FN-E06 Flags word
Status: Confirmed for the bits below. 0x28 u32:
- `0x0200` = the "New" property. MicroStation shows "New" for cell headers
  written with it (T05, T06) and "Not New" for a text without it (T02). The
  GDAL sample has it on some objects; the writer no longer sets it.
- `0x0800` = 3D (ezdgn; the writer never sets it).
- `0x8000` = hole. The sample's two type-2 elements also carry `0x4000` on
  the header; MicroStation lists them as "Grouped Hole", not "Cell" (T00,
  with every element selected; the Element Selection list shows the types
  of the current selection first).
Evidence: EXP-0001; golden tests; EXP-0002 T00, T02, T05, T06.

### H-E12 Flags word follows the V7 properties layout
Status: Hypothesis. The low 16 bits of 0x28 match the public V7 properties
word (class `0x000f`, locked `0x0100`, new `0x0200`, modified `0x0400`,
view-independent `0x1000`, planar `0x2000`, non-snappable `0x4000`, hole
`0x8000`), except that `0x0800` means 3D. Consistent so far: all-zero flags
show as Primary, Unlocked, Not New, Not Modified, View Dependent, Snappable
(T02); `0x0200` shows as New (T05).
Evidence: EXP-0002 T02, T05 (partial); V7 layout per GDAL's MIT-licensed
V7 `dgnlib`.

### FN-E07 Complex elements
Status: Confirmed. A complex header (types 2, 7, 12, 14) is followed
immediately by its components; u32 component count at 0x68. The header range
covers its components.
MicroStation: lists the components under each complex element (T07) and
draws them with the components' own symbology, not the header's (T01).
Evidence: ezdgn @e7d72db; golden tests 42, 61, 72, 82; EXP-0002 T01, T07.

## Element payloads (2D)

### FN-E08 Line string, shape, curve
Status: Confirmed. Types 4, 6, 11: u32 vertex count at 0x68, u32 zero at
0x6c, then f64 x, y pairs in UOR. A shape repeats its first vertex.
MicroStation: displays all three as written; the curve runs through the
inner points with two control points at each end (T01).
Evidence: golden tests 47, 57, 72–74; EXP-0002 T01.

### FN-E09 Line
Status: Confirmed. Type 3: start x, y and end x, y (f64) at 0x68.
MicroStation: displays as written (T01).
Evidence: golden tests 37, 39; EXP-0002 T01.

### FN-E10 Ellipse and arc
Status: Confirmed. Type 15: primary axis, secondary axis, rotation (rad),
centre x, y. Type 16: start angle, sweep angle (rad; parametric, measured from
the rotated primary axis; negative sweep runs clockwise), primary, secondary,
rotation, centre x, y. A circle is an ellipse with equal axes.
MicroStation: circle, rotated ellipse and both arcs display as written and
are listed as Circle, Ellipse and Arc (T01, T07).
Evidence: golden tests 51, 52, 54, 55, 84; EXP-0002 T01, T07.

## Text

### FN-T01 Text size
Status: Confirmed. Type 17: f64 width multiplier at 0x70 and height multiplier
at 0x78; size in UOR = multiplier × 6 / 1000. The writer computes
`uor × (1000 / 6)`, which reproduces the sample bits exactly.
MicroStation: Height 1.0000 and Width 1.0000 for a 1 m text (T02).
Evidence: ezdgn @e7d72db; golden tests 40, 43; EXP-0002 T02.

### FN-T02 Text payload
Status: Confirmed. u16 byte length at 0x6e; u16 editable-field word at 0xa8;
payload from 0xaa; object padded to even length. ASCII is stored as-is.
`myTéxt` is stored as `ff fe 01 00` + Windows-1252 bytes.
MicroStation: shows `myTéxt` (T00) and the ASCII texts (T02) correctly.
Evidence: ezdgn @e7d72db; golden tests 40, 43; unit test `text::tests`;
EXP-0002 T00, T02.

### FN-T03 Font, justification, rotation, origin
Status: Confirmed for font 1024 and justification 0. 0x68 u32 font number;
0x6c u16 justification; 0x80–0x8f zero; 0x90 rotation (rad); 0x98 origin.
MicroStation: font 1024 shows as "Arial" in the GDAL sample's font table.
Code 0 shows as "Left Top". The stored origin is the lower-left of the text:
for origin (20, 50) and height 1, MicroStation reports "User Origin
20, 51", the justification point, which it derives itself (T02). Rotation
30° displays as written. Font 127 draws lowercase as capitals in this
MicroStation setup.
Evidence: golden tests 40, 43; EXP-0002 T02.

### H-T05 Justification codes
Status: Hypothesis. Codes follow the public V7 table: 0 left top, 1 left
centre, 2 left bottom, 3–5 left margin, 6 centre top, 7 centre centre,
8 centre bottom, 9–11 right margin, 12 right top, 13 right centre, 14 right
bottom. Code 0 is confirmed (FN-T03). Test file T08 has codes 1, 2, 7, 12,
14.
Evidence: EXP-0002 T02 (code 0 only); V7 table per GDAL's MIT-licensed V7
`dgnlib`.

### FN-T04 Text range depends on font metrics
Status: Confirmed. Text 43 (`z`, 10000 UOR) stores a 6693 × 9922 range, a
glyph box. The writer cannot compute this and stores a conservative box:
`n × width` long, from −0.25 × height to height. MicroStation accepts it.
Evidence: EXP-0001 (objects 40, 43); EXP-0002 T02.

### FN-T05 Unicode text
Status: Confirmed. A text payload of `ff fd` + UTF-16LE (the string-linkage
form, FN-L02), without terminator. The writer's default for non-ASCII text.
MicroStation: shows `Rīga ĀČĒĢĪĶĻŅŠŪŽ āčēģīķļņšūž` correctly in Arial
(T04).
Evidence: EXP-0002 T04.

### H-T03 Latvian text as escaped Windows-1257
Status: Rejected. `ff fe 01 00` + Windows-1257 bytes. MicroStation shows the
bytes as Windows-1252 ("Rîga ÂÈÇÌÎÍÏÒÐÛÞ"), so the marker does not follow
the system code page. The option is kept only to rebuild T03.
Evidence: EXP-0002 T03.

### H-T04 Latvian text as UTF-16
Status: Promoted to FN-T05.
Evidence: EXP-0002 T04.

## Text node

### FN-N01 Text node header
Status: Confirmed. Type 7 (2D, 0xa8 bytes): u32 line count at 0x68, u32 node
number at 0x6c, rotation at 0x90, origin at 0x98; lines follow as type-17
components.
MicroStation: displays both lines of the node (T02).
Evidence: golden test 42/43; EXP-0002 T02.

### H-N01 Text node style block
Status: Hypothesis. 0x70–0x8f (32 bytes) are zero in the sample. They
probably hold font, justification, line spacing and size. The writer writes
zeros, as the sample does; MicroStation displays such nodes (T02).
Evidence: EXP-0002 T02 (acceptance only; the meaning needs a MicroStation
text node, EXP-0003).

## Complex chain and complex shape

### FN-X01 Complex chain and complex shape header
Status: Confirmed. Types 12 and 14 (0x70 bytes): u32 component count at 0x68,
u32 zero at 0x6c. The sample headers carry the 3D bit while their components
are 2D; the writer leaves it clear for 2D.
MicroStation: recognises both with their components (T07).
Evidence: golden tests 61, 82; EXP-0002 T01, T07.

## Cells

### FN-C01 Cell header
Status: Confirmed. Type 2 (2D, 0xc0 bytes): u32 component count at 0x68,
u32 = 1 at 0x6c, range low (x, y) at 0x70, range high (x, y) at 0x80, 2×2
matrix at 0x90, origin (x, y) at 0xb0. Components are stored in model
coordinates. Header on level 0, as in both samples.
MicroStation: T05 and T06 store range low (59, 49) and (69, 49) at 0x70;
MicroStation reports "Origin 60,50" and "70,50", "Angle 0°", "Scale 1",
"Number of elements 3" and "2", "Cell Type Graphic". So 0xb0 is the origin.
**Differs from ezdgn**, which reads 0x70 as the origin.
Evidence: golden test 72; EXP-0002 T05, T06.

### FN-C02 Cell name
Status: Confirmed. The name is a string linkage (FN-L02) with property ID 1,
as on the GDAL sample's shared cell.
MicroStation: "Cell Name ADTI_TEST", "Linkages 1" (T06).
Evidence: EXP-0002 T06.

### H-C02 Cell name
Status: Promoted to FN-C02.
Evidence: EXP-0002 T06.

## Linkages

### FN-L01 Linkage framing
Status: Confirmed. Linkages follow the primary data. Each starts with a u16
whose low byte is `length / 2 - 1` and high byte `0x10`, then a u16 linkage
ID. Lengths are multiples of 8 bytes.
Evidence: ezdgn @e7d72db; EXP-0001 (model header and shared cell linkages);
EXP-0002 T06.

### FN-L02 String linkage
Status: Confirmed. Linkage ID `0x56d2`: u32 property ID, u32 payload length,
payload `ff fd` + UTF-16LE, zero padding. Model name and description use
property IDs 1 and 2; unit labels use 0x13 and 0x14.
Evidence: EXP-0001; unit test `element::tests::string_linkage_matches_fixture_layout`;
EXP-0002 T06.

## Model header (`Dgn~Mh`)

### FN-M01 Model header stream
Status: Confirmed. zlib from offset 0; the inflated stream ends with one
complete type-66 object (sample: u32 1, 4096 zero bytes, object at 0x1004).
MicroStation: opens files whose `Dgn~Mh` this code recompressed (T00–T07).
Evidence: ezdgn @e7d72db; EXP-0001; EXP-0002.

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
MicroStation: coordinates written from master units show as intended, for
example cell origin 60,50 (T05).
Evidence: ezdgn @e7d72db; EXP-0001; EXP-0002.

### FN-M05 Model extents
Status: Confirmed. +0x90: low x, y, z as i64; +0xa8: high x, y, z as i64 —
unlike element ranges, high is stored directly. Sample high
(60000, 70000, 80000) is the largest coordinate in the model.
Evidence: EXP-0001; test `roundtrip::extents_grow_to_cover_new_elements`.

## File header (`Dgn~H`)

### FN-H01 File header stream
Status: Confirmed. 20-byte uncompressed prefix, zlib from 0x14, 1576 bytes
inflated in the sample.
Evidence: ezdgn @e7d72db; EXP-0001; EXP-0002 T07 (MicroStation opens a file
whose `Dgn~H` this code recompressed).

### H-H02 Element ID counter
Status: Hypothesis. Inflated +0x128 u64 is 93 in the sample (highest ID 95).
+0x130 holds the same f64 time as the objects. T07 writes the new highest ID
there and opens fine; T01 leaves it unchanged and also opens. Still open:
which ID MicroStation gives a newly placed element in T01 and in T07.
Evidence: EXP-0002 T01, T07 (acceptance only).

[MS-CFB]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/

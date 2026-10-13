# EXP-0004 MicroStation acceptance on the ADTI seed (round 2)

| Field | Value |
| --- | --- |
| Date | _pending_ |
| Performed by | raitisx (licensed MicroStation seat) |
| Product | _Help > About of the seat used_ |
| Seed | The empty ADTI LKS-2020 seed, E1 of EXP-0003 (SHA-256 `431af1fd…a757c8`) |
| Files tested | Built with `testfiles/adti_jobs/*.json` at the commit that adds those jobs; hashes below |
| Storage | The files contain VZD seed content and are not in this repository. Anyone with the same seed rebuilds them byte for byte: copy the seed to `private/adti_seed.dgn` (gitignored) and run `dgnv8w build testfiles/adti_jobs/<name>.json`. |
| Status | Pending |
| Decides | FN-P05, FN-E13, FN-E14, FN-T04, FN-T07, FN-N02, H-N03, FN-C03, FN-V01 on the ADTI seed; whether the untouched `Dgn~Mh` and `Dgn~Mix` fields matter (EXP-0003 obs. 12) |

All files except R01o use the `microstation` profile. Coordinates are in
metres.

## Files

| File | SHA-256 prefix | Content |
| --- | --- | --- |
| R00_rewrite | `47ebfaeb` | The seed rewritten by the writer, nothing added |
| R01_geometry | `87d712e4` | Line, line string, shape, curve, circle, rotated ellipse, two arcs, complex chain (U), complex shape (D), level Default, colours 1–6, weight 1 |
| R01o_geometry_oda | `efd31a97` | Same as R01 with the `oda` profile (round-1 conventions) |
| R02_text | `d9abb01b` | Texts in font 1025: ASCII 0.25 m, Latvian in MicroStation's form (`auto`) and in the round-1 form (`utf16`), 1 m, 30°; text nodes with justification 0, 1 and 2, and a three-line Latvian node |
| R03_cells | `70878272` | Cell ADTI_TEST at 0,40; cell ADTI_ROT at 5,40 rotated 30°, scale 2; unnamed cell at 10,40 with a text; cell ŠŪNA_Ā at 15,40 |
| R04_bylevel | `0770fe1f` | Line and text with ByLevel colour, weight and style; a line with colour 3, weight 2, style 2 |
| R04b_missing_level | `032dbd59` | A line on level 64 (Default) and a line on level ID 1, which the seed's level table lacks |
| R05_lks_coordinates | `e82d3074` | Line from 506000,312000 to 506010,312005, circle, text, cell, at real LKS-92 coordinates |

Full hashes:

```
47ebfaeb90ed01e341fb439f778c5b320136f845d19e143aa996ff1e92c92102  R00_rewrite.dgn
87d712e456890b54535df602c1b255722bb49a1d67f6ffcd3a29176c14677952  R01_geometry.dgn
efd31a97be8457e00ff326b11bb28ffa11128fbdb6325e16556c1c79536af614  R01o_geometry_oda.dgn
d9abb01bf4c12678ed621d887a8a8037837a85365ab4ea1dbe16f6218afaa9b9  R02_text.dgn
70878272e385b94269f43f59ffa8058fadebc9521bf1e70003c36a3a2840c9ba  R03_cells.dgn
0770fe1fa96e374651b862d6158a56fd27964c623e35efc6543e5b9bcb292aca  R04_bylevel.dgn
032dbd59432a0bbcedd08825e4890d056b14c9a84305c8af3353f78f0e4359c0  R04b_missing_level.dgn
e82d3074b809c414751f5bbce50a6cdc237ffb41014aeeb7611fbd43d3e2e42e  R05_lks_coordinates.dgn
```

Checked before sending: ezdgn 0.2.6 reads every file with the expected
element kinds and parents. Our ByLevel line in R04 has the same header
bytes as MicroStation's line in E2, apart from the time stamp.

## Checklist

For every file: open it as the first file after starting MicroStation,
note any message word for word, then `fit view extended`.

| File | Check |
| --- | --- |
| R00 | Opens without messages? |
| R01 | All ten elements visible as in the preview? Select the line: start 0,0 and end 10,5 in Element Information? Level "Default"? Place one new line and note its Element ID (expected 1303 or more). |
| R01o | Opens and looks the same as R01? If R01 fails and R01o works, the `microstation` profile is the cause. |
| R02 | Both Latvian lines readable? Font name for font 1025? For each text node: Justification, Line Spacing and Origin in Element Information (expected: J0 node 12,30.25; J1 node 12,26.75; J2 node 12,23.25; line spacing 0.5). Then edit one line of the J0 node (Edit Text, add a letter, OK): do the lines stay where they were? |
| R03 | Cell names ADTI_TEST, ADTI_ROT, ŠŪNA_Ā shown? Origin, Angle (30° for ADTI_ROT) and Scale (2) as written? Click each cell at a high zoom: does it highlight whole? |
| R04 | The first line and the text: Color, Weight and Line Style show "ByLevel"? The second line: 3, 2, 2? |
| R04b | What does MicroStation show for the line on level ID 1 (level name, or a message)? Does Level Manager list a new level? |
| R05 | Element Information of the line: start 506000,312000? Does MicroStation show the same numbers, or swapped (X north)? |

Also useful:

- **E2's line** ran to 0,10.5 instead of 10,5 (EXP-0003 obs. 5). Was the
  point typed as `xy=10,5` with a decimal comma setting, and is an ACS
  active? R05 helps answer this.
- **Levels:** the seed has only "Default". To use ADTI level names, the
  writer needs a seed that already contains them. Would a working seed with
  the ADTI levels imported (Level Manager > Import from the ADTI dgnlib) be
  acceptable in your workflow?

## Results

_pending_

# Inspection scripts

Python scripts used to study DGN V8 files for the research records
(`research/EXP-*.md`). They only read files. The writer itself does not
need them.

| Script | Needs | Purpose |
| --- | --- | --- |
| `v8dump.py` | `pip install olefile` | Containers, pages, objects, linkages, tables, stream diffs |
| `preview.py` | `pip install "ezdgn[plot]"` | Draws files with the independent ezdgn reader, side by side, as a PNG |

## Safety

Treat every DGN file you inspect as untrusted data, especially files from
others. Keep the file in its own folder, run the script from a different
folder, and start Python with `-I` so it does not load modules from the
current folder:

```text
python3 -I research/tools/v8dump.py summary /path/to/evidence/E1.dgn
```

## v8dump.py

| Command | Shows |
| --- | --- |
| `summary FILE` | Size, SHA-256, the saving application and save time from SummaryInformation (no user names) |
| `entries FILE` | Every CFB storage and stream with its size |
| `pages FILE` | Every `$N` page stream with its header (count, version, page number, population), and each model header's type word |
| `objects FILE STORAGE` | Every object of the pages in `STORAGE`: type, role, length, level, ID, time, symbology |
| `hex FILE STORAGE ID[,ID]` | Hex dump of the objects with those element IDs |
| `linkages FILE STORAGE ID[,ID]` | The linkages of those objects, one per line |
| `tables FILE` | Table headers (type 96) and entries (type 95) with table kind, count or ID, and name; kind 1 is the level table, `library` marks library copies |
| `stream FILE PATH [ZOFF]` | Hex dump of one stream, inflated from offset `ZOFF` (for example `0` for `Dgn~Mh`, `0x14` for `Dgn~H`) |
| `diff FILE_A FILE_B` | Which streams differ, compared after inflating, with the first differing offsets |

`STORAGE` is a path inside the container such as `Dgn-Md/#000000/Dgn^G`
(graphic pages of the first model) or `Dgn^Nm` (tables).

Typical uses:

- **What changed when MicroStation did one thing?** `diff E1.dgn E2.dgn`,
  then `objects` and `hex` on the streams that changed.
- **What did the writer produce?** `objects out.dgn "Dgn-Md/#000000/Dgn^G"`
  and compare with a MicroStation-made object using `hex`.
- **Which levels does a seed have?** `tables seed.dgn`, or `dgnv8w inspect
  seed.dgn`.

## preview.py

```text
python3 -I research/tools/preview.py out.png a.dgn b.dgn --from-id 1288
```

Draws each file in its own panel and prints the kinds of elements with an
ID of at least `--from-id`, plus their texts as ezdgn decodes them. ezdgn
shows Unicode texts as raw bytes and draws colour 0 white, so such texts can
look wrong or be invisible in the preview while MicroStation shows them
correctly.

## Licence

These scripts are part of this repository (MIT). `olefile` is BSD-licensed;
ezdgn is MIT-licensed.

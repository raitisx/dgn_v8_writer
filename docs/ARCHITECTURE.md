# Architecture

The writer is one Rust crate, `crates/dgnv8-writer`: a library
(`dgnv8_writer`) and a command-line program (`dgnv8w`). Its runtime
dependencies are pure Rust: `cfb` (compound files), `flate2` (zlib),
`encoding_rs`, `serde`/`serde_json`, `thiserror`. The reader used to check
output, ezdgn, is a test-only dependency.

## Method: seed + append

A DGN V8 file is a compound file (like an old `.doc`) holding many streams.
The writer never builds one from nothing. It copies the seed byte for byte
and changes only:

1. the model's last graphic page (`Dgn-Md/#000000/Dgn^G/$N`), where new
   elements are appended;
2. the model header (`Dgn~Mh`), whose extents grow to cover them;
3. the level table page (`Dgn^Nm/$N`), when the job adds levels;
4. only on request, an experimental counter in `Dgn~H` (not needed).

Everything else, including fonts, colours, line styles, units and settings,
comes from the seed unchanged. This keeps the writer small, and keeps
anything it does not understand exactly as MicroStation wrote it.

## Data flow of `dgnv8w build job.json`

```text
job.json ──spec.rs──▶ Element values (in UOR) ──element.rs──▶ object bytes
                          ▲                                       │
seed.dgn ──seed.rs──▶ SeedInfo (units, levels, max ID)           ▼
                                                   document.rs: copy seed,
                                                   level.rs: add levels,
                                                   page.rs: append objects,
                                                   grow extents, save
```

1. `spec::run_job` reads the JSON (`JobSpec`) and opens the seed as a
   `Document`.
2. `seed::analyse` finds the model, its UOR per master unit and global
   origin, its extents, the highest element ID in the file, and the level
   table.
3. Levels listed in the job are registered with `Document::ensure_level`.
4. Each element spec becomes an `Element` in UOR (`master × UOR + origin`),
   with symbology resolved: level names to IDs, `"by_level"` to the ByLevel
   values, defaults and inheritance applied.
5. `Document::to_bytes` allocates element IDs above the seed's highest,
   writes new level entries, encodes every element (`Element::encode`),
   appends the objects to the last graphic page, grows the model extents
   and writes the compound file.

## Modules

| Module | Role | Main rules |
| --- | --- | --- |
| `seed.rs` | Read-only analysis of a seed: model, units, extents, highest ID, levels, 2D flag | FN-S01, FN-S02, FN-M01–M05, FN-V01 |
| `page.rs` | `$N` page streams: 16-byte header, zlib payload, framed objects | FN-P01–P05 |
| `element.rs` | Byte encoders for every element type; profiles; linkages | FN-E*, FN-T*, FN-N*, FN-X01, FN-C*, FN-L* |
| `text.rs` | Text payload encodings and decoding | FN-T02, FN-T05, FN-T07, FN-L03 |
| `geom.rs` | Points, bounds, integer ranges, ellipse and arc extents | FN-E03 |
| `level.rs` | New entries in the level table | FN-V03, FN-V04 |
| `document.rs` | Seed + append: IDs, levels, pages, extents, saving | FN-M05, FN-S02 |
| `spec.rs` | The JSON job format ([JOB_FORMAT.md](JOB_FORMAT.md)) | FN-E14, FN-V02, FN-N04 |
| `bin/dgnv8w.rs` | `inspect` and `build` commands | |
| `error.rs` | `WriteError` | |

### Profiles

ODA (which wrote GDAL's sample) and MicroStation store some things
differently: a type-word bit, the properties word, page header fields, the
text range, the text node style block, the cell range, cell name bytes and
text node linkages. `Profile::MicroStation` (the default) follows
MicroStation; `Profile::Oda` reproduces the round-1 test files exactly. The
differences are listed in [SCOPE.md](SCOPE.md#profiles).

## Using the library from Rust

```rust
use dgnv8_writer::element::{Element, Geometry, Symbology};
use dgnv8_writer::{Document, WriteOptions};

let mut doc = Document::from_seed_path("adti_seed.dgn", None)?;
let level = doc.ensure_level("MY_LEVEL", Some("Mans līmenis"))?;
let start = doc.master_to_uor(506_000.0, 312_000.0);
let end = doc.master_to_uor(506_010.0, 312_005.0);
doc.add(Element {
    symbology: Symbology { level, color: 3, ..Symbology::default() },
    geometry: Geometry::Line { start, end },
    extra_flags: 0,
});
let report = doc.save("out.dgn", &WriteOptions::default())?;
```

The `Geometry` values take UOR (convert with `master_to_uor` and
`distance_to_uor`). The JSON job format does those conversions, and the
symbology defaults and inheritance, for you; for most integrations, writing
job files and calling `dgnv8w build` is simpler. API documentation:
`cargo doc --open`.

## Tests

| Layer | Where | What it proves |
| --- | --- | --- |
| Golden bytes | `tests/golden.rs` | The encoders reproduce 18 objects of GDAL's ODA-written sample byte for byte (`oda` profile) |
| Read-back | `tests/roundtrip.rs` | Files the writer makes are read back correctly by the independent ezdgn reader; seed objects stay unchanged; levels are added correctly |
| Unit tests | `#[cfg(test)]` in each module | Layouts observed in MicroStation's own files (EXP-0003), job parsing, symbology rules; every job file in the repository parses |
| Reproducibility | `tools/build_testfiles.sh` | The test files rebuild byte for byte from their jobs; with `--adti`, also the files tested on the ADTI seed |
| Traceability | `tools/check_trace.py` | Every rule cited in code exists in FORMAT_NOTES.md, has a status and evidence, and every cited experiment has a record |
| MicroStation | `research/EXP-0002`–`EXP-0006` | A person opened the files in MicroStation and recorded what it showed |

CI (`.github/workflows/ci.yml`) runs everything except the last layer and
the `--adti` rebuild on every push. `.github/workflows/package.yml` builds
the release bundles for Windows, Linux, macOS and FreeBSD (the last in a
FreeBSD virtual machine, with the full test suite).

## Adding a feature

1. **Evidence first.** Find out how MicroStation stores it: make a
   minimal-pair file in MicroStation (one action on a copy of the empty
   seed), compare with `research/tools/v8dump.py diff`, and write the
   observation into a research record (`research/EXP-NNNN-*.md`).
2. **Rule.** Add the rule to [FORMAT_NOTES.md](FORMAT_NOTES.md) with an ID,
   a status and an evidence line. Anything not observed is a hypothesis
   (`H-` ID) and stays behind an explicit option.
3. **Code.** Implement it, citing the rule ID in a comment, with a unit test
   built from the observed bytes or values.
4. **Test file.** Add a job in `testfiles/adti_jobs/`, build it with
   `tools/build_testfiles.sh --adti`, add its hash to
   `testfiles/adti_jobs/SHA256SUMS`, and write a checklist in a new
   research record.
5. **MicroStation.** Have the files opened in MicroStation, record the
   results, and update the rule's status (Hypothesis → Provisional →
   Confirmed).
6. **Commit** the evidence before the code that depends on it, and run
   `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
   and `python3 tools/check_trace.py` first.

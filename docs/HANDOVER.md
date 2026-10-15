# Handover

State of the project on 2026-10-15, for whoever integrates or continues
it. Start here; the other documents are linked where they matter.

## What this is

`dgnv8w` writes 2D DGN V8 drawings for Latvian ADTI (augstas detalizācijas
topogrāfiskā informācija) into the official LKS-2020 ADTI seed. It takes a
JSON job (seed, output, levels, elements) and writes a DGN that
MicroStation CONNECT Edition opens and edits normally.

It was built clean-room: every byte-level rule comes from files the project
may legally study, or from experiments in a licensed MicroStation, and each
rule is traceable from the code to its evidence
([PROVENANCE.md](PROVENANCE.md), [FORMAT_NOTES.md](FORMAT_NOTES.md)).

## Status

Everything in the original scope works in MicroStation CONNECT v10 on the
ADTI seed:

| Feature | Verified in MicroStation |
| --- | --- |
| Line, line string, shape (with hole flag), curve, circle, ellipse, arc | Displays as written (EXP-0004 R01) |
| Complex chain, complex shape | Displays as written (R01) |
| Text, ASCII and Latvian letters, any rotation, font 1025 = Arial | Displays correctly (R02); stays in place when edited (EXP-0005) |
| Text justification | Codes 0, 1, 2, 7, 12, 14 (EXP-0002, V8i); node codes 0, 1, 2 (EXP-0005) |
| Text node (several lines) | Displays; lines stay apart when edited (EXP-0005) |
| Normal cell, named or not, rotated, scaled, Latvian name | Name, origin, angle, scale as written (R03) |
| Colour, weight, line style, ByLevel | As written (R04) |
| Levels by name; levels added by the writer | Listed, used, hidden, deleted, kept through save (EXP-0006) |
| LKS-92 coordinates | Shown as written, x = easting (R05) |
| Element IDs | New IDs above the seed's; MicroStation continues after ours (R01) |

Format rules: 59 in FORMAT_NOTES.md; 43 confirmed, 7 hypotheses promoted to
rules, 6 provisional (observed, not fully pinned down; none blocks the
features above), 1 hypothesis the writer does not rely on (H-E12), 2
rejected.

Tests: unit, golden-byte and read-back tests, the traceability check, and
byte-for-byte rebuilds of every file tested in MicroStation; CI runs them on
every push.

## Getting started

Without Rust: download a bundle from the repository's **Actions → Package**
run (Windows, Linux, macOS or FreeBSD), or from a release if one has been
tagged. Each bundle holds `dgnv8w`, these docs, examples and the inspection
scripts (see `BUNDLE.md` inside).

With Rust 1.83 or newer:

```text
cargo build --release
target/release/dgnv8w inspect adti_seed.dgn      # units, levels, highest element ID
target/release/dgnv8w build examples/gdal_example.json
```

Then write a job for the ADTI seed, starting from
[`examples/adti_example.json`](../examples/adti_example.json). The full
format is in [JOB_FORMAT.md](JOB_FORMAT.md).

## Integrating

The simplest integration is: your code turns source data into a job file,
then runs `dgnv8w build job.json`. The job format does unit conversion,
level lookup, defaults and symbology inheritance. A Rust program can use
the library directly instead ([ARCHITECTURE.md](ARCHITECTURE.md#using-the-library-from-rust)).

Recommendations:

- Use the empty ADTI seed (E1 of EXP-0003, SHA-256 `431af1fd…a757c8`) and
  the default `"microstation"` profile.
- Name levels; list the ADTI levels you need under `"levels"` and the writer
  adds them. Keep `"allow_unknown_levels"` off.
- Set `"font": 1025` (Arial in the ADTI seed) and the text height in the
  job's `defaults`.
- Coordinates are LKS-92 metres with x = easting, y = northing. If source
  data gives X as northing, swap the values when writing the job.
- Set `"modified_ms"` when you need reproducible output (same job, same
  bytes); leave it out to stamp the current time.
- Check the first real drawing in MicroStation before relying on it, and
  keep that check in a research record.
- Inside an element, misspelled keys are ignored (`colour` instead of
  `color`); top-level keys are checked.

## Rules that keep the legal path clean

These come from [PROVENANCE.md](PROVENANCE.md); keep following them.

1. **Forbidden inputs**: ODA or Bentley SDK code, headers, documentation or
   binaries (including GDAL's `dgnv8` driver source), NDA material, leaked
   specifications, decompiled code.
2. **Evidence first**: a rule enters FORMAT_NOTES.md only with evidence
   (a golden test, a recorded experiment, a public specification). What an
   AI assistant proposes is a hypothesis (`H-` ID) until evidence confirms
   it, and stays behind an explicit option meanwhile.
3. **No VZD material in the repository**: the ADTI seed, level and cell
   libraries, and any file made from them stay outside it. The repository
   is public. Jobs for the ADTI seed read it from `private/adti_seed.dgn`,
   which git ignores; only hashes and observations are recorded.
4. **No real ADTI data in tests**: it can contain restricted information.
5. **Open items**: the contributor declaration in PROVENANCE.md is still
   to be filled in by raitisx, and PROVENANCE.md asks for a review by a
   Latvian IP lawyer before any release.

## How the work was done

| Record | What it established |
| --- | --- |
| [EXP-0001](../research/EXP-0001-gdal-fixture.md) | Layouts from GDAL's public test file (written by ODA); golden tests |
| [EXP-0002](../research/EXP-0002-microstation-acceptance.md) | Round 1 in MicroStation V8i: all element types display as written |
| [EXP-0003](../research/EXP-0003-microstation-evidence-files.md) | What MicroStation itself writes into the ADTI seed (E1–E4) |
| [EXP-0004](../research/EXP-0004-adti-seed-acceptance.md) | Round 2 in MicroStation CONNECT on the ADTI seed |
| [EXP-0005](../research/EXP-0005-text-node-editing.md) | Round 3: editable text nodes |
| [EXP-0006](../research/EXP-0006-new-levels.md) | Round 4: levels added by the writer |

The evidence files E1–E4 and the round 2–4 test files are private (they
contain the VZD seed). Anyone with the same seed rebuilds the test files
byte for byte with `tools/build_testfiles.sh --adti`.

## Known limits and open questions

| Item | Effect | Next step |
| --- | --- | --- |
| New levels copy the seed's "Default" level | Their ByLevel colour, weight and style are Default's | Evidence file E6: a level made in Level Manager with colour 3, weight 2, style 2 (EXP-0006) |
| Level Manager's number column | A level made in MicroStation showed "no number"; meaning of that column unclear | The file saved in EXP-0006 step 6 shows it |
| Large drawings | All elements go into the model's last page; thousands of elements are untested | Evidence file E5 (a few thousand elements made in MicroStation) and a large test job |
| Text length | The writer cannot measure fonts, so stored text ranges are generous | Accepted by MicroStation; affects only fit and selection boxes |
| Text node justification | Only 0, 1, 2 with the `microstation` profile | Others need line lengths |
| Not supported | Shared cells, nested cells, complex elements inside cells, B-splines, dimensions, tags, 3D | Not needed for ADTI so far ([SCOPE.md](SCOPE.md)) |
| ADTI line styles | The seed has custom line styles by number; using them is untested | A test job with `style` set to one of them |
| `Dgn~Mh`, `Dgn^Ix/Dgn~Mix` | MicroStation updates fields there that the writer leaves alone | Accepted by MicroStation (EXP-0004); nothing to do |

Findings worth offering to the ezdgn maintainer (with their agreement
first) are in [UPSTREAM.md](UPSTREAM.md); among them, ezdgn reports the
ADTI 2D model as 3D and does not decode MicroStation's Unicode text.

## Repository map

| Path | Contents |
| --- | --- |
| `crates/dgnv8-writer/` | The library and `dgnv8w` ([ARCHITECTURE.md](ARCHITECTURE.md)) |
| `docs/` | This handover, job format, architecture, scope, format rules, provenance, upstream notes |
| `research/` | One record per experiment, and the inspection scripts in `research/tools/` |
| `testfiles/` | Round-1 test files (GDAL sample) with their jobs and checksums; `adti_jobs/` for rounds 2–4 |
| `examples/` | Two jobs to start from |
| `tools/` | `check_trace.py`, `build_testfiles.sh`, `package.sh` |
| `.github/workflows/` | `ci.yml` (checks on every push), `package.yml` (bundles) |
| `private/` | Ignored by git: the ADTI seed and files made from it |

## Everyday commands

```text
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test
python3 tools/check_trace.py            # code -> rule -> evidence
tools/build_testfiles.sh [--adti]       # rebuild and verify test files
tools/package.sh                        # bundle for this machine, in dist/
python3 -I research/tools/v8dump.py     # inspect a DGN (see research/tools/README.md)
```

## Git

All work is on branch `ccr-8751c1e4-vgwssb` of `raitisx/dgn_v8_writer`; no
pull request has been opened yet. Commits are signed off (`git commit -s`)
and evidence is committed before the code that relies on it.

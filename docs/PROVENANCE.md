# Provenance policy and evidence inventory

This is an engineering record of where every piece of format knowledge in
this repository comes from. It is not legal advice; have a Latvian IP lawyer
review it before any release.

## Approach

The writer is built by black-box study of files, never of programs:

- Public specifications (the CFB container, [MS-CFB]).
- Files this project may legally use, each with a recorded origin, licence
  and SHA-256.
- Controlled experiments: files made by the project owner in a licensed
  MicroStation, changing one thing at a time, with the steps recorded in
  `research/EXP-*.md`.
- Open-source code under compatible licences, used as documented.

Each rule enters [FORMAT_NOTES.md](FORMAT_NOTES.md) with a status and an
evidence line. Code cites the rule ID. `tools/check_trace.py` (run in CI)
checks the chain: code → rule → experiment record.

## Forbidden inputs

Never used, not even for "a quick look":

- ODA (Open Design Alliance) SDK source, headers, documentation, binaries,
  generated bindings, debugger output or decompiled code. This includes
  GDAL's `dgnv8` driver source, which wraps the ODA API.
- Bentley SDK material, documentation obtained under NDA or a developer
  programme, and decompiled or disassembled MicroStation code.
- Leaked or confidential format specifications.
- Text copied from Bentley manuals (facts may be restated; wording may not).

Files produced by ODA-based software (such as GDAL's test file) may be
*analysed* as files, like any other output file.

## AI assistance

This repository is developed with an AI assistant (Claude Code). An AI model
cannot say where it learned something, so anything it proposes about DGN
internals is treated as a hypothesis (`H-` IDs). A rule becomes Provisional
or Confirmed only through repository-owned evidence: a golden test against a
recorded file, or a recorded experiment. Hypotheses are used only behind
explicit experiment switches or in dedicated test files.

## Contributor declarations

Each contributor states whether they have ever accepted terms that limit
reverse engineering or study of DGN (ODA membership, Bentley developer or
partner agreements, NDAs, employer agreements). Someone bound by such terms
must not contribute format findings covered by them.

| Contributor | Declaration | Date |
| --- | --- | --- |
| raitisx | _to be filled in by raitisx_ | |

## Evidence inventory

| ID | Item | Origin and licence | SHA-256 | Use |
| --- | --- | --- | --- | --- |
| F-GDAL-V8 | `crates/dgnv8-writer/tests/data/gdal_test_dgnv8.dgn` | OSGeo/GDAL `autotest/ogr/data/dgnv8/test_dgnv8.dgn` @ `18e7cceb43a0dd58be474c9fdd5384baa3cde7c9`; GDAL MIT-style licence (`LICENSE.GDAL.txt`) | `8f32f87ce4b16881aa64f5cb9f75c98851833f96fef37ca0ad31aa6bb18d1df0` | Layout analysis (EXP-0001), golden tests, seed for test round 1 |
| F-GDAL-CSV | `test_dgnv8_ref.csv` (not copied here) | Same GDAL commit; GDAL licence | `09f765e0aa8dc06bb1d0da4ed86f060dc0a839d480b3dc67e54841ecceec6433` | Black-box check of values only |
| C-EZDGN | ezdgn source and `docs/v8/FORMAT_NOTES.md` | monozukuri-ai/ezdgn @ `e7d72db22c9b7f19b0404bb699706378a48d6fb0`, MIT | n/a | Read-back oracle (dev-dependency); cited rules re-observed in EXP-0001 |
| MS-* | MicroStation evidence files E1–E5 | Made by raitisx (EXP-0003); private repository, not redistributed | _pending_ | Confirm or reject Provisional and Hypothesis rules |

## Data policy

- No real ADTI extracts in this repository or its tests. They can contain
  restricted information (for example utility networks). Synthetic data only.
- VZD seed, level library, cell library, line-style and font files are not
  redistributed. The writer reads them from paths the user supplies.
- Files derived from those resources (evidence files, re-saved test files)
  go to the private evidence repository; only hashes and findings come here.

## Admitting a new evidence file

Record, before using it:

- who made or supplied it, and under what redistribution terms;
- the creating application and version;
- synthetic, sanitised or production-derived;
- what was deliberately varied compared with its predecessor;
- size and SHA-256;
- where the expected values came from (manual reading in MicroStation,
  another parser, or the creation steps).

[MS-CFB]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/

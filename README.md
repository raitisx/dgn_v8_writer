# dgn_v8_writer

A clean-room DGN V8 writer for 2D Latvian ADTI drawings, with a traceable
record of where every piece of format knowledge comes from.

**Status: prototype, not for production.** MicroStation V8i opened every
round-1 test file (built on GDAL's sample) and showed the elements as
written ([EXP-0002](research/EXP-0002-microstation-acceptance.md)).
MicroStation CONNECT v10 opened every round-2 file built on the real ADTI
seed ([EXP-0004](research/EXP-0004-adti-seed-acceptance.md)); text nodes
keep their lines when edited
([EXP-0005](research/EXP-0005-text-node-editing.md)), and levels the
writer adds work like MicroStation's own
([EXP-0006](research/EXP-0006-new-levels.md)).

## Documentation

| Document | For |
| --- | --- |
| [docs/HANDOVER.md](docs/HANDOVER.md) | **Start here**: status, getting started, integration, rules to keep, open questions |
| [docs/JOB_FORMAT.md](docs/JOB_FORMAT.md) | Every option of the JSON job that `dgnv8w build` reads |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | How the code works, tests, and how to add a feature |
| [docs/SCOPE.md](docs/SCOPE.md) | What is in and out of scope, element status per test round |
| [docs/FORMAT_NOTES.md](docs/FORMAT_NOTES.md) | Every byte-level rule with its status and evidence |
| [docs/PROVENANCE.md](docs/PROVENANCE.md) | Allowed and forbidden sources, evidence inventory, AI-assistance policy |
| [docs/UPSTREAM.md](docs/UPSTREAM.md) | Findings to offer the ezdgn maintainer |
| [research/](research/) | One record per experiment; inspection scripts in [research/tools/](research/tools/README.md) |
| [testfiles/README.md](testfiles/README.md) | The MicroStation test rounds and how to rebuild their files |
| [examples/](examples/README.md) | Two jobs to start from |

## Command line

```text
cargo build --release
target/release/dgnv8w inspect seed.dgn        # what the writer sees in a seed
target/release/dgnv8w build job.json          # append the job's elements
```

Ready-built bundles for Windows, Linux and macOS come from the **Package**
workflow (Actions tab), or from releases.

## How it works

The writer takes a seed DGN (for ADTI: the official LKS-2020 seed, which you
supply) and appends 2D elements to one of its models, adding levels when
the job asks for them. Everything it does not need to change is copied byte
for byte.

Supported: line, line string, shape, curve, circle/ellipse, arc, complex
chain, complex shape, text, text node and normal cell, with levels by name,
ByLevel symbology and Latvian text.

## Traceability

- `tools/check_trace.py` (run in CI) checks that code cites only documented
  rules, and that each rule has evidence.
- `crates/dgnv8-writer/tests/golden.rs`: the encoders reproduce objects of
  GDAL's public V8 test file byte for byte.
- `crates/dgnv8-writer/tests/roundtrip.rs`: output is read back with the
  independent [ezdgn](https://github.com/monozukuri-ai/ezdgn) reader.
- `tools/build_testfiles.sh`: every file tested in MicroStation rebuilds
  byte for byte from its job.

## Licence

MIT, see [LICENSE](LICENSE). The test seed
`crates/dgnv8-writer/tests/data/gdal_test_dgnv8.dgn` and the test files built
from it are GDAL test data under GDAL's MIT-style licence
(`crates/dgnv8-writer/tests/data/LICENSE.GDAL.txt`).

MicroStation is a trademark of Bentley Systems. This project is not
affiliated with or endorsed by Bentley Systems or the Open Design Alliance.

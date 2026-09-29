# dgn_v8_writer

A clean-room DGN V8 writer for 2D Latvian ADTI drawings, with a traceable
record of where every piece of format knowledge comes from.

**Status: first prototype, not for production.** The writer passes its own
tests. It has not yet been checked in MicroStation.

## How it works

The writer takes a seed DGN (for ADTI: the official LKS-2020 seed, which you
supply) and appends 2D elements to one of its models. Everything it does not
need to change is copied byte for byte. See [docs/SCOPE.md](docs/SCOPE.md).

Supported so far: line, line string, shape, curve, circle/ellipse, arc,
complex chain, complex shape, text, text node and cell. Status per element
is in the scope document.

## Testing in MicroStation

Test files and a checklist are in [testfiles/](testfiles/README.md). No
coding is needed.

## Traceability

- [docs/FORMAT_NOTES.md](docs/FORMAT_NOTES.md): every byte-level rule, its
  status and its evidence.
- [docs/PROVENANCE.md](docs/PROVENANCE.md): allowed and forbidden sources,
  evidence inventory with hashes, AI-assistance policy.
- [research/](research/): one record per experiment.
- `tools/check_trace.py` (run in CI) checks that code cites only documented
  rules, and that each rule has evidence.
- `crates/dgnv8-writer/tests/golden.rs`: the encoders reproduce objects of
  GDAL's public V8 test file byte for byte.
- `crates/dgnv8-writer/tests/roundtrip.rs`: output is read back with the
  independent [ezdgn](https://github.com/monozukuri-ai/ezdgn) reader.

## Command line

```text
cargo build --release
target/release/dgnv8w inspect seed.dgn        # what the writer sees in a seed
target/release/dgnv8w build job.json          # append the job's elements
```

A job is a JSON file naming the seed, the output and the elements, in master
units (metres) and degrees. Examples: [testfiles/jobs/](testfiles/jobs/).

## Licence

MIT, see [LICENSE](LICENSE). The test seed
`crates/dgnv8-writer/tests/data/gdal_test_dgnv8.dgn` and the test files built
from it are GDAL test data under GDAL's MIT-style licence
(`crates/dgnv8-writer/tests/data/LICENSE.GDAL.txt`).

MicroStation is a trademark of Bentley Systems. This project is not
affiliated with or endorsed by Bentley Systems or the Open Design Alliance.

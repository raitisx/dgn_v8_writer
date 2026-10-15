#!/usr/bin/env bash
# Build a release bundle for this machine:
#   dist/dgnv8w-<version>-<target>/      program, docs, examples, inspection scripts
#   dist/dgnv8w-<version>-<target>.tar.gz
# The bundle contains no VZD material; the ADTI seed is supplied by the user.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
target=$(rustc -vV | sed -n 's/^host: //p')
name="dgnv8w-$version-$target"
out="dist/$name"

cargo build --release --locked --quiet
exe=dgnv8w
[[ -f target/release/dgnv8w.exe ]] && exe=dgnv8w.exe

rm -rf "$out"
mkdir -p "$out/examples" "$out/tools"
cp "target/release/$exe" "$out/"
cp README.md LICENSE "$out/"
cp -R docs "$out/docs"
cp research/tools/*.py research/tools/README.md "$out/tools/"

# Examples: seeds sit next to the jobs in the bundle.
cp examples/README.md "$out/examples/"
cp crates/dgnv8-writer/tests/data/gdal_test_dgnv8.dgn \
   crates/dgnv8-writer/tests/data/LICENSE.GDAL.txt "$out/examples/"
sed -e 's|"../crates/dgnv8-writer/tests/data/gdal_test_dgnv8.dgn"|"gdal_test_dgnv8.dgn"|' \
    examples/gdal_example.json > "$out/examples/gdal_example.json"
sed -e 's|"../private/adti_seed.dgn"|"adti_seed.dgn"|' \
    -e 's|"../private/adti_example.dgn"|"adti_example.dgn"|' \
    examples/adti_example.json > "$out/examples/adti_example.json"

mkdir -p "$out/research"
cp research/EXP-*.md "$out/research/"

commit=$(git rev-parse HEAD 2> /dev/null || echo unknown)
cat > "$out/BUNDLE.md" <<BUNDLE
# dgnv8w $version for $target

Built from commit \`$commit\` of https://github.com/raitisx/dgn_v8_writer.

| Path | Contents |
| --- | --- |
| \`$exe\` | The writer. \`$exe inspect seed.dgn\` shows what it sees in a seed; \`$exe build job.json\` writes a DGN. |
| \`docs/HANDOVER.md\` | Start here: status, how to use and continue the project |
| \`docs/JOB_FORMAT.md\` | Every job option |
| \`examples/\` | Two jobs; the GDAL one runs as is, the ADTI one needs your ADTI seed as \`examples/adti_seed.dgn\` |
| \`tools/\` | Python inspection scripts (\`tools/README.md\`) |
| \`docs/\`, \`research/\` | Format rules with their evidence, and the experiment records they cite |

No VZD material is included. The ADTI seed is supplied by the user.
BUNDLE
tar -czf "dist/$name.tar.gz" -C dist "$name"
echo "wrote dist/$name.tar.gz"

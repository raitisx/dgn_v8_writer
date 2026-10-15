#!/usr/bin/env bash
# Rebuild the MicroStation test files from their jobs and check their hashes.
#
#   tools/build_testfiles.sh          round 1 (GDAL sample): must match testfiles/SHA256SUMS
#   tools/build_testfiles.sh --adti   also rounds 2-4 on private/adti_seed.dgn: must match
#                                     testfiles/adti_jobs/SHA256SUMS (the files tested in
#                                     research/EXP-0004..EXP-0006)
#
# CI runs the first form. The second needs the empty ADTI seed (E1 of EXP-0003),
# which is not in this repository.
set -euo pipefail
cd "$(dirname "$0")/.."

sha256() {
  if command -v sha256sum > /dev/null; then sha256sum "$@"; else shasum -a 256 "$@"; fi
}

cargo build --release --quiet
bin=target/release/dgnv8w

for job in testfiles/jobs/*.json; do "$bin" build "$job" > /dev/null; done
(cd testfiles && sha256 -c --quiet SHA256SUMS)
echo "round 1: every file matches testfiles/SHA256SUMS"

if [[ "${1:-}" == "--adti" ]]; then
  seed=private/adti_seed.dgn
  if [[ ! -f "$seed" ]]; then
    echo "missing $seed: copy the empty ADTI seed (E1 of EXP-0003) there" >&2
    exit 1
  fi
  expected=431af1fd0b043baaa90832bfa46dca3028796c8f6be5f4f556de2a44c7a757c8
  if [[ "$(sha256 "$seed" | cut -d' ' -f1)" != "$expected" ]]; then
    echo "note: $seed is not the seed used in EXP-0004..0006, so hashes will differ" >&2
  fi
  mkdir -p private/round2
  for job in testfiles/adti_jobs/*.json; do "$bin" build "$job" > /dev/null; done
  (cd private/round2 && sha256 -c ../../testfiles/adti_jobs/SHA256SUMS)
fi

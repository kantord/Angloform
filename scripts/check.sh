#!/usr/bin/env bash
# The repo's invariants, enforced. Run locally before committing, and in CI.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== data present + checksums =="
./scripts/fetch-data.sh

echo "== no conflict-silencing in the grammar =="
# ADR 0014 §guarantee: the one-parse proof holds only while the grammar has
# zero precedence/assoc annotations — conflicts must be designed away.
if grep -nE "#\[precedence|assoc" crates/grammar/src/angloform.lalrpop; then
    echo "ERROR: precedence annotations found — ambiguity may be silently resolved"
    exit 1
fi

echo "== tests (morphology, corpus snapshots, banned structures) =="
cargo test --workspace --quiet

echo "== regenerate everything =="
cargo run -q -p lexgen
cargo run -q -p grammar
cargo run -q -p textcost
cargo run -q -p textcost -- corpus/dogfood-pairs.tsv docs/dogfood-cost-report.md
cargo run -q -p triage
./scripts/showcase.sh > /dev/null
./scripts/coherence.sh > /dev/null

echo "== markdown block parser (docs/markdown-linting.md) =="
python3 scripts/test-mdblocks.py

echo "== no homophone collisions among seed/definitions words (real IPA, sampled dialects) =="
python3 scripts/homophone-check.py --seed-def-only

echo "== no same-synset redundancy among seed/definitions words (real WordNet synsets) =="
python3 scripts/redundancy-check.py --curated-only

echo "== web playground (wasm, typecheck, unit + e2e tests) =="
(
    cd web
    pnpm install --frozen-lockfile
    bash scripts/build-wasm.sh
    python3 ../scripts/build-dictionary-data.py
    pnpm typecheck
    pnpm test
    # browser binaries only, no --with-deps (that needs sudo — fine in
    # CI, which installs OS packages as its own step; a dev machine
    # already has them, or playwright will say so clearly)
    pnpm exec playwright install chromium
    pnpm test:e2e
)

echo "== committed artifacts must match their sources =="
if ! git diff --exit-code -- lexicon.tsv docs/lexicon-report.md CONTEXT.md \
    docs/parse-report.md docs/cost-report.md docs/dogfood-cost-report.md \
    docs/triage-report.md docs/showcase.md docs/coherence-report.md; then
    echo "ERROR: generated files drifted from their sources — commit the regenerated versions"
    exit 1
fi

echo "all checks passed ✓"

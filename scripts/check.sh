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

echo "== markdown block parser (docs/markdown-linting.md) =="
python3 scripts/test-mdblocks.py

echo "== committed artifacts must match their sources =="
if ! git diff --exit-code -- lexicon.tsv docs/lexicon-report.md CONTEXT.md \
    docs/parse-report.md docs/cost-report.md docs/dogfood-cost-report.md \
    docs/triage-report.md docs/showcase.md; then
    echo "ERROR: generated files drifted from their sources — commit the regenerated versions"
    exit 1
fi

echo "all checks passed ✓"

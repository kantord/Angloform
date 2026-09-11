#!/usr/bin/env python3
"""Generate web/src/lib/dictionary-data.json from seed/definitions/*.yaml —
the data source for the dictionary page (web/dictionary.html). Run before
`vite build`/`vite dev` (wired into web/package.json's build/dev scripts),
same pattern as web/scripts/build-wasm.sh generating the wasm glue.

Every word here has a checked definition (ADR 0061, parse_definition).
Most of the lexicon is NOT included — bare seed/seed.json entries have no
definition text at all (see docs/vocab-ratchet-findings-2026-09-11.md).
"""
import glob
import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

try:
    import yaml
except ImportError:
    raise SystemExit("PyYAML is required: pip install pyyaml")

entries = []
for path in sorted(glob.glob(f"{ROOT}/seed/definitions/*.yaml")):
    lemma = os.path.basename(path)[:-5]
    with open(path) as f:
        data = yaml.safe_load(f)
    entries.append({
        "lemma": lemma,
        "category": data["category"],
        "definition": data["definition"],
    })

out_path = f"{ROOT}/web/src/lib/dictionary-data.json"
with open(out_path, "w") as f:
    json.dump(entries, f, indent=2)
    f.write("\n")

print(f"wrote {len(entries)} entries to {os.path.relpath(out_path, ROOT)}")

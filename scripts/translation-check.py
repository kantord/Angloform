#!/usr/bin/env python3
"""Validate docs/translations/*.yaml — the per-sense English->Angloform
dictionary for rejected/not-yet-enabled words.

Shape mirrors docs/translations.schema.json (kept as the canonical,
human-readable spec — hand-checked here rather than via the `jsonschema`
package, which isn't a project dependency and CI has no pip-install step
for; this project's own convention, matching scripts/build-dictionary-data.py,
is PyYAML only).

Two checks, matching the shape/grammar-vs-schema split scripts/word-check.py
and lexgen already use elsewhere in this project:
  1. Structural: every file matches the documented shape.
  2. Real: every `redirect.word` resolves to an actually-enabled Angloform
     lemma, and its `category` matches that lemma's real Category —
     unverified documentation is exactly what ADR 0060 exists to catch.

Usage:
  python3 scripts/translation-check.py            # validate everything
  python3 scripts/translation-check.py look        # validate one file
"""
import glob
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
DIR = ROOT / "docs" / "translations"

ANSWER_REQUIRED = {
    "redirect": {"word", "category"},
    "construction": {"pattern", "note"},
    "gap": set(),
    "excluded": {"reason"},
}


def load_enabled_categories():
    """surface -> Angloform Category (the form-tag family), for redirect
    validation. Uses lexicon.tsv's own tag, not seed source category
    strings, so it reflects what's REALLY enabled right now."""
    cat_of = {}
    with open(ROOT / "lexicon.tsv") as f:
        for line in f:
            if line.startswith("#"):
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) == 4 and parts[1] == "form":
                surface, _, tag, _ = parts
                cat_of[surface] = tag
    return cat_of


def check_shape(path, data):
    """Returns a list of shape errors — pure structural checks, no I/O."""
    errs = []
    if not isinstance(data, dict):
        return [f"{path}: top level must be a mapping"]

    status = data.get("status")
    if status in ("exempt", "excluded"):
        if not data.get("reason"):
            errs.append(f"{path}: status {status!r} needs a non-empty `reason`")
        if status == "exempt" and data.get("exempt_kind") not in ("garbage", "proper-noun"):
            errs.append(f"{path}: exempt needs `exempt_kind`: \"garbage\" or \"proper-noun\"")
        extra = set(data) - {"status", "reason", "exempt_kind"}
        if extra:
            errs.append(f"{path}: unexpected field(s) on a whole-lemma verdict: {sorted(extra)}")
        return errs

    if status is not None:
        errs.append(f"{path}: unknown top-level `status` {status!r} (use by_upos instead, or exempt/excluded)")
        return errs

    by_upos = data.get("by_upos")
    if not isinstance(by_upos, dict) or not by_upos:
        errs.append(f"{path}: needs a non-empty `by_upos` mapping (or a whole-lemma `status`)")
        return errs
    extra = set(data) - {"by_upos"}
    if extra:
        errs.append(f"{path}: unexpected top-level field(s): {sorted(extra)}")

    for upos, block in by_upos.items():
        if not isinstance(block, dict) or set(block) != {"senses"}:
            errs.append(f"{path}: by_upos.{upos} must be exactly {{senses: [...]}}")
            continue
        senses = block["senses"]
        if not isinstance(senses, list) or not senses:
            errs.append(f"{path}: by_upos.{upos}.senses must be a non-empty list")
            continue
        for i, sense in enumerate(senses):
            tag = f"{path}: {upos}.senses[{i}]"
            if not isinstance(sense, dict) or "gloss" not in sense or "angloform" not in sense:
                errs.append(f"{tag}: needs `gloss` and `angloform`")
                continue
            if not sense["gloss"]:
                errs.append(f"{tag}: `gloss` must be non-empty")
            ans = sense["angloform"]
            if not isinstance(ans, dict) or "status" not in ans:
                errs.append(f"{tag}.angloform: needs a `status`")
                continue
            astatus = ans["status"]
            if astatus not in ANSWER_REQUIRED:
                errs.append(f"{tag}.angloform: unknown status {astatus!r} (redirect/construction/gap/excluded)")
                continue
            missing = ANSWER_REQUIRED[astatus] - set(ans)
            if missing:
                errs.append(f"{tag}.angloform ({astatus}): missing field(s) {sorted(missing)}")
            extra = set(ans) - ANSWER_REQUIRED[astatus] - {"status"} - ({"note"} if astatus == "gap" else set())
            if extra:
                errs.append(f"{tag}.angloform: unexpected field(s) {sorted(extra)}")
    return errs


def main():
    only = sys.argv[1] if len(sys.argv) > 1 else None
    files = sorted(DIR.glob(f"{only}.yaml")) if only else sorted(DIR.glob("*.yaml"))
    if not files:
        print(f"no translation files found{' for ' + only if only else ''}", file=sys.stderr)
        return 1

    cat_of = load_enabled_categories()
    errors = []

    for path in files:
        rel = path.relative_to(ROOT)
        try:
            data = yaml.safe_load(path.read_text())
        except yaml.YAMLError as e:
            errors.append(f"{rel}: invalid YAML: {e}")
            continue

        shape_errs = check_shape(rel, data)
        errors.extend(shape_errs)
        if shape_errs:
            continue

        if data.get("status") in ("exempt", "excluded"):
            continue

        for upos, block in data["by_upos"].items():
            for i, sense in enumerate(block["senses"]):
                ans = sense["angloform"]
                if ans["status"] != "redirect":
                    continue
                word = ans["word"]
                if word not in cat_of:
                    errors.append(
                        f"{rel}: {upos}.senses[{i}] redirects to {word!r}, "
                        f"which is not an enabled Angloform word"
                    )
                    continue
                real_cat = cat_of[word]
                # a redirect's declared category is the *Category* family
                # (e.g. "VERB_TRANS"), not always the exact Form Tag on the
                # base surface (which could be e.g. VERB_TRANS_BASE) — match
                # by prefix, same lenience the lexicon-report tooling uses.
                declared = ans["category"]
                if not real_cat.startswith(declared.split("_BASE")[0].split("_SG")[0]):
                    errors.append(
                        f"{rel}: {upos}.senses[{i}] declares {word!r} as "
                        f"{declared}, but it's really enabled as {real_cat}"
                    )

    if errors:
        print(f"FAIL: {len(errors)} error(s):\n", file=sys.stderr)
        for e in errors:
            print(f"  {e}", file=sys.stderr)
        return 1

    print(f"{len(files)} translation file(s) checked, all clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())

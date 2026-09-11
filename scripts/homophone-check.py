#!/usr/bin/env python3
"""Homophone collision check across real English dialects, via espeak-ng.

Usage: python3 scripts/homophone-check.py [--seed-def-only]

For every enabled Angloform Surface Form (lexicon.tsv — base forms AND
inflections: "rights"/"writes" is exactly as real a collision as
"right"/"write"), gets a real IPA transcription from espeak-ng in each
of a sample of English dialect voices, strips stress marks (ˈ primary,
ˌ secondary — two words that differ only in stress placement, e.g.
"project" noun/verb, are still treated as a collision: ESL speakers and
several dialects don't reliably distinguish stress-only minimal pairs —
decided in the homophone-check grilling, 2026-09-11), and reports every
pair of *differently-spelled* forms whose IPA is identical in at least
one sampled dialect (same-spelling collisions are ADR 0001's job, not
this script's).

CARRIER-PHRASE, not bare-word: espeak-ng is context-sensitive for
English heteronyms ("the project" -> PROject, "to project" -> proJECT,
a genuinely different vowel, not just stress) — querying a word bare
silently picks whichever reading espeak defaults to, which may not be
the sense Angloform actually enabled for that word. Every Surface Form
is instead put in a carrier phrase matching its real Form Tag before
extracting its IPA (see CARRIER below); unmapped tags fall back to a
generic, verified-neutral "I said X" carrier.

Scope (per the homophone-check grilling, 2026-09-11): retroactive over
the whole enabled lexicon, but only seed/definitions/*.yaml words are
"obligated" to be collision-free — seed.json legacy words still count
as occupying their sound (block new collisions) but are not themselves
required to fix anything. Pass --seed-def-only to only report
collisions that touch at least one seed/definitions word (checked by
LEMMA, so an inflected form of a seed/definitions word still counts).

DIALECT SAMPLE — not exhaustive by design (this session's own finding:
none of the currently-installed espeak-ng voices cover Southern-
Hemisphere or South Asian English; only what ships in this espeak-ng
build's data is used). Extending the sample is cheap (add a voice ID)
should broader coverage matter later.

ESL/L1-TRANSFER LAYER (--esl, experimental, 2026-09-11): a deliberately
simplistic *coarsening* of the dialect-layer IPA, not a second espeak
query or a real learner-accent model — cheap to reason about, cheap to
extend. Six well-documented, common L1-transfer mergers, each verified
against a real minimal pair before trusting it (see this script's
revision history): θ/t (dental fricative gap — French, German, most
Slavic, Mandarin, Japanese, ...), ð/d (same gap, voiced), ɹ/l (famous
for Japanese/Korean and some Sinitic L1s), v/w (German, Hindi, Slavic),
ɪ/iː and ʊ/uː (many languages lack English's tense/lax vowel-length
contrast: "ship"/"sheep", "full"/"fool"), æ/ɛ ("bad"/"bed"), plus
word-final obstruent devoicing (German, Russian, Polish, Turkish,
Dutch, ...: "bag"→"back"). Deliberately NOT exhaustive (picks one
canonical confusion target per phoneme rather than modeling every
attested substitution — e.g. θ merges with /t/ here, not also /s/,
even though both are real) and NOT position-sensitive except for
devoicing, which genuinely needs it (the earlier dialect-layer T-
flapping bug — unconditional substitution — taught that lesson).
Reported as a clearly separate section, never mixed into the base
dialect collision count or the --seed-def-only build gate.
"""
import subprocess
import sys
import re
import glob
from collections import defaultdict

ROOT = "/home/kantord/repos/minglish"

DIALECTS = [
    "en-us", "en-gb", "en-gb-x-rp", "en-gb-scotland",
    "en-gb-x-gbclan", "en-gb-x-gbcwmd", "en-029", "en-us-nyc",
]

STRESS_RE = re.compile(r"[ˈˌ]")

# --esl: word-final devoicing pairs (voiced -> voiceless), applied to
# only the LAST phoneme of the (already stress-stripped) IPA string —
# unconditional/position-free application was a real bug found and
# fixed in the dialect layer (T-flapping), not repeated here.
ESL_FINAL_DEVOICE = {"b": "p", "d": "t", "ɡ": "k", "v": "f", "z": "s", "dʒ": "tʃ", "ð": "θ"}

# --esl: symbol groups collapsed to one canonical symbol each, applied
# globally (not position-restricted) — a whole-word L1-transfer effect,
# not a positional one, so global application is the correct scope here
# (unlike devoicing above).
ESL_MERGE_GROUPS = [
    (["θ", "t"], "T"),
    (["ð", "d"], "D"),
    (["ɹ", "l"], "R"),
    (["v", "w"], "V"),
    (["ɪ", "i"], "I"),   # note: applied after length marks are stripped
    (["ʊ", "u"], "U"),
    (["æ", "ɛ"], "E"),
]


def esl_normalize(ipa):
    for voiced, voiceless in ESL_FINAL_DEVOICE.items():
        if ipa.endswith(voiced):
            ipa = ipa[:-len(voiced)] + voiceless
            break
    ipa = ipa.replace("ː", "")
    for symbols, canon in ESL_MERGE_GROUPS:
        for s in symbols:
            ipa = ipa.replace(s, canon)
    return ipa


# --esl --spelling: "spelling pronunciation" — a learner reads a silent
# letter aloud instead of dropping it (well documented for L2 English:
# "wr-"/"kn-" clusters, silent final consonants, etc.). Each entry:
# (spelling regex, phoneme(s) to insert, insertion point relative to the
# IPA — "start"/"end" or "before:<symbol>"/"after:<symbol>" to anchor
# next to a specific existing phoneme). Deliberately only the patterns
# actually found in the current lexicon (checked directly, not a
# textbook-exhaustive list) — see this script's revision history.
SPELLING_PRONOUNCE = [
    (re.compile(r"^wr"), "w", "start"),
    (re.compile(r"^kn"), "k", "start"),
    (re.compile(r"gn$"), "ɡ", "before-last"),
    (re.compile(r"bt$"), "b", "before-last"),
    (re.compile(r"lf$"), "l", "before-last"),
    (re.compile(r"^honest"), "h", "start"),
]


def spelling_pronounced_variant(surface, ipa):
    """Returns the spelling-pronounced IPA variant for this surface form,
    or None if no pattern matches. "before-last" inserts right before
    the IPA's final character (the silent consonant's neighbor)."""
    for pattern, phoneme, where in SPELLING_PRONOUNCE:
        if pattern.search(surface):
            if where == "start":
                return phoneme + ipa
            if where == "before-last":
                return ipa[:-1] + phoneme + ipa[-1]
    return None


# Form Tag -> (carrier template, index of the target word's IPA token
# within the phrase's output line, split on whitespace). Every template
# spot-checked against real espeak-ng output before trusting it (see
# this script's revision history). Unmapped tags fall back to CLOSED's
# generic carrier — English closed-class words don't have a noun/verb-
# style heteronym split, so one neutral carrier covers all of them.
CARRIER = {
    "NOUN_SG": ("the {w}", 1),
    "NOUN_PL": ("the {w}", 1),
    "VERB_TRANS_BASE": ("to {w} it", 1),
    "VERB_INTRANS_BASE": ("to {w}", 1),
    "VERB_TRANS_3SG": ("it {w} it", 1),
    "VERB_INTRANS_3SG": ("it {w}", 1),
    "VERB_TRANS_ED": ("it {w} it", 1),
    "VERB_INTRANS_ED": ("it {w}", 1),
    "VERB_TRANS_ING": ("it is {w} it", 2),
    "VERB_INTRANS_ING": ("it is {w}", 2),
    "ADJ": ("very {w}", 1),
    "ADJ_LONG": ("very {w}", 1),
    "ADJ_CMP": ("it is {w}", 2),
    "ADJ_SUP": ("it is the {w}", 3),
    "CLOSED": ("I said {w}", 2),
}


def load_enabled_forms():
    """surface -> (lemma, Form Tag), from every lexicon.tsv "form" row.
    lexicon.tsv's own header: "surface  kind  tag  value(=lemma)" —
    column 0 is the surface form, column 3 is the lemma, NOT the other
    way around (a real bug in an earlier version of this script)."""
    forms = {}
    with open(f"{ROOT}/lexicon.tsv") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 4 or parts[1] != "form":
                continue
            surface, _, tag, lemma = parts
            if surface.islower() and surface.isalpha():
                forms[surface] = (lemma, tag)
    return forms


def ipa_for_dialect(forms, voice, batch_size=200):
    """forms: {surface: (lemma, tag)}. Returns {surface: ipa}, one
    espeak-ng call per batch (comma-joined carrier phrases -> one output
    line each), verifying line count == batch size; falls back to
    one-at-a-time for any batch that doesn't line up."""
    items = list(forms.items())
    result = {}
    for i in range(0, len(items), batch_size):
        batch = items[i:i + batch_size]
        templates = [CARRIER.get(tag, CARRIER["CLOSED"]) for _, (_, tag) in batch]
        phrases = [tmpl.format(w=w) for (w, _), (tmpl, _) in zip(batch, templates)]
        out = subprocess.run(
            ["espeak-ng", "-v", voice, "--ipa", "-q", ", ".join(phrases)],
            capture_output=True, text=True, timeout=60,
        ).stdout.splitlines()
        if len(out) == len(batch):
            for (w, _), (_, idx), line in zip(batch, templates, out):
                tokens = line.split()
                if idx < len(tokens):
                    result[w] = STRESS_RE.sub("", tokens[idx])
        else:
            for (w, (lemma, tag)), (tmpl, idx) in zip(batch, templates):
                one = subprocess.run(
                    ["espeak-ng", "-v", voice, "--ipa", "-q", tmpl.format(w=w)],
                    capture_output=True, text=True, timeout=10,
                ).stdout.split()
                if idx < len(one):
                    result[w] = STRESS_RE.sub("", one[idx])
    return result


def main():
    import shutil
    if shutil.which("espeak-ng") is None:
        print(
            "espeak-ng is required (apt: espeak-ng, arch: espeak-ng) — "
            "not found on PATH",
            file=sys.stderr,
        )
        sys.exit(2)

    seed_def_only = "--seed-def-only" in sys.argv
    esl = "--esl" in sys.argv

    seed_def = {p.split("/")[-1][:-5] for p in glob.glob(f"{ROOT}/seed/definitions/*.yaml")}

    forms = load_enabled_forms()
    print(f"{len(forms)} enabled surface forms, {len(DIALECTS)} dialects", file=sys.stderr)

    collisions = defaultdict(set)  # frozenset({w1, w2}) -> set of dialects
    esl_collisions = defaultdict(set)  # frozenset({w1, w2}) -> set of dialects (ESL-merged)
    spelling_collisions = defaultdict(set)  # frozenset({w1, w2}) -> set of dialects

    for voice in DIALECTS:
        print(f"  querying {voice}...", file=sys.stderr)
        ipa = ipa_for_dialect(forms, voice)

        by_ipa = defaultdict(list)
        for w, ph in ipa.items():
            by_ipa[ph].append(w)
        for ph, ws in by_ipa.items():
            ws = sorted(set(ws))
            if len(ws) < 2:
                continue
            for i in range(len(ws)):
                for j in range(i + 1, len(ws)):
                    if ws[i] != ws[j]:  # same-spelling collision is ADR 0001's job
                        collisions[frozenset((ws[i], ws[j]))].add(voice)

        if esl:
            by_esl = defaultdict(list)
            for w, ph in ipa.items():
                by_esl[esl_normalize(ph)].append(w)
            for ph, ws in by_esl.items():
                ws = sorted(set(ws))
                if len(ws) < 2:
                    continue
                for i in range(len(ws)):
                    for j in range(i + 1, len(ws)):
                        pair = frozenset((ws[i], ws[j]))
                        # exclude: same spelling (ADR 0001's job), already
                        # caught by the dialect layer, or two inflections
                        # of the SAME lemma (e.g. hypothesis/hypotheses) —
                        # not a different-word confusion at all
                        if (ws[i] != ws[j] and pair not in collisions
                                and forms[ws[i]][0] != forms[ws[j]][0]):
                            esl_collisions[pair].add(voice)

            # spelling pronunciation: a word matching a silent-letter
            # pattern, pronounced WITH that letter, checked against
            # every other word's REAL (unmodified) pronunciation
            by_real = defaultdict(list)
            for w, ph in ipa.items():
                by_real[ph].append(w)
            for w, ph in ipa.items():
                variant = spelling_pronounced_variant(w, ph)
                if variant is None:
                    continue
                for other in by_real.get(variant, []):
                    if other == w:
                        continue
                    pair = frozenset((w, other))
                    if forms[w][0] != forms[other][0]:
                        spelling_collisions[pair].add(voice)

    reported = []
    for pair, dialects in sorted(collisions.items(), key=lambda kv: sorted(kv[0])):
        a, b = sorted(pair)
        lemma_a, lemma_b = forms[a][0], forms[b][0]
        if seed_def_only and lemma_a not in seed_def and lemma_b not in seed_def:
            continue
        reported.append((a, b, dialects))

    print(f"\n{len(reported)} homophone pairs (colliding in at least one of {len(DIALECTS)} dialects):\n")
    for a, b, dialects in reported:
        lemma_a, tag_a = forms[a]
        lemma_b, tag_b = forms[b]
        tag = " [seed/definitions]" if (lemma_a in seed_def or lemma_b in seed_def) else ""
        print(f"  {a} ({tag_a}) / {b} ({tag_b}){tag}  —  {', '.join(sorted(dialects))}")

    if esl:
        esl_reported = []
        for pair, dialects in sorted(esl_collisions.items(), key=lambda kv: sorted(kv[0])):
            a, b = sorted(pair)
            lemma_a, lemma_b = forms[a][0], forms[b][0]
            if seed_def_only and lemma_a not in seed_def and lemma_b not in seed_def:
                continue
            esl_reported.append((a, b, dialects))
        print(
            f"\n--- ESL/L1-transfer layer (experimental, informational only, "
            f"never gates the build) ---\n"
            f"{len(esl_reported)} additional pairs, on top of the {len(reported)} above:\n"
        )
        for a, b, dialects in esl_reported:
            lemma_a, tag_a = forms[a]
            lemma_b, tag_b = forms[b]
            tag = " [seed/definitions]" if (lemma_a in seed_def or lemma_b in seed_def) else ""
            print(f"  {a} ({tag_a}) / {b} ({tag_b}){tag}  —  {', '.join(sorted(dialects))}")

        spelling_reported = []
        for pair, dialects in sorted(spelling_collisions.items(), key=lambda kv: sorted(kv[0])):
            a, b = sorted(pair)
            lemma_a, lemma_b = forms[a][0], forms[b][0]
            if seed_def_only and lemma_a not in seed_def and lemma_b not in seed_def:
                continue
            spelling_reported.append((a, b, dialects))
        print(
            f"\n--- Spelling-pronunciation layer (a silent letter read aloud "
            f"instead of dropped — experimental, informational only) ---\n"
            f"{len(spelling_reported)} pairs:\n"
        )
        for a, b, dialects in spelling_reported:
            lemma_a, tag_a = forms[a]
            lemma_b, tag_b = forms[b]
            tag = " [seed/definitions]" if (lemma_a in seed_def or lemma_b in seed_def) else ""
            print(f"  {a} ({tag_a}) / {b} ({tag_b}){tag}  —  {', '.join(sorted(dialects))}")

    # gate mode: --seed-def-only is the enforcement invocation (scripts/
    # check.sh) — every seed/definitions/*.yaml word is obligated to be
    # collision-free (seed.json legacy words are not, per the homophone-
    # check grilling, 2026-09-11); a non-empty report there is a real
    # build failure, not just information. ESL layer never gates
    # (deliberately experimental/informational — grilling decision).
    if seed_def_only and reported:
        print(
            f"\nFAIL: {len(reported)} homophone collision(s) touch a seed/definitions "
            "word — every seed/definitions/*.yaml word must be phonetically distinct "
            "(homophone-check grilling, 2026-09-11)",
            file=sys.stderr,
        )
        sys.exit(1)


if __name__ == "__main__":
    main()

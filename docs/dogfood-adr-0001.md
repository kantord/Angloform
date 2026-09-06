# Dogfood: translating ADR 0001 into angloform

Sentence-by-sentence attempt to render our own first ADR in angloform
(2026-09-01, lexicon at 47+ lemmas, tier-1 grammar). Faithful translations
join `corpus/pairs.tsv`; blockers carry the reason codes from
`corpus/untranslatable.tsv`. Retroactive rewriting of the ADR itself is
sanctioned only when lossless — which, as the tally shows, is not yet
reachable for most of it.

Before/after previews with costs and parse guarantees for the faithful
pairs: **docs/dogfood-cost-report.md** (source: corpus/dogfood-pairs.tsv).

## Sentence audit

| ADR 0001 sentence (abridged) | verdict |
|---|---|
| "angloform needs a lexicon: every allowed surface form, each with exactly one form-tag…" | BLOCKED — QUANTIFIER (*every, each, exactly one*), COMPOUND (*form-tag*) |
| "Word choice must stay a human judgment." | ✓ translated (post-ADR 0013) — "people must choose words", drops: nominalization |
| "…exactly the kind of check humans miss and machines don't." | ✓ translated (post-ADR 0013) — "people do not find collisions. machines find collisions.", drops: nominalization |
| "A hand-edited seed/seed.json is the single source of truth." | ✓ translated (post-ADR 0018) — "people edit the file \"seed.json\". the file \"seed.json\" is the source of the truth.", drops: emphasis |
| "Both generated files are committed; they are…never hand-edited." | ✓ resolved by in-place rewrite (dogfood case adr0001-01): the flag was correct about the prose; the rewrite is itself valid angloform |
| "Reference data is used only for checking — never for choosing words." | ✓ resolved by in-place rewrite (case adr0001-03); only/never expressed as affirmative + generic-negation pair |
| "Curation stays human and cheap: one JSON entry per word…" | ✓ resolved by in-place rewrite (case adr0001-02); all rewrite sentences valid angloform |
| "Every curation commit shows its effect in the diffs." | ✓ translated (post-ADR 0014/0015) — "every commit shows the effect of the commit in the diffs", drops: nominalization |

## What technical prose needs, by blocking frequency

0. **GENERIC** — generic reference (bare plurals: *humans, machines*;
   process nominals: *word choice*). Mandatory determiners removed English's
   genericity device, so every translation attempt forces a false definite
   or an ambiguous indefinite. Caught only by human review — no metric sees
   semantic reference ambiguity. Kin to QUANTIFIER (both are quantification).
1. **QUANTIFIER** (*every, each, both, only, single, exactly one*) — in 5 of 8
   sentences. Now the top-blocking feature across every corpus we have.
2. **COMPOUND** (noun-noun: *form-tag, seed list, reference data, build
   output*) — 4 of 8. Technical prose is compound-dense; the research
   options were ban-and-rephrase (of-genitives) or hyphenate-as-lexical-entry.
3. **NOMINALIZATION** (*curation, word choice, judgment*) — verbal recast is
   the right direction, but recasting a generic nominal drags in the GENERIC
   problem above; safe only once genericity has a sanctioned form.
4. **PASSIVE / agentless statements**, **ANAPHORA** (*they, its*),
   **adverbs of quantification** (*never, only*) — the familiar residue.

## Verdict

**7 of 8 sentences resolved** (4 translated, 3 by in-place source rewrite) (progression: 2 → 0 after the
generic-reference retraction → 2 after ADR 0013 → 4 after names (ADR 0018),
every/one, and the transparency policy unblocked the seed.json and
curation-commit rows). **Reframe (2026-09-01): the goal is NOT to translate arbitrary English.**
angloform parses well-written text and flags problems in text that is not —
so each blocked row must first be judged: is the rejection the linter being
*right* about the prose? Verdicts on the remaining 4: the both/they/passive
semicolon splice and the "curation stays human and cheap" compression are
**legitimate style flags — the source sentences should be rewritten in
English first**, then translated. *never* already has a sanctioned
equivalent (generic negation: "people do not edit these files"). The only
residue that survives as a genuine gap candidate is ***only*** (a precision
device — though placement-ambiguous in English, so its design needs its own
interview). New language features require a *well-written* rejected
sentence as evidence, not coverage of the original prose. Self-hosting also exposed a blind
spot in the tooling itself: structural-ambiguity guarantees and cost metrics
are silent about semantic reference ambiguity — that check is human (or a
future semantic layer), and ADR 0012 review is where it happens.

## 2026-09-01: full rewrite — every sentence of ADR 0001 parses

The whole ADR was rewritten in place (68 sentences, 68 parse; meaning
review per ADR 0012 in corpus/dogfood-pairs.tsv, 15 pairs). At each
blocked sentence the choice was: fix a gap in the system, or apply the
linter's advice when it was right about the prose.

| Blocker | Choice | Result |
|---|---|---|
| Vocabulary (≈55 lemmas: *language, form, option, seed, lemma, category, replacement, waiver, contain, derive, migrate, prior, rare, unused, …*) | gap — curated into the seed, WordNet-attested, cross-POS handled | lexicon 224 → 373 forms |
| *43% of its swaps* | gap — proportion is propositional; "43 of 100" would invent a count | ADR 0024 (*percent*) |
| *every form of the file `lexicon.tsv`* | gap — appositive names were not allowed inside an of-PP | ADR 0018 extended |
| *commit* as a verb (case adr0001-05) | advice — the redirect (*save*) was right; *commit* stays a noun | "save … in the repository"; *save* enabled |
| *reports/checks* as verb/noun in the wrong slot | advice — redirects were right (*describe*, *test*); both were dormant on inflected forms (ideas, advice gap 2) | *describe* enabled |
| *rejected* as an adjective | advice by construction — collides with the verb form | *unused sense* |
| NP coordination (*a replacement or a waiver*), *every*-subject with a tail | advice — predicate coordination / indefinite subject | rewritten |
| parentheses, colons, semicolons, *e.g.*, *hence*, *~*, *at the same time* | advice — meta-prose devices; split into sentences | rewritten, drops declared |
| named options ("Option 3") | advice — ordinals deferred (ADR 0022) | `the option "seed"` (ADR 0018 appositive) |
| *the maintainers vendor the reference data* | the ADR was stale: CC BY-SA data is fetched, not vendored | rewritten to the current facts |

Open from this pass: *only* (case adr0001-04) is no longer used by the ADR
but stays a language-design gap; redirect targets outside the lexicon are
now listed in the lexicon report so the ADR 0023 hole is visible.

### Pair review (2026-09-01, ADR 0012 meaning check with the maintainer)

- Pair 8: *every unused sense has a replacement* overclaimed (waivers exist)
  → "a word can have an unused sense. the lexicon can contain a replacement
  for the unused sense".
- Pair 10: restated around *improvement* so the claim is readable without
  the source's jargon.
- Pair 11: generic statements had lost the cause → two conditionals;
  *silent gaps* restored as *hidden gaps*.
- Pairs 11/12: "5 forms" and "10 rules" were false precision → ADR 0025
  (*about* / *~*).
- Pair 14: the sentence had absorbed a parenthetical that belonged
  elsewhere → restated as drift + check + hand-edit failure.
- Pair 15: the causal link ("hence this ADR") has no angloform form. Declared
  as `causal`, a known tier-1 debt; causal connectives are now the head of
  the STATUS queue.
- Pair 13 (*commit* → *save … in the repository*) kept. The maintainer's
  note: the odd phrasing may be the better one; the real question is how a
  domain model would separate the noun and verb senses of *commit* under
  the one-tag rule (queue: ambitransitives).
- Pair 15 follow-up: ADR 0026 (causal connectives) closed the debt —
  "…grows with the seed, so the maintainers record the decision".

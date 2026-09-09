# Rebuilding Angloform's core vocabulary from a monosemy rule (design, 2026-09-09)

## Problem

The current vocabulary (`seed/seed.json`, ~789 entries, plus this session's
76-word `seed/definitions/*.yaml` migration) was built by iterative,
LLM-in-the-loop curation: propose a word, react to a person's feedback, adjust,
repeat. That process caught real problems when applied carefully (this
session's whole 10-round definition trial), but as the *sole* mechanism for
choosing which words even enter the vocabulary in the first place, it produced
a lexicon the maintainer now considers "completely garbage" — not because any
individual word is wrong, but because there was never a uniform, checkable
rule for *why a word belongs at all*.

A separate research thread (summarized, not reproduced in full here — see the
conversation this design came out of) tested an earlier, different automated
approach: swapping ambiguous common words for rarer, "more precise-sounding"
synonyms. Measured against a random-rare-word control, that swap algorithm
added no real disambiguation value beyond what picking any rare word would
give — except for one real, measured effect: it did genuinely reduce
cross-part-of-speech ambiguity. That finding motivates the rule below: don't
try to rank words by some notion of "precision" or rarity; use actual sense
counts per grammatical category, directly.

## This contradicts an already-accepted ADR — needs an explicit supersession

`data/README.md` states: "Used by `lexgen`/`triage` for **checking only** —
never for choosing words (see ADR 0001)." ADR 0001 (accepted) cites the *same*
rarity-swap research this doc's Problem section cites (43% of swaps didn't
reduce ambiguity, rarity explained the effect) and draws the opposite
conclusion from it: **"The choice of the words must stay with people."**

This design proposes fully mechanical word *admission* from WordNet sense
counts — the reverse of ADR 0001's decision, using the same evidence. That's
not automatically wrong (this design's mechanism — monosemy-by-category — is
different from anything ADR 0001 actually evaluated; ADR 0001 was about
picking *any* word for a concept, this is about admitting a word only when a
whole category is unambiguous for it), but it cannot happen silently. Before
building anything: **this needs a real ADR that explicitly supersedes ADR
0001** — stating what's actually changing (mechanical *candidate selection*,
with a human still reviewing every admit and writing every definition, is not
the same as "a machine picks the words" that ADR 0001 rejected) and why the
new mechanism avoids ADR 0001's original objection.

## The rule

For a candidate word, using WordNet's per-category sense counts:

1. Count senses separately for each part of speech the word is attested in
   (noun / verb / adjective / adverb).
2. The word's candidate category is whichever part of speech has **exactly
   one** sense.
3. **Fewer senses is what we want, not more.** A word with 5 noun senses is a
   *bad* noun candidate — using it as a noun still requires the reader to
   guess which of 5 meanings is intended, which is exactly the ambiguity this
   whole language exists to eliminate. A word with exactly 1 verb sense is an
   ideal verb candidate: using it as a verb carries zero disambiguation cost.
4. If no category has exactly 1 sense, the word doesn't qualify — deferred,
   not force-fit.
5. If *two or more* categories tie at exactly 1 sense (e.g. 1 noun sense and
   1 verb sense, both individually clean), the word is **also deferred**, not
   auto-admitted with a tiebreaker. Reasoning: a word offering two equally
   clean readings is itself a source of ambiguity — a reader needs context to
   know which was intended, even though each individual reading is
   unambiguous. Explicitly *not* discarded, though — this tied-word pool is
   flagged as a good hunting ground later, when a specific missing concept
   needs filling and a human can deliberately pick a winner informed by that
   need.
6. Threshold is strict (exactly 1, no "1–2 is close enough" tolerance) on
   purpose, for this first pass — loosening it is a decision to make *after*
   seeing how many words the strict rule actually admits, not before.
7. **A winning category whose closest runner-up is also low (2–3) is a
   weaker win than one with no real runner-up at all** — flagged, not
   rejected. Example, checked against real data: `boat` has noun=2, verb=1 —
   it passes the rule (exactly 1 verb sense) despite "boat" the noun being
   the word's overwhelmingly dominant real-world use. The rule can't see
   that; the report can. See Deliverable.
8. **The candidate pool excludes every lemma already present in `seed.json`,
   under *any* category** — not just the definitional ones — before the rule
   ever runs. Checked directly: without this filter, `not` (already `NEG`),
   `so` (already `RESULT`, ADR 0026), `who` (already the ADR 0059
   relativizer), and `an` (already `DET_SG`) all independently pass the
   monosemy rule and would show up as false "clean admits" — the rule has no
   way to know they already have a real grammatical job. (`who`'s single
   WordNet noun sense is almost certainly the WHO/World-Health-Organization
   acronym folded to lowercase — the proper-noun-folding noise named above,
   caught here concretely rather than left as an abstract caveat.)

**Expected outcome, stated up front so it isn't a surprise later**: common
words are *more* polysemous on average, not less (the well-known "Zipf's law
of meaning" effect). **Correction** (an earlier version of this doc claimed
"none of `run`/`make`/`go`/`get`/`take`/`be`/`have`/`do`/`say`/`see`/`want`/
`use`/`find`/`time`/`person`/`way`/`thing`/`man` hit exactly 1 sense in any
category" as directly-checked fact — re-verified against
`data/wordnet/index.*` and that's false: 7 of the 18 (`get`, `be`, `have`,
`say`, `see`, `way`, `go`) do hit exactly 1 in some category, each a real,
minor, mostly-obscure sense sitting alongside a dominant, heavily-polysemous
one). All 18 are already in `seed.json` regardless, so the exclusion filter
(step 8) would catch every one of them in a real run — the practical
conclusion holds even though the specific claim didn't. Restated more
honestly: common words *typically* have no clean winner, but a minority hit
one anyway, almost always in an obscure minor category alongside a dominant
one the rule can't see (see the "high-count runner-up" bucket below — this
is exactly that shape). The clean admits this rule produces will still skew
toward less common words overall; that's a real, structural consequence of
choosing monosemy over frequency/centrality — not a bug to fix, but
something to know going in.

**The "~200 from top 2000" original estimate was too low, not too high** —
a real run against the top 500 resolved lemmas (633 raw frequency rows
walked; 174 already-excluded as pre-existing `seed.json`/`seed/definitions/`
words, 83 unresolvable to any lemma) gave: 188 no-clean-winner, 33 tied-at-1,
28 clean-weak-margin, **77 clean-strong-margin**. That's 105/500 clean admits
— roughly a third of the non-excluded pool. Extrapolated to 2000 lemmas, this
plausibly lands in the several-hundreds, not a bare ~200-word foundation.

**But raw counts overstate quality — a hand-check of 20 "clean, strong"
admits from that run found 3 (15%) visibly wrong on inspection**, and all
three are real, distinct failure modes worth naming rather than waving away
as generic "noise":

- **Proper-noun folding produces real false admits, not just a theoretical
  risk.** `mr` passed clean-strong — a WordNet artifact of a name/title
  sense folded into the lowercase lemma, same mechanism as the `who`/WHO
  case above. At a measured ~15% junk rate in one small sample, this needs
  the report to visibly flag any candidate word matching common
  title/name/acronym patterns for extra scrutiny, not just a footnote.
- **A real lemmatizer bug, worth fixing before running this for real, not
  documenting as a known limitation.** `getting` passed clean-strong by
  resolving to its own rare noun sense in the WordNet index, instead of
  reducing to the verb `get` — because a naive lemmatizer can match a raw
  inflected wordform's own index entry before it ever checks whether that
  wordform is really an inflected form of something else. Fix: attempt
  de-inflection (via the `.exc` files, then regular-suffix stripping) first,
  and only fall back to treating the wordform as its own lemma if
  de-inflection finds nothing.
- **The `seed.json`-exclusion filter only catches words already curated
  somewhere — it can't catch words a design decision *elsewhere* already
  covers but that were never entered as a `seed.json` word.** `me` passed
  clean-strong (from an unrelated Maine/solfège WordNet sense) despite
  Angloform already case-collapsing `i`/`me` into one pronoun by design —
  that decision lives in the grammar, not as a `seed.json` entry, so the
  exclusion filter never sees it. No blanket fix for this class of gap;
  it's exactly what the human review pass is for. Named here so "the
  exclusion filter handles already-decided words" isn't overstated as
  complete.

**Explicitly out of scope for the mechanical rule**: transitive vs.
intransitive, for a word whose winning category is "verb." WordNet doesn't
distinguish this, and deriving it from another automated signal risks
reintroducing exactly the kind of unreliable automated judgment this reset is
reacting against. A human decides trans/intrans at the same moment they write
the word's real Angloform `definition` — the same moment they'd notice the
verb doesn't take an object.

## Word source

Top 2000 **unique lemmas** by frequency, not raw wordforms. `data/freq/en_zipf.tsv`
(already vendored) is a wordform list — `is`/`the`/`a`, and presumably
`run`/`runs`/`running`/`ran` as separate rows. Taking the top 2000 rows as-is
would waste slots on inflected duplicates of the same lemma. Instead: walk the
frequency list, map each wordform to its lemma (via WordNet's morphological
exception files — see Data below), dedupe, and keep going until 2000 distinct
lemmas are collected.

Not every wordform resolves to a WordNet lemma at all — checked directly
against the real top-2000 rows of `data/freq/en_zipf.tsv`: about 3% are
contractions (`don't`, `isn't`, `i'm`...) or abbreviation-like tokens (`u.s`),
none of which the `.exc` files (irregular *inflection* only) resolve. These
are skipped silently, not a failure — meaning the script walks noticeably
more than 2000 raw frequency rows to actually collect 2000 usable lemmas.
(Bare numerals like `1`/`2`/`3` are *not* in this unresolvable set, despite
looking like they should be — checked directly: `index.noun`/`index.adj`
carry literal cardinal-number entries, so numeral tokens resolve fine and
land in the ordinary tied-at-1 bucket, noun=1/adj=1.)

## Data (all already-licensed, some not yet vendored)

- `data/wordnet/index.{noun,verb,adj,adv}` — **already vendored.** Each row's
  3rd field is the sense count for that lemma in that category. This is
   the entire input the counting rule needs; no new fetch required for step 1.
  Two known-and-accepted sources of noise in this count, not fixed for v1:
  (a) WordNet folds proper-noun/instance senses into the lowercase lemma
  (verified: `index.noun` has zero capitalized entries), so a word's count
  can be inflated by name/place senses unrelated to ordinary usage; (b) raw
  synset count (`synset_cnt`) is a coarser signal than it looks — WordNet's
  sense-splitting granularity is uneven, so a "5-sense" word may in practice
  almost always mean one thing. Each row also carries `tagsense_cnt`
  (SemCor-tagged real-usage frequency per sense), a more directly relevant,
  already-available signal for "does a reader actually need to guess" —
  **deliberately not used for v1**, same reasoning as the strict threshold:
  ship the simplest version of the rule first, see how it performs, revisit
  only if the results call for it.
- `data/wordnet/{noun,verb,adj,adv}.exc` — **not yet vendored**, needed for
  correct lemmatization of irregular forms (`ran`→`run`, `better`→`good`).
  **Correction, checked directly against the live archives**: these are
  *not* in `WNdb-3.0.tar.gz` (the archive the vendored `index.*`/`data.*`
  files actually come from, per `data/README.md`) — that tarball contains
  only `index.*`/`data.*`. The `.exc` files live in the separate, full
  `WordNet-3.0.tar.gz` package (confirmed live, confirmed to contain
  `noun.exc`/`verb.exc`/`adj.exc`/`adv.exc`, license text identical to the
  vendored `data/wordnet/LICENSE`). Same project, same license, **different
  tarball** — the license claim holds, "just more files from the same
  archive" was wrong. Practically: `scripts/fetch-data.sh` has no existing
  WordNet-fetch code path to extend (`index.*` was vendored by hand, not by
  that script, unlike `data/ud/`/`data/freq/` which it does fetch) — this
  needs new fetch logic, not an addition to an existing one.
- `data/freq/en_zipf.tsv` — already vendored (wordfreq, CC BY-SA 4.0, already
  satisfied per `data/README.md`).

No new licenses, no new data sources outside this already-cleared set.

## Bootstrap / migration flow

`seed.json` becomes a shrinking, deprecated fallback vocabulary, not a
coexisting second source of truth:

1. Run the rule against the top 2000 lemmas → a candidate report (see
   Deliverable).
2. **Re-check the same rule against the 76 words already migrated into
   `seed/definitions/*.yaml` this session** — no grandfathering. A word
   survives only if it independently passes the same mechanical test with a
   matching category. This is the literal reset: even this session's own
   already-written, already-tested definitions don't get an exemption.
   **"Survives" means category-match only, not meaning-match** — the check
   confirms WordNet's single winning synset is the right *part of speech*,
   not that it's the same *sense* the human-written `definition` actually
   describes. We don't have WordNet's gloss text vendored (only `index.*`,
   not `data.*`), so an automated meaning-check isn't available; a category
   match is the only thing this step can mechanically verify. Real semantic
   drift between the curated definition and the winning synset stays a
   human-review concern, same as the rest of this design.

   **Expect heavy attrition here, not a near-total pass.** Spot-checked
   against the first 10 `seed/definitions/*.yaml` files alphabetically: only
   `algorithm` (noun=1) survives outright; `accept`, `agent`, `alone`,
   `arrive`, `assert`, `avoid`, `belong`, `build` all fail (no category hits
   exactly 1). This isn't a sign the check is broken — it's the reset
   working as designed, and the doc's earlier framing (a mostly-empty
   `seed/definitions/` at the start of the bootstrap) should be read
   literally, not as a worst case.

   One case needs its own bucket, not silent folding into "failed": a word
   whose rule-derived category *actively disagrees* with a deliberate
   existing decision, versus one that simply has no clean winner at all.
   Concretely: `author` is curated as `NOUN` with `VERB` explicitly rejected
   ("the verb sense is not enabled") — but WordNet gives noun=2 (fails) and
   verb=1 (passes cleanly), i.e. the rule's answer is exactly the category a
   human already rejected on purpose. That's a real disagreement worth a
   person's attention, not an absence of a signal — report it separately as
   **"rule disagrees with existing curation,"** distinct from "no clean
   category" and distinct from "clean pass."
3. Words that fail step 2 (currently in `yaml`, don't pass the check) drop
   back into `seed.json` — not deleted, since they're still needed as
   building-block vocabulary for writing the *surviving* words' definitions
   (a definition like "make a thing" needs "make" and "thing" to exist
   *somewhere*, even during a mostly-empty transition).

   **"Drop back into `seed.json`" is not yet a capability that exists —
   it needs new code, not a move of existing capability.** Checked against
   the real `crates/lexgen` code: `to_seed_entries()`
   (`crates/lexgen/src/definitions.rs`) only converts `DefinitionEntry →
   SeedEntry` in memory, one direction; nothing serializes a `SeedEntry`
   back out to `seed.json`'s JSON format. This also isn't a lossless round
   trip once it exists: `SeedEntry.reject` can only hold one redirect word
   per rejected category (`crates/lexgen/src/seed.rs`), so a
   `seed/definitions/` entry with multiple `redirects` collapses to the
   first one on demotion; `SeedEntry` has no `antonym` field at all, so that
   link is dropped entirely. **Accepted as-is**: `seed.json` is the
   deprecated, shrinking format — losing a second redirect option and an
   antonym link on the way back down to it is a reasonable simplification
   for words already on their way to eventual deletion, not a defect to fix
   by extending `seed.json`'s schema further.
4. Going forward, the flow is one-directional: **a word entering
   `seed/definitions/` is removed from `seed.json` in the same move.** Never
   both places at once — mechanically caught today, though not by the check
   this doc originally named: the duplicate-lemma-*and*-category check
   (`main.rs`) wouldn't by itself catch the same lemma enabled under two
   *different* categories in the two sources, but the separate surface-form
   collision check (both categories push the bare lemma spelling as a form)
   catches it anyway. Outcome holds; the earlier "mechanically enforced by
   the duplicate check" framing was the wrong mechanism, not a wrong
   conclusion.
5. Eventually — not scoped as part of *this* pass — once nothing in
   `seed/definitions/` still depends on a given `seed.json` word (i.e. no
   surviving definition's text/advice/redirects reference it), that word can
   be deleted outright. Detecting "nothing depends on it anymore" needs its
   own small dependency-tracking tool; not built yet, named here so it isn't
   lost.
6. **`seed.json` does not shrink to zero, ever — only its definitional-category
   share does.** WordNet's `index.*` files cover only noun/verb/adj/adv. This
   project's closed-class vocabulary — determiners, prepositions, the
   copula, modals, conjunctions, negation, quantifiers, and similar (checked
   against real `seed.json` category strings, e.g. `DET`, `PREP_V`, `BE`,
   `COPULA_SG`/`PL`, `NEG_AUX_BASE`/`PAST`, `QUANT_UNIV`, `QUANT_NEG`, and
   more — illustrative, not an exhaustive list) — has no mechanical rule
   that could ever categorize it, and
   stays permanently resident in `seed.json` (or wherever closed-class
   vocabulary ends up living — a separate question, out of scope here). The
   "eventually deleted outright" language in step 5 applies only to
   definitional-category (`NOUN`/`VERB_TRANS`/`VERB_INTRANS`/`ADJ`) entries,
   never to the whole file.

## Tooling

One-off Python script (not a new Rust crate/binary). This is a single
research/reporting pass whose real deliverable is its *output* (the
candidate report), not a permanent, repeatedly-run part of the toolchain —
unlike `lexgen`, which runs on every build. If this needs to run again
regularly (e.g. expanding past top-2000 later), that's the point to promote
it into Rust.

## Deliverable (first iteration)

A single Markdown report, sorted by frequency (most common first), with:

- **Clean admits, strong** — lemma, winning category, its sense count
  (always 1), runner-up category + count, frequency rank. "Strong" = no
  other category has more than 1 sense either (the word is essentially
  unattested outside its winning category).
- **Clean admits, weak margin** — same shape, but **any** other category has
  more than 1 sense (no fixed cutoff — 2 or 200 both qualify). Two real
  examples this covers: `boat` (noun=2, verb=1 — a nearby, plausible-sounding
  runner-up) and `be` (noun=1, verb=13 — the word's real everyday usage is
  overwhelmingly a *different* category than the one it technically won,
  arguably the riskier case of the two, since nothing about "exactly 1" by
  itself flags it). Passes the rule either way, but flagged for a closer
  human look before treating it as settled — a losing category with *any*
  real presence is a signal the mechanical rule can't act on but a person
  can.
- **Deferred: tied at 1** — same shape, flagging which categories tied. This
  is the "good raw material for later gap-filling" pool.
- **Deferred: no category at 1** — everything else, with full per-category
  sense-count breakdown, for reference.
- A one-line summary: N words evaluated, X clean admits (strong/weak split),
  Y tied-deferred, Z no-clean-winner.

Separately: a cross-reference of the clean-admit list against the existing 76
`seed/definitions/*.yaml` words, in three buckets — **survives** (matching
category), **fails, no clean winner** (demoted to `seed.json`), and **fails,
rule disagrees with existing curation** (the `author`-style case — demoted
the same way, but flagged distinctly since it's a real conflict, not an
absence of signal).

## Execution order (confirmed in the final grilling round)

1. Fetch `.exc` files, build the script, run a **500-lemma pilot first** —
   not the full 2000 — to catch any remaining implementation mistakes on a
   report small enough to read end-to-end (this is exactly how round 3's
   own lemmatizer bug and junk-rate findings were found).
2. **No stoplist for known junk patterns** (proper-noun folding, the `me`
   case) this round — rely on human review of the report. Revisit only if
   the junk rate on a real full run turns out much higher than the small
   sample suggested.
3. **Run the tool and produce the report before writing the ADR-0001
   supersession**, not after. The report is data, not a Seed change; the
   ADR is only needed once results get acted on (words start actually
   moving into `seed/definitions/`), and should be grounded in real numbers
   from this mechanism, not written speculatively in advance.

## Explicitly deferred (named so they aren't silently dropped)

- Loosening the "exactly 1" threshold, if the strict rule admits far fewer
  than expected.
- A tiebreaker rule for words tied at 1 across multiple categories — for now,
  human-decided, case by case, when a specific gap needs filling.
- Auditing/expanding past the top 2000 lemmas.
- The dependency-tracking tool that would let `seed.json` shrink to zero.
- Promoting the Python script into the Rust toolchain.

# Readiness: how close is angloform to usable, and how to get there

Answers "how close are we?" with measured evidence, not impression, and
turns that into an ordered plan. Written 2026-09-05. Update this file
(don't leave it to rot like the pre-2026-09-05 README.md did) whenever a
condition's status changes.

**"Usable" defined**: someone outside this project's own authorship —
human or a different LLM with no special coaching — can pick up the
onboarding doc, write a real technical document (a spec, an ADR, a
plan) in angloform, have it read back naturally by another such person,
and get unblocked quickly when the linter rejects something. Five
conditions, each with current measured status.

## Condition 1: grammar/vocabulary expresses real content without hitting a wall

**Status: met, for the register tested.** `docs/dogfood-sweep.md`:
every ADR 0001–0041 (the project's own real technical prose, not toy
sentences) parses 100% (1337/1337). `docs/language-gaps.md`'s full
ranked list from the 2026-09-03 naturalness pass is fully closed —
clause-level "but", causal/conditional-internal coordination,
"without", coordinated NPs (via the colon-list), all temporal/hedge
adverbs, "only", "other" — see the ADR 0037–0049 chain. This session's
`finding-frequency` tool (`docs/finding-frequency-report.md`) confirms
it from the opposite direction: the *generic* "no explanation"
fallback fires on only 0.4% of 6643 real near-miss sentences, meaning
almost every rejection already has a specific, named cause — not "the
grammar can't express this," but "the writer needs to phrase it
differently," a much smaller kind of gap.

**Status: closed.** All 4 of the originally-deferred numeric/comparative
gaps are done as of 2026-09-06. The only remaining item in this
condition's original scope is ambitransitive verbs (no case attached yet
— a separate, findings-level gap, never one of the 4 ADR-numbered
deferrals; not blocking).

**2026-09-06 update, in closing order**: The named-standard comparison
"identical to X" (ADR 0023's deferral) via ADR 0055 — one new grammar
alternative (an adjective complement can take a PP, reusing ADR 0031's
attachment argument for a noun-phrase complement), zero LALR(1) conflict,
2 stale linter rules removed (they were rejecting a shape the grammar now
accepts). Superlatives ("the biggest file", "the most transparent file",
ADR 0029/0030's deferral) via ADR 0056 — same short/long inflection split
ADR 0030 already used for comparatives, the same attributive slot and
optional of-PP ADR 0029 already used for the Ordinal; unbanned "most" and
the hardcoded "shortest" ban (both now auto-derived); Lexgen's
unattested-form lint caught 9 irregular-looking generated superlatives
(all real-if-rare regularizations already tolerated for the matching
comparative, e.g. "wronger" → "wrongest"), acknowledged via explicit Seed
overrides, the process working exactly as ADR 0030 designed it to.
Percent of a singular noun and a bare share as a Complement value (ADR
0024's other 2 deferrals) via ADR 0057 — the existing plural
percent-share rule was narrowed to a plural inner noun and a new sibling
rule added for a singular inner noun (so "50 percent of the file is …"
and "43 percent of the swaps are …" each get the right agreement and the
wrong-agreement form now correctly rejects, closing a latent
over-acceptance the original rule had); a bare share ("the load is 43
percent") is legal only in Complement position, so ADR 0024's
anti-anaphora ban on a bare share as subject/object is untouched. ADR
0024's 3rd deferral, decimals, turned out already closed by ADR 0029's
general decimal-digit support — verified, not built. A bare digit as a
measurement value ("the value is 0", ADR 0022's core deferral) via ADR
0058 — "0" now tokenizes as its own NUM_VAL terminal instead of a hard
lex ban, distinct from the NUM_PL count slot, so it's legal as a bare (or
approximate) Complement value but still cannot reach a count position
("0 files" stays rejected, now via a named Linter finding instead of an
unnamed lex error). ADR 0022's other 2 sub-questions (thousands
separators, negative numbers) and units generally stay open — real but
smaller and not evidenced as blocking yet; see STATUS.md's queue if they
become live. Zero LALR(1) conflict, full corpus + workspace tests green
all 4 times. One session each.

## Condition 2: the prose that results is natural, not merely parseable — THE gap

**Status: not met, and this is the real blocker.** `docs/judge-report.md`
(blind sub-agent judgement, mean naturalness ≥4.0 + no paragraph ≤2 +
fidelity ≥4 + zero unclear spans to pass): **3 of 37 documents pass**
(8%). This is after a full naturalness rewrite pass already raised the
corpus mean from 3.2 to 3.7 — real progress, but the pass bar is a cliff,
not a slope, and most documents are still on the wrong side of it.

**What's actually failing** (from the report's own failure notes, not
guessed): a small number of recurring *shapes*, not scattered typos —
1. **Parallel short-sentence "tableization"**: several definitions in a
   row using identical syntax read as a table rendered in prose, not
   writing ("the keyword X marks Y. the keyword Z marks W. …").
2. **Unanchored term definitions**: abstract terms introduced with a
   one-line definition and no concrete example score low on
   imaginability — a reader can parse the sentence and still not form a
   mental model of the thing (5 terms flagged this way in the sampled
   failures alone).
3. **Referent ambiguity from banned pronouns done badly**: angloform
   bans anaphoric pronouns (ADR 0002) to kill one kind of ambiguity, but
   several failures show noun-repetition used *mechanically* — reads as
   "the tool… the tool… the tool" rather than restructured for flow.
4. **Vague/unreconciled connective content**: a handful of "the two
   passages are not reconciled," "which X refers to is unclear" notes —
   genuine content gaps in the source reasoning carried through the
   rewrite, not a grammar limitation.

**Why this is likely the same root cause as Condition 3, not two
separate problems**: shapes 1 and 3 are both symptoms of *insufficient
sentence-combining and cohesion machinery* — the grammar can express
each isolated clean sentence but gives the writer too few ways to vary
rhythm or connect sentences smoothly, so a writer under the "must
parse" constraint reaches for the same few safe shapes repeatedly.
STATUS.md's queued "Structured repair" and "Coherence relations beyond
cause" work (queue items 1 and 3) are aimed at exactly this class of
problem.

**To close**: do not treat this as "add more words." Treat the
judge-report's failure catalogue as the language-gaps.md of naturalness
— rank the 4 shapes above by frequency across all 34 failing documents
(a mechanical pass, similar effort to `finding-frequency`'s bucketing),
then design 1–2 constructions that directly target the top shape (a
strong candidate: an appositive/parenthetical-definition construction,
which would fix both "unanchored term" and "tableization" at once by
letting a definition ride inside the sentence that first uses the
term, instead of needing its own flat sentence). This is a design
problem with real uncertainty, not a mechanical extension — budget
accordingly, likely the single largest remaining item.

**2026-09-05 update**: built (ADR 0054, `Subj, NP, Predicate`, singular
subject only) — zero LALR(1) conflict. Real naturalness payoff **not
yet shown**: a direct A/B on one real example scored 4/5 both with and
without it (tied, not a win) — see `docs/top-5-issues-2026-09-05.md`
item 1. Two other candidate cheap fixes were tested and falsified the
same day: same-subject predicate coordination (already legal) doesn't
fix a whole paragraph's rhythm, and inverting a repeated-predicate
chain into a colon-list actively regressed one real document's score
(3/5 → 2/5). The remaining honest next step is testing ADR 0054 at
scale (several real failing documents, same 3-for-3 bar ADR 0051
cleared) before trusting it.

**Same-day correction, after a skeptic-agent audit**: the original
"zero corpus regression" claim above was misleadingly precise — no
sentence in the corpus (1582 sentences) exercises the construction at
all, so "zero regression" meant "unused," not "validated." Also found:
the construction initially only worked in a bare top-level statement,
not inside `if/then`, `because`/`so`, `and`-coordination, or a Step —
most of the content it was built for. Both are fixed now (see
`docs/top-5-issues-2026-09-05.md`'s second update), but log this as a
real instance of the exact undisclosed-sub-decision failure mode
`docs/controversial-decisions-2026-09-05.md` warns about, not just a
bug.

**Second correction, same day — the load-bearing one**: a scale test
(2 real domain-model paragraphs, ADR 0054 §8) found 1 win, 1 tie by
blind LLM judges — until the user, a real human, was shown the actual
example sentences and disagreed: flagged a real garden-path reading
(`Subj, NP, Predicate` looks like an asyndetic list until the verb's
agreement resolves it) and judged the flat original more natural than
every embedded example shown, including the "winning" one. Reverted the
shipped `linter` change. This is Condition 5's gap (no human validation
data) materializing directly — every LLM-judge naturalness number this
construction has ever gotten should be read as contradicted by the
first real human check, not confirmed. Do not treat ADR 0054 as a
naturalness win until a human, not a subagent, says so.

## Condition 3: documents cohere above the sentence

**Status: partially met — infrastructure exists, coverage is thin.**
`docs/coherence-report.md` (`just coherence`) already measures the
relation inventory and topic continuity across every ADR. Per STATUS.md's
open decision queue (item 1): only the *cause* relation has real
grammar/vocabulary support; exemplification, sequence, concession,
purpose, elaboration, and cross-sentence relations generally are all
still open. This is very likely upstream of several Condition 2
failures (see above) — closing it may fix naturalness failures for
free, not just add new capability.

**To close**: read the current `docs/coherence-report.md` findings
first (don't re-derive what's already measured), then work the queued
paragraph-repair flow design already agreed in `docs/ideas.md`,
"Paragraph repair." Sequence this *with*, not after, Condition 2 — same
root cause, one combined design pass is more efficient than two.

## Condition 4: tooling lets an outsider actually write it

**Status: partially met, and the front door was actively broken.**
What exists: a CLI (`just lint`), a web playground
(`crates/wasm` + `web/`, wasm-bindgen + Vite + Playwright e2e — real
infrastructure, not a stub), and `skills/angloform/SKILL.md` (137 lines,
the LLM-facing onboarding doc, already proven to work — see Condition
5). What was broken: `README.md`, the actual front door for a human
visitor, still read **"Early stage — currently the lexicon layer and
its measurement tooling"** — describing a 2026-08 snapshot on a
2026-09-05 project with a working grammar, linter, antiparser system,
and 41 dogfooded ADRs. A reader following the README's own `cargo run
-p lexgen` / `cargo run -p triage` instructions would never discover
`diagnose`, `just lint`, or the web playground exist at all. This isn't
a cosmetic issue — it actively mis-set the bar for anyone deciding
whether to invest time evaluating the language.

**To close**: rewrite README.md to reflect current capability (this is
the cheapest, highest-leverage item in this whole document — no design
risk, no corpus regression risk, a few hours). Separately: the web
playground's actual UX maturity for a first-time visitor (does it
explain *why* something was rejected, or just show a parse error?) was
not verified this session — worth a real walkthrough before claiming
Condition 4 fully met.

## Condition 5: independent validation — does it work unaided?

**Status: real but narrow signal only.** `agenttest`'s cold-start
first-try rate (STATUS.md metrics snapshot: 44→51→48→52 of 54 across
runs, ~81–96%) is a genuine, repeated, quantified result — a model
given only `SKILL.md` + the word list, with no other coaching, gets a
requested repair right on the first try the large majority of the time.
But: (a) LLM-only, one model family tested (deepseek via opencode); (b)
"first try" means "with the linter's feedback loop available," not
zero-shot cold authoring of original content; (c) **zero data exists on
a human** attempting to read or write angloform unaided. For a language
whose stated purpose includes human-readable technical documents, this
is a real gap in the evidence, not just a formality.

**To close**: last gate, not first — running this before Conditions 1–4
show real movement would just rediscover Condition 2's failure modes
at higher cost (a human's first reaction to a document that fails the
naturalness bar is exactly the judge-report's own complaints, learned
more slowly and expensively). Once Condition 2 shows real improvement
on a handful of documents, pick 2–3 of the newly-passing ones and run
one real session: an outsider reads one, writes a short one from a
prompt, using only the README + SKILL.md + the web playground, no other
help. Their friction points are worth more than another synthetic
metric.

## The plan, in efficient order

1. **Fix README.md.** Hours, zero risk, no dependency on anything else,
   and it's actively misleading right now — do this regardless of what
   else gets prioritized.
2. ~~**Close the deferred numeric/comparative gaps** (Condition 1).~~ Done
   2026-09-06 — all 4 closed (ADR 0055, ADR 0056, ADR 0057, ADR 0058).
3. **Rank Condition 2's failure shapes by real frequency**, then design
   the 1–2 highest-leverage constructions (likely an
   appositive/parenthetical-definition form). Treat Condition 3's
   coherence-relation gap as part of the same design pass, not a
   separate follow-on — they share a root cause.
4. **Rewrite the failing documents** with the new construction(s), and
   re-run `docs/judge-report.md`'s pass rate as the acceptance check
   (target: most of the 34 failing documents, not just a handful —
   the pass bar itself is fine, the language needs to actually clear
   it).
5. **Verify the web playground's onboarding UX** with a real
   walkthrough (cheap, can happen any time after step 1).
6. **Run the human validation pilot** (Condition 5) only after step 4
   shows real movement — it's the acceptance test for the whole plan,
   not a discovery tool for problems steps 1–4 already know about.

Steps 1–2 have no design uncertainty and can start immediately. Step 3
is the load-bearing item — budget it as the largest single piece of
remaining work, and don't let steps 1–2's easy wins substitute for
starting it.

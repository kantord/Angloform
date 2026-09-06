# Review checklist: known failure types

Read before judging any linter output, rewrite, agent snapshot, or dogfood
case. Each entry is a failure that has actually happened here; the source
notes where. Add an entry when a new type is caught; never delete one.

## A. The rewrite changed the meaning (ADR 0012 tier 1 — never acceptable)

- **Dropped operator**: quantifier, negation, or deontic silently gone
  ("no agent must …" repaired to a sentence without *must*; agent run 5).
- **Flipped scope or polarity**: *every … not* read as ∀¬ when ¬∀ was meant
  (ADR 0014).
- **Changed speech act**: instruction turned into obligation or statement
  ("please delete" → "you must delete"; ADR 0019) or the reverse.
- **Participant invented or lost**: passive → active with a guessed doer;
  impersonal narrowed to *you* without noting it (ADR 0012).
- **Concept decomposed into general words**: a defined term paraphrased away
  ("the ambiguity of the reference" for Reference Ambiguity). Reads unnatural
  and loses the definition. Use the Capitalized term (ADR 0027).
- **Nominalization blind spot**: verb → noun rewrites read as dense and score
  well while hiding the event ("the process stops" → "the stop of the
  process"); the cost metric does not see it (findings §5).

## B. The rewrite optimized a metric instead of the prose (findings §5, five catches)

- **Parseability as the target**: a well-written sentence rewritten only so
  it parses. Well-written but unparseable is *gap evidence*, not a writer
  error (dogfood README step 4).
- **Rarity as the target**: substitute chosen because it is rarer, not
  clearer (*need* → *necessitate*; ADR 0008, 0023).
- **Density as the target**: compression that drops declared-loss content
  without declaring it (pairs.tsv third column).
- **Repetition tolerance misjudged**: repeating the noun is an accepted
  cost (ADR 0002); a rewrite that avoids it by adding reference is wrong.

## C. The linter's message misleads the writer

- **Template read as literal**: advice text with a canned example that does
  not match the sentence ("write \"the file\"" shown for *copies*; ADR 0023
  advice, fixed 2026-09-01). Write templates as `"the <noun>"` and mark the
  example as an example.
- **Wrong error class**: a deliberately banned word reported as "not a
  angloform word", so bans read as typos (fixed 2026-09-01: bans now say
  "is banned in angloform").
- **One reading named, the other meant**: a capitalized verb at sentence
  start diagnosed as a bad name when an imperative was meant (agent run 4;
  fixed by dual-reading advice, ADR 0019).
- **Dormant redirect**: a redirect that never fires because the word lexes
  as its enabled form ("the agent files the report"; ideas, advice gap 2).
- **Generic fallback**: "not recognizable as an angloform-like sentence" where
  a pattern finding exists or could (tier-2 count 0 with no findings). The
  ADR 0002 paragraph run showed 115 of these for four silent gaps —
  noun-noun compounds, unquoted mention, copula + PP, noun coordination —
  all now named (2026-09-02).
- **Invalid example in our own output**: an example sentence in the skill
  or in linter advice that does not itself parse. Guarded by the self-lint
  tests since 2026-09-02 (crates/diagnose/tests/selflint.rs).
- **Unfindable fix**: advice that names a word the writer would never reach
  for, or that only makes sense with insider context (ADR 0008, 0015).
- **Redirect to a word outside the lexicon**: the advice names a synonym
  that is itself not enabled (*report* → *describe*, *commit* → *save*;
  caught rewriting ADR 0001). Listed in the lexicon report since 2026-09-01.

## D. The decision was made on the wrong evidence

- **Coverage of arbitrary English treated as a goal**: the ADR sweep parse
  rate is telemetry; ADRs are meta-linguistic register (STATUS, dogfood ADR
  0001 reframe).
- **Frequency table trusted for technical words**: general-English zipf
  underrates *emit*, *notify*; guards stay report-only (ADR 0023).
- **A proprietary controlled-language spec as precedent**: not citable, not
  derivable-from (docs/resources.md policy).
- **Embedding similarity as a role judge**: pooled embeddings are order-
  blind; a subject/object swap looks identical (ideas, structured repair).
- **A word reused across constructions assumed to need a second word**:
  reuse costs nothing when something else in the sentence already
  disambiguates which construction applies (a comma, a position, a
  category) — check the judged naturalness/fidelity scores (`just
  prejudge`, `just judge-docs`) before assuming a construction needs a
  distinct word, rather than forcing distinctness as a rule (ADR 0037: "of"
  costs real ambiguity and got a hard bound; "but" reused across
  predicate- and clause-level coordination costs nothing, since the comma
  already marks which one applies).

## E. Tooling made the result look wrong

- **Shell re-split the sentence**: unquoted multi-word input lints word by
  word (justfile `lint`, fixed with positional arguments).
- **Nonzero exit on rejection read as a crash**: the linter's exit code
  reports rejection; `just` printed "recipe failed" on top of it (fixed
  2026-09-01: the recipe ignores the exit status).
- **Stale generated artifact**: report or lexicon not regenerated after a
  seed change; `./scripts/check.sh` catches drift only against the index.

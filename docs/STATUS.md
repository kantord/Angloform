# Project status and handoff

Last updated: 2026-09-05 (`just finding-frequency` — real-usage
instrumentation of `diagnose()` outcomes over 6643 near-miss minglish
sentences from `tests/paragraph-cases/` + `tests/agent-cases/`. Ranks
STYLE finding kinds so the antiparser backlog (docs/ideas.md,
"Antiparsers") gets built off real frequency, not guesses. The ranking
immediately caught a real bug, not just a coverage gap: the #2 finding,
"a clause cannot be the object of a verb" (473×, 7% of all near-miss
sentences), was ~98.5% false positives — the old token-window heuristic
("two verb-ish tokens, no connective") misfired on ordinary predicates
(do-support negation, modal + verb, copula + passive participle — each
is exactly two verb-ish tokens with nothing between them). Replaced
with a 4th antiparser, `AntiClauseObject`
(crates/antiparse/src/anti_clause_object.lalrpop): requires a real
subject-like NP wedged between the two verbs, the actual structural
signal a second clause has started — a token-window guess can never
tell that from an aux/modal/copula next to its own main verb. Bucket
dropped 473 → 7 genuine matches on rerun. Report: docs/finding-
frequency-report.md; full writeup in docs/ideas.md, "Antiparsers".
Determiner-omission (480×) and noun-noun-compounds (273×), the next two
ranked findings, were then audited the same way (40 real trigger
sentences sampled each from the near-miss corpus) — both came back
clean, zero false positives, no antiparser needed. Lesson: the
antiparser thesis applies specifically when the checked token pattern
is itself common in *valid* constructions (true of "two verb-ish
tokens", false of "bare noun with no preceding determiner" or "two
adjacent nouns"). Cleaned up two pieces of dead/tautological code found
in the determiner check during the audit (`crates/diagnose/src/
lib.rs`), verified behavior-identical (finding-frequency counts
unchanged). See docs/ideas.md's "Still not done" for what's left
(defined-term 121×, inline-list 114×, transitive-needs-object 108×, all
unaudited) and the `Repair::Single` surfacing decision. Fixed along the
way: adding a crate's second `[[bin]]` target silently broke every
`cargo run -p diagnose` call site that didn't pin `--bin diagnose`
(showcase.sh, lint-file.py, dogfood-sweep.py, justfile's `lint`) —
`|| true` swallowed the ambiguous-target error and truncated
docs/showcase.md to nothing. On 2026-09-04,
`crates/antiparse` was wired into `diagnose()` as a fourth channel,
ahead of the generic fallback — when the hand-written
`pattern_findings`/`slot_findings` checks find nothing, an antiparser
match is tried, ranked by proximity to Tier-1's real failure position
(new: `grammar::parse_tokens` / `failure_position`). Confirmed
genuine, not just infrastructure: "the mechanism only stores the
report" (ADR 0047's free-`only` ban, no prior dedicated check) now
gets a specific `[AntiFreeOnly]` explanation with both candidate fixes
named, instead of the generic message. See docs/ideas.md,
"Antiparsers", for the full writeup). Everything a fresh agent needs
that is not derivable from the code, ADRs, or git history.

## Where everything lives

- `docs/readiness.md` (2026-09-05) — "how close is minglish to usable":
  5 measured readiness conditions (vocabulary coverage: met; naturalness:
  NOT met, 3/37 docs pass the naturalness bar — the real blocker;
  paragraph coherence: partial; tooling/onboarding: partial, README.md
  was badly stale; independent validation: LLM-only signal, no human
  data) and an efficient ordered plan. Read this before answering "is
  it ready" or picking the next priority.
- `docs/prompt-ab.md` (2026-09-05) — A/B test of the `agenttest` system
  prompt: a new, 100%-dogfooded, ≤300-word prompt
  (`skills/minglish/repair-prompt.md`) matches or beats the production
  1109-word `SKILL.md` on repair fix-rate (50%→100% on the test set)
  and ties it exactly on naturalness (blind judge, two authoring
  tasks). Finding: naturalness is capped by the language's own
  conventions, not by prompt wording — further gains need a language
  construction (Condition 2/3), not more prompt tuning.
  `crates/agenttest`'s `SKILL_PATH` now points at the new file;
  `SKILL.md` itself is untouched (still the general onboarding doc).
- `docs/naturalness-iteration-2026-09-05.md` — round 2: tested whether
  a *grammar* construction (ADR 0050, 3-way clause Coordination) fixes
  naturalness where the prompt A/B couldn't. **Read the "Correction"
  section of that file first**: 7 straight blind-judge trials scored
  2/5 and looked like a confirmed hard ceiling — until it turned out
  every one of those judges was missing `CONTEXT.md` access (a
  self-inflicted deviation from the real `docs/prejudge.md` protocol).
  Re-judged correctly, the same paragraph moved 2→3/5 (ADR 0050 alone)
  →4/5 (ADR 0050 + splitting a mixed-granularity list into homogeneous
  Enumerations), with fidelity also verified at 4/5 against the
  original — clearing `docs/judge-report.md`'s actual pass bar on a
  real paragraph in `docs/adr/0014-universal-and-no.md`, now applied.
  One paragraph, not a corpus-wide result yet — see that file's
  Recommendation for the generalization step. ADR 0050 kept regardless
  (real, regression-tested capability gain, and it did measurably help
  once judged correctly). **Round 3** (same file, `/goal make at least
  10 documents pass`): found `docs/judgements.yaml` is stale for 18 of
  37 documents (including 2 of the 3 "passing" ones) — the honest
  official count right now is **1/37**, not 3/37. Also found single-run
  naturalness judging has enough variance to flip pass/fail on its own
  (documented swings up to 1.0 points between two runs on
  near-identical content) — do not trust a single judge run near the
  4.0 boundary. **Round 4**: pushed the 5 closest documents to a full
  pass via targeted unclear-span fixes — all 5 got *worse* on
  re-judging, not better (naturalness and unclear-resolution are
  frequently in direct tension on this corpus: resolving an ambiguity
  needs exactly the explicit, spelled-out phrasing naturalness scoring
  penalizes). Decisive, not noise — 5/5, one direction. **Round 5**:
  built the structural fix round 4 pointed at instead — ADR 0051 adds
  real grammar-validated table support (`is_mapping`/`parse_mapping`,
  `scripts/mdblocks.py`), asked before building since the alternative
  (an unchecked markdown table) would have abandoned ADR 0001's
  check-don't-choose discipline for a score. Applied to
  `docs/adr/0002-pronoun-policy.md`: the fixed paragraph scored 4/5
  naturalness, 0 unclear on re-judge — the first result in the whole
  investigation that held up instead of regressing. Doesn't make that
  document pass alone (other paragraphs have unrelated problems), and
  wasn't applied to the rest of the corpus given session length — the
  concrete, de-risked next step. Read that file's Round 5 section
  before doing more document-naturalness work; refreshing stale
  judgements still comes first, but the fix for the once-dominant
  failure shape now exists and works.
- `docs/controversial-decisions-2026-09-05.md` — every unilateral,
  user-unvalidated call made across both A/B rounds, most notably: a
  live grammar change may contradict ADR 0048 ("one shape per
  meaning") and directly overlaps a `grilling`-skill interview on that
  exact question that got cut off mid-conversation and never resumed;
  and both rounds' "test"/"judge" agents were free Claude subagents,
  not the real OpenRouter model `agenttest` actually uses in
  production. Read this before trusting either A/B round's numbers at
  face value, and before deciding whether ADR 0050 stands.
- `domain/model.json` — the domain model (ADR 0027): every project term with
  its minglish definition; `CONTEXT.md` is generated from it. Noun terms are
  written Capitalized in minglish text. `just define <Term>` looks one up.
  ADR 0036 schema: every noun term has `kind` ("unique" | "category"), a
  category needs `examples` (an example ending in "." is a minglish
  sentence and is self-linted; a multi-line one is a Block), `member_of`
  names its parent category. These fields live only in the model, never in
  the seed; a core lemma that becomes a term is dropped from the seed
  (done for "pronoun" and "block").
- `docs/adr/0001–0049` — every language and policy decision, each citing its
  evidence. 0006 (comprehension-first + density + expressiveness-subordinate
  + enforcement hierarchy), 0008 (redirect vs ban) as amended by 0023
  (per-sense synonyms, absolute findability floor), 0012 (loss taxonomy),
  0015 (transparency-first vocabulary) are the governing ones.
- `docs/research/cnl-design-findings.md` — empirical findings, including five
  documented cases where a metric was optimized at the cost of meaning or
  quality; read the relevant entries before treating any metric as a target.
- `tests/dogfood-cases/` — the standard dogfood flow (README there) and open
  gap cases. `tests/agent-cases/` — LLM repair snapshots with human verdicts.
- `justfile` — check / showcase / lint / agenttest / autofix /
  autofix-paragraphs (paragraph repair flow, `tests/paragraph-cases/`) targets.
- `docs/review-checklist.md` — known failure types to check in every
  evaluation. `docs/language-gaps.md` — ranked tally of missing words and
  constructions from the 2026-09-03 full-corpus naturalness rewrite;
  clause-level "but" (ADR 0037), coordination inside a causal/conditional
  clause (ADR 0038), "without" (ADR 0039), and coordinated noun phrases
  (ADR 0040 banned the bare form on a documented LALR finding; ADR 0041
  reopened it behind a colon, "the mechanism stores: a word and a
  message", which sidesteps the conflict entirely) are resolved. What is
  left is single missing words ("only", "other"), not constructions.
- `docs/prejudge.md` / `just prejudge` — blind sub-agent pre-judgement of
  paragraph repairs (naturalness score; telephone game: explain-then-rate
  fidelity) recorded under `prejudge:` in each case, docs/prejudge-report.md.
  Runs on Claude Code sub-agents, so it costs no API key; it filters and
  flags, the human verdict stays `just verdict`.
- `scripts/lint-file.py` / `just lint-file <md>` — document-level lint,
  works on any markdown file (2026-09-03): per-sentence verdicts, a
  heading check (vocabulary-only, not the sentence grammar), topic
  continuity, relation inventory. Shared markdown structural parsing lives
  in `scripts/mdblocks.py` (code fences, tables, blockquotes, links,
  nested/task lists, headings) — see `docs/markdown-linting.md` for the
  block-classification and heading rules and why. `just coherence`
  regenerates `docs/coherence-report.md` over all ADRs.

## Open decision queue (in order, with standing recommendations)

1. **Coherence relations beyond cause** (exemplification, sequence,
   concession, purpose, elaboration) and cross-sentence relations: decide
   from `docs/coherence-report.md` (relation inventory + topic continuity,
   `just coherence`), not from word counts. Next: the paragraph repair flow
   (agreed design in docs/ideas.md, "Paragraph repair").
2. Parked with design notes in `docs/ideas.md`: vocative (directed
   imperatives), quotation-as-mention, pseudocode/analysis-code compiler,
   embeddings-as-measurement, core+jargon-packs split, LM-based cost model.
3. Structured repair (docs/ideas.md, "Structured repair"): structure
   enumerator → role assignment → table-driven rewrite with explanations;
   NLI as the future faithfulness gate. Steps 1–3 need no model and no API
   spend; the first candidate for build work after the vocabulary queue.
4. ~~Deferred by ADR 0022 (numbers, decided 2026-09-01): measurement values
   ("the exit code is 0" — the natural home for a future 0).~~ Closed
   2026-09-06 by ADR 0058 — "0" tokenizes as its own NUM_VAL terminal, a
   value slot distinct from the NUM_PL count slot, legal as a bare
   Complement value but still unable to reach a count position. Units and
   thousands separators remain open (smaller, not evidenced as blocking
   yet). This was the last of the original 4 numeric/comparative gaps —
   see docs/readiness.md.
5. ~~Deferred by ADR 0023 (same): the named-standard form "identical to the
   report".~~ Closed 2026-09-06 by ADR 0055 — an adjective complement can
   now take a PP, same attachment as ADR 0031's NP complement.
6. ~~Deferred by ADR 0024 (percent): percent of a singular mass, percentages
   as predicates, decimals.~~ Closed 2026-09-06 by ADR 0057 (mass/predicate)
   — decimals were already covered by ADR 0029. Ambitransitives ("the
   process stops") remain a findings-level gap with no case attached;
   commit-as-verb (adr0001-05) was resolved by the linter's own redirect
   (*save*).

## Working process (session-established, not in any ADR)

- Language decisions go through a grilling interview (options + recommendation,
  user decides), then land as: tentative ADR + seed/grammar change + corpus
  sentences + reject tests + regenerated reports, all in one commit.
- `./scripts/check.sh` must pass before commits; it regenerates everything
  and drift-checks committed artifacts. It includes the self-lint tests:
  every example sentence in the skill, in linter advice, and in ban advice
  must itself parse — never feed the model an invalid example.
- The user runs all `git commit`s and all API-spending runs (agenttest,
  autofix) themselves; those runs are milestones, not routine.
- Between milestones, `just replay` is the proxy: it re-validates every
  stored paragraph proposal (tests/paragraph-cases/) against the current
  linter, no API. A linter change should raise valid counts and lower the
  no-advice count; a language change that lowers them needs a reason.
- Every review of linter output, rewrites, or agent snapshots goes through
  `docs/review-checklist.md` (known failure types; append, never delete).
- Dogfood flow: lint own docs → useful flag → rewrite in place (English
  quality first — parse-validity is NEVER the rewrite target; well-written
  but unparseable = gap evidence) → else file gap case.

## Metrics snapshot (2026-09-01)

- Lexicon 652 forms incl. 24 Capitalized term nouns (ADR 0027); corpus 70/70
  parse; grammar LR(1)-clean, zero
  precedence declarations (CI-enforced).
- agenttest first-try trajectory: 44 → 51 → 48 → 52 of 54 (run 5). Predictions registered for run 6: "please delete the file" should
  snapshot to the bare imperative (skill now teaches it); the no-must case
  should stop dropping the deontic (advice got a concrete example);
  autofix rerun on ADR 0001 should yield real proposals + GAP declarations
  (extractor fixed, refusal-routing strengthened).
- Naturalness pass (2026-09-03): every ADR (0001–0036) and the domain
  model were rewritten by blind sub-agents for naturalness, judged by
  independent naturalness/telephone-fidelity/imaginability judges before
  and after (docs/judge-report.md, docs/judgements.yaml,
  scripts/docjudge.py, `just judge-docs`). Naturalness mean rose from
  3.2 to 3.7 corpus-wide; every document still fully parses (100%,
  docs/dogfood-sweep.md) and lints clean. docs/language-gaps.md records
  what blocked further improvement.
- ADR sweep: every ADR (0001–0028) is fully minglish since 2026-09-02
  (docs/dogfood-sweep.md; docs/dogfood-adr-00NN.md per ADR). 0002 was rewritten
  by hand after nine paragraph-repair runs; 0005 after one cold run; the other
  26 with no model run (lint-file + fix-gap-or-follow-advice). The problem
  archetypes met are logged in docs/rewrite-archetypes.md for the batch review.
- Property-based tests (2026-09-03): `proptest` is a dev-dependency of
  `grammar`, `diagnose`, and `lexgen` — crash-freedom fuzzing on `tokenize`/
  `units`/`parse`/`diagnose`/morph.rs, plus a lexicon-driven generator in
  `crates/diagnose/tests/proptest_generated.rs` asserting every generated
  `Statement`/`Coordination` sentence parses and Tier-2 stays a superset of
  Tier-1. Full survey and what's not yet built in docs/ideas.md,
  "Property-based testing with proptest".
- Lexicon ≈1770 forms, ≈95 domain terms; `just replay` is the no-LLM proxy.
  `scripts/seedcheck.py` pre-flights a batch of new lemmas (unattested forms,
  cross-POS waivers) so one lexgen run succeeds; its auto-waivers are noted
  `[auto-waived …]` and belong to the batch review. Expected: ADRs are meta-linguistic
  register, and coverage of them is a non-target (see the reframe in
  docs/dogfood-adr-0001.md: coverage of arbitrary English is a non-goal).
- Dogfood ADR 0001: every sentence parses (in-place rewrite, 15 declared
  pairs in corpus/dogfood-pairs.tsv); cost ratio in docs/dogfood-cost-report.md
  (understated by the nominalization blind spot — see findings).

## Standing policies easy to violate accidentally

- Epistemic hedges ("maybe", "probably", "likely", "seem", "might",
  "could", 13 words total, ADR 0042) are a permanent Ban, not a
  coverage gap — do not add one to close a vocabulary gap; the fix is
  always to state the claim directly or name the source.
- No references to (or derivation from) non-open controlled-language
  specifications anywhere in the repo — the anonymous policy note in
  docs/resources.md is deliberate; do not name them even to warn.
- CC BY-SA data (UD-EWT, wordfreq table) is fetched, never vendored
  (scripts/fetch-data.sh); tracked content stays MIT/Apache + permissive.
- The dogfood curation phase is our-own-jargon-first (docs/ideas.md,
  "Curation phasing"); EWT triage numbers are telemetry, not targets.

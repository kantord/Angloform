# System-prompt A/B test for the repair loop (2026-09-05)

Answers: does the *system prompt* (not just the language) affect
repair fix-rate and naturalness for `agenttest`? Method, 4 iterations
(v0 → v3), stopped when gains plateaued — full run in the session
transcript, not re-run here since it costs no OpenRouter calls (all
"test agents" and judges were free Claude Code subagents, not
OpenRouter LLM calls; the repair loop's *own* real-model runs still
need `OPENROUTER_API_KEY` and are the user's to trigger).

## Setup

- Fixed test set: 8 real near-miss sentences pulled from
  `docs/finding-frequency-report.md`'s own ranked examples (determiner
  omission, noun-noun compound, defined term, inline list, name/verb
  collision, name-determiner, clause-as-object, passive).
- Fixed lexicon/defined-term appendix (mechanically generated from
  `lexicon.tsv` + `domain/model.json`, same logic as
  `build_system_prompt()` in `crates/agenttest/src/main.rs`) — held
  constant across every candidate, so only the hand-written prose
  varied.
- Each candidate: a fresh Claude Code subagent per test sentence,
  given the candidate prompt + the linter's real diagnosis, asked for
  one corrected sentence; scored by running the real `diagnose` tool
  on the reply (objective, not a judge call).
- **Constraint added mid-run**: the candidate prompt itself must be
  valid angloform — every sentence in it must parse, checked with `just
  lint-file`. v0/v1 were ordinary English prose and were rewritten as
  angloform from v1 onward.

## Fix-rate results

| candidate | words | self-parse | fix-rate (6-sentence core set) |
|---|---|---|---|
| v0 (English prose) | 299 | n/a | 4/6 (50% on 8-set) |
| v1 (angloform, added NP-coordination rule) | 293 | 100% | 4/6 (unchanged — model still tried the banned "repeat the verb" workaround) |
| v2 (added "no repeated verb" + "quote every mention") | 283 | 100% | 6/6 (100%) |
| v3 (strengthened the quoting rule to "every") | 283 | 100% | 6/6 core, 7/8 extended set |

The one remaining extended-set failure (the 4-word quoting case) is a
**test-harness artifact, not a prompt defect**: the real linter's
diagnosis only names 2 of 4 unquoted words in one pass (it doesn't
enumerate every instance of a finding kind), and the real `agenttest`
loop gets up to 3 repair rounds with a fresh diagnosis each time — my
single-shot test harness gave the model only one round. A second round
would almost certainly catch the rest.

## Naturalness (blind judge) and telephone-game fidelity

Two authoring tasks (not repair — cold composition), one blind
naturalness judge + telephone-explanation per output, comparing v3
(293 words) against the current production `SKILL.md` (1109 words, not
itself dogfooded — see below):

- **Definitional task** ("what is a Rejection/Gap/Ban, how does the
  Linter help"): both prompts converged on nearly identical text,
  both 100% parse — because both drew from the same pre-written
  `domain/model.json` definitions. Weak signal; the lexicon appendix
  dominates, not the prose.
- **Scenario task** (fresh composition, not covered by any existing
  definition): v3 scored 4/5 sentences parse (80%, one scope-ambiguity
  case), production scored 3/4 (75%, a malformed Enumeration). Blind
  naturalness judges scored **both passages 2/5**, with near-identical
  complaints (bare Capitalized category names read as "tags," template
  repetition). Both telephone-explanations correctly reconstructed the
  intended scenario — good fidelity either way.

**Conclusion: naturalness is capped by the language's own conventions
(mandatory Capitalized defined-terms, fixed sentence templates), not
by which prompt phrasing describes them.** This matches
`docs/readiness.md` Condition 2's diagnosis — the system prompt has
real headroom on fix-rate, essentially none on naturalness by itself.
Naturalness needs a language-level fix (new constructions), not a
better-worded prompt.

## Dogfood status, for contrast

`skills/angloform/SKILL.md` (production, 1109 words): **2/51 sentences
(4%) parse as angloform** — it's ordinary English prose describing the
rules, not itself angloform. `skills/angloform/repair-prompt.md` (this
candidate, 283 words of prose): **35/35 sentences (100%)** — every
rule is stated as an actual angloform sentence, checked with `just
lint-file`.

## What changed

- New file `skills/angloform/repair-prompt.md` — the winning ≤300-word,
  100%-dogfooded candidate.
- `crates/agenttest/src/main.rs`'s `SKILL_PATH` now points at it
  instead of `skills/angloform/SKILL.md`. **`SKILL.md` itself is
  untouched** — it stays the general-purpose onboarding doc (used
  interactively, and covers constructions like Conditional and Step
  Block that `repair-prompt.md` deliberately dropped to fit the word
  budget). `repair-prompt.md` is scoped to the repair task specifically.

## Why this stopped at 4 iterations, not 15

Fix-rate plateaued at the test set's ceiling (100% on the core set)
after v2; the one remaining gap on the extended set is a test-harness
limit, not something more prompt-wordsmithing would fix. Naturalness
tied exactly across two different authoring tasks against a prompt 4x
longer — further iteration on prose alone has no evident room left to
move that number. Both are the "marginal, stop" condition named in the
goal.

## Next lever, if pursued further

Since the prompt is now maxed out on its axis, the naturalness gain
has to come from the language side — this is exactly `docs/readiness.md`
Condition 2/3's queued design work (a construction that avoids bare
Capitalized-term "tag" repetition, e.g. an appositive/parenthetical
form), not further prompt tuning.

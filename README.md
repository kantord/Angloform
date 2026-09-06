# angloform

A formalized, unambiguous subset of English with a real LALR(1) grammar:
every enabled word has exactly one syntactic role and every sentence has
exactly one parse, so text can be checked and parsed deterministically —
not just a curated word list. Used to write its own design decisions:
every ADR in `docs/adr/` (54 so far) is itself angloform, self-parsing,
enforced by CI.

**How ready is it?** `docs/readiness.md` is the living, measured answer —
read that, not this file, for current status; it's updated whenever a
condition changes so it doesn't go stale the way this file once did.

## Try it

```
just lint "the tool stores an error."     # lint one sentence — ✓ or a named reason for rejection
just lint-file docs/adr/0001-generated-lexicon-from-curated-seed.md  # lint a whole document
just web                                   # run the browser playground (crates/wasm + web/)
```

An LLM (or a person) writing angloform should start from
`skills/angloform/SKILL.md` — the onboarding doc, itself proven against a
real repair-loop harness (see `docs/readiness.md`, Condition 5).

## Layout

- `seed/seed.json` — hand-curated vocabulary source of truth (the only
  hand-edited lexical input; see `docs/adr/0001`)
- `crates/lexgen` — generates `lexicon.tsv` + `docs/lexicon-report.md`
- `crates/grammar` — the LALR(1) grammar (`angloform.lalrpop`), parser, and
  cognitive-load metrics (peak-open dependencies, right-branching share)
- `crates/diagnose` — the linter CLI: parses cleanly (✓) or names exactly
  why not (a missing word, a banned shape, a specific style finding)
- `crates/antiparse` — pattern-matches common near-miss shapes the main
  grammar rejects, to give a more specific reason than "no parse"
- `crates/triage` — evaluates the lexicon against a gold-tagged corpus
- `crates/wasm` + `web/` — the browser linter playground (wasm-bindgen +
  Vite, with Playwright e2e tests)
- `crates/agenttest` — the LLM repair-loop harness (milestone runs only,
  needs an API key — see `docs/readiness.md`, Condition 5)
- `crates/textcost` — compares angloform vs. English token/word cost
- `domain/model.json` — the project's own domain model (ADR 0027);
  `CONTEXT.md` is generated from it (`just define "<Term>"` looks one up)
- `docs/adr/` — every language/policy decision, each self-parsing angloform
  citing its own evidence · `docs/research/` — empirical findings ·
  `docs/readiness.md` — current status and the ordered plan to close gaps
- `corpus/accept.txt` — hand-picked grammar-feature regression sentences
  (the real content corpus is the ADRs themselves, in `docs/adr/`)

## Build

```
./scripts/fetch-data.sh     # once: fetch/derive non-vendored reference data
cargo run -p lexgen         # regenerate lexicon + report (lint-gated)
cargo run -p triage         # evaluate against UD-EWT
./scripts/check.sh          # everything: tests, regeneration, drift check
```

## License

Code and original content: MIT OR Apache-2.0, at your option
(see LICENSE-MIT, LICENSE-APACHE).

## Third-party data notices

- **WordNet 3.0** (vendored in `data/wordnet/`): © Princeton University,
  used under the WordNet License — see `data/wordnet/LICENSE`.
- **Moby Part-of-Speech** (vendored in `data/moby/`): Grady Ward, public
  domain.
- **UD_English-EWT r2.16** (fetched, not vendored): © the UD English-EWT
  contributors, CC BY-SA 4.0
  (<https://github.com/UniversalDependencies/UD_English-EWT>). Used
  unmodified as an evaluation corpus; triage reports quote sample sentences
  from it.
- **wordfreq 3.1.1 data** (derived table, fetched/regenerated, not vendored):
  Robyn Speer, data CC BY-SA 4.0
  (<https://github.com/rspeer/wordfreq>). Change: reformatted to a
  word/zipf TSV, truncated to the top 100k English words.

The generated `lexicon.tsv` is original work: reference data is used only to
*check* human word choices, never to select or copy content (ADR 0001).

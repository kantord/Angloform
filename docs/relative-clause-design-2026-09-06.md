# Relative clause construction: design scoping (2026-09-06)

Working notes from a grilling-interview session, ahead of a formal ADR.
Motivation: `docs/judge-report.md`'s failure catalogue, tagged by shape
(see `docs/readiness.md` Condition 2's 2026-09-06 update) — 74/151
naturalness sub-notes (49%) are the same subject repeated as a full noun
phrase across consecutive sentences ("the Linter is a tool of the
project. the Linter lints a sentence. the Linter parses…"), the single
largest naturalness-failure bucket found.

## Prior art this design must respect

- `seed/seed.json` already bans **"that"** outright (3-way ambiguity:
  pointer / complementizer / relative — advice text cites ADR 0010).
- `docs/adr/0010-simple-past.md` bans the **Reduced Relative** (bare
  participle, no relativizer — "the file stored in the cache") and the
  Passive, citing the classic reduced-relative garden path. This danger
  is specific to the *reduced* (relativizer-less) form.
- `docs/showcase.md:228` states plainly: "angloform has no relative
  clauses yet."
- `docs/adr/0054-appositive.md` + `docs/garden-paths-2026-09-06.md`:
  the Appositive (`Subj, NP, Predicate`) was shipped on LLM-judge
  naturalness scores, then a **real human** caught a garden path the
  judges missed (bare comma-then-NP reads as an asyndetic list until
  the predicate's agreement resolves it, many tokens later). Fixed by a
  mandatory "namely" marker. Governing lesson, stated directly by the
  maintainer: revise any decision until we can no longer find a way to
  challenge it — and never trust LLM-judge naturalness alone; a human
  must check real examples before anything ships as a "win."
- `docs/garden-paths-2026-09-06.md` also gives the project's own
  mechanical detection method (`scripts/garden-path-scan.py`, scans
  `angloform.lalrpop` for "comma-only junction, same category, no
  conjunction until late") and its governing principle: *"the best
  tool is to just not allow the construction in the grammar, if there
  is a way to do that."*

## Decisions made so far

1. **Frame as revising ADR 0010**, not a fresh unrelated decision — the
   danger ADR 0010 cited (Reduced Relative) doesn't apply to a
   construction with a mandatory explicit relativizer; "that" stays
   banned regardless, for its own separate reason.
2. **Two relativizers**: "which" (non-human antecedent), "who"
   (human/agent antecedent — maintainer, writer, reader, agent, user,
   person, …). A single universal relativizer was considered and
   rejected — "the maintainers, which own the project" would actively
   hurt naturalness in exactly the register this exists to help.
3. **Who/which correctness is Linter-enforced, not grammar-enforced**:
   a small, hand-curated agent-noun list (not a lexicon-wide animacy
   field across all 364 `NOUN` entries), checked by `diagnose()` as a
   STYLE finding on mismatch — same Grammar-accepts/Linter-verifies
   split already used for ADR 0008 (Redirect vs. Ban) and the
   register-parameter idea (`docs/ideas.md`).
4. **Object-relative: not built**, not merely deferred. Similarity-based
   interference research (Gordon, Lewis & Vasishth) predicts object-
   relatives are *worse* in angloform's register specifically — the
   intervening subject NP is almost always a similarly-shaped definite
   NP (the exact worst-case condition for that effect), and there's no
   evidenced need.
5. **Subject-attachment (the antecedent is the matrix subject) is v2,
   gated on testing** — same fresh-agent-incremental-reading + real-
   human-check protocol that caught ADR 0054's problem, not shipped on
   judge score. Reasoning: attaching a relative clause to the matrix
   subject center-embeds — it holds the subject-verb dependency open
   across the whole clause, the same structural shape as ADR 0054's
   confirmed garden path, independent of the SRC/ORC-internal-role
   question.
6. **v1 scope: non-subject attachment only** (object or PP-object of a
   non-copula verb), **internally subject-relative only** (the
   relativizer plays the subject role inside its own clause — no gap-
   tracking needed), **antecedent must be a bare NP** — no trailing PP
   (rules out attachment ambiguity per Cuetos & Mitchell's N1/N2
   attachment-preference research — "a tool of the project that lints…"
   has two candidate attachment sites, the wrong one favored by late
   closure).
7. **Copula-Complement/predicate-nominal position dropped from v1
   scope.** Originally proposed as a way to keep the definiendum as
   subject/topic (avoiding a Condition-3 topic-continuity cost) while
   still attaching peripherally. Rejected on a closer look: non-
   restrictive relative clauses don't sit naturally on **indefinite**
   NPs ("the Linter is a tool, which lints a sentence" reads oddly —
   non-restrictive adds info about something already uniquely
   identified; "a tool" hasn't been identified yet, that's what a
   *restrictive* clause does, which is out of scope). Since "X is a Y"
   copula-Complements are almost always indefinite, this carve-out
   would rarely fire — not worth a special-cased rule for.

## Allowed / banned, worked examples

Allowed:
- *"the Lexer produces the token, which sits in the position of a
  determiner and takes a plural noun."* — object antecedent, definite,
  bare NP, "which," subject-relative internally.
- *"the tool notifies the maintainers, who approve the release."* —
  same shape, "who" on an agent-noun-list antecedent.
- *"the tool writes the file, which enters the pipeline."*

Banned:
- Subject antecedent (*"the Linter, which lints a sentence, parses…"*)
  — v2, gated.
- Object-relative internal role (*"the report, which the Linter
  names, needs a fix."*) — not built.
- PP-modified antecedent (*"a tool of the project, which lints…"*) —
  attachment ambiguity.
- "that" as relativizer — already lexicon-banned.
- who/which mismatch (*"the tool notifies the Linter, who lints…"*) —
  Linter STYLE finding.

## Ambiguity analysis

Structurally low risk **as scoped**: the relativizer is a closed-class
word used nowhere else in the grammar today, so "NP, which" cannot be
mistaken for a list/coordinate-NP start the way the Appositive's bare
"NP, NP" was; no PP-attachment ambiguity by construction (Q5); no
subject/object attachment ambiguity because the two candidate NPs in
"the tool writes the file, which…" are separated by the verb, not
PP-adjacent (unlike the classic "servant of the actress who…" case).
**Still requires empirical verification before trusting it** —
`scripts/garden-path-scan.py` against the built rule, plus the same
fresh-agent incremental-reading + real-human-check protocol used for
ADR 0054. Formal reasoning alone was exactly what went wrong there.

## Real payoff is smaller than the framing first suggested

Re-checked against the actual corpus, not assumed: the token example's
4 repeated uses of "the token" only lose **one** repetition to v1
(object-attachment can fold in the 3rd sentence; the 2nd and 4th still
need "the token" restated as their own subject — v1 can't touch subject
position, and there's no chaining of two RCs off one antecedent). Most
of the 49% bucket's actual payoff lives in the subject-attachment case,
which is gated on testing, not in v1. v1 is a real but modest, low-risk
slice — not most of the win. Also: v1 does nothing for shape 2
(unanchored-term imaginability, 15 notes) — different failure shape,
never in scope for this construction.

8. **Clause-internal predicate: reuse the ordinary `Predicate`
   nonterminal wholesale** (object, Complement, PP-modifiers,
   same-subject coordination), capped at **depth 1** — no second
   relative clause or Appositive nested inside it. Needed for the
   flagship example ("which sits… and takes…") to even be expressible;
   reuses an already-conflict-checked nonterminal rather than inventing
   a parallel restricted one, matching ADR 0055/0057's precedent.
9. **Must work inside Conditional/Causal/Coordination from the start**,
   not bare-statement-only — ADR 0054 shipped bare-only, found the gap
   post-ship, had to patch it in. Checking proactively this time instead
   of repeating that miss.
10. **Acceptance bar before v1 counts as validated** (stacks every
    lesson this project has already paid for, not just ADR 0051's bar
    alone):
    1. `scripts/garden-path-scan.py` against the built rule — zero
       flagged risk shape.
    2. Fresh-agent incremental self-paced reading on 2-3 constructed
       examples (the method that actually caught ADR 0054's problem) —
       no reanalysis reported.
    3. **3-for-3** blind-judge naturalness improvement on 3 real
       documents from the failing corpus (ADR 0051's bar) — not
       synthetic examples.
    4. **Mandatory real human review of the actual before/after
       sentences**, not just the aggregate score, before the ADR's
       status moves from "proposed (tentative)" to trusted. No
       shortcut around this step — it's the one that caught ADR 0054's
       real problem after the judges missed it.

Steps 10.1-10.3 run before anything comes back to the maintainer; step
10.4 is the maintainer's by construction.

# Garden paths in angloform: a definition, an audit, and static detection (2026-09-06)

Requested via `/goal`. Every empirical claim below was tested, not
assumed — either against the real grammar (`cargo build`/`just lint`)
or with a fresh agent doing incremental, word-by-word reading (a
self-paced-reading protocol, the standard psycholinguistic method for
this exact question). One hypothesis here was tested and **falsified**;
keeping that in, not just the wins, is the point of testing at all.

## Definition

A **garden-path** sentence is one where a reader's incremental,
left-to-right parser commits to a locally-preferred structural analysis
of the words seen so far, and that commitment is later **proven wrong**
by material that hasn't arrived yet — forcing a costly revision
("reanalysis") of something already structurally assigned. The classic
example: *"The horse raced past the barn fell."* A reader parses "The
horse raced past the barn" as a complete clause (subject + main verb
"raced"); "fell" then arrives and cannot fit, forcing the reader back to
reinterpret "raced" as a reduced relative ("the horse **that was**
raced past the barn") with "fell" as the real main verb.

**Why it's bad, precisely**: it is not the same claim as "the sentence
is formally ambiguous." A CFG can be provably unambiguous (LALR(1),
exactly one parse for every string) and still garden-path a human
reader, because a human's parser is incremental and resource-bounded —
it commits before it has seen the disambiguating token, using cheap
heuristics (*minimal attachment*: prefer the analysis needing the least
extra structure; *late closure*: prefer attaching new material to the
phrase currently open) that are usually right but sometimes aren't.
"Proven conflict-free by the LALRPOP build" and "doesn't make a human
stumble" are different claims — this session conflated them once
already, on the Appositive (ADR 0054), until the user pointed it out
directly.

**Distinguish from a weaker, related phenomenon**: *suspended
ambiguity* — the reader holds 2 readings open without wrongly
committing to either, and resolution arrives cleanly with no revision
cost. This is real and worth tracking, but it is not a garden path in
the strict sense; conflating the two would over-flag.

## Audit of the current grammar

### Confirmed real garden path

**The Appositive (ADR 0054): `Subj, NP, Predicate`.** `Subj, NP` is
visually identical to an asyndetic list (2 comma-joined NPs, no "and")
until the predicate's agreement arrives, which may be many tokens
later (worse when embedded in a Conditional/Causal). Confirmed by the
user's own direct reaction to real examples this session ("this
actually weirdly somehow sounds ambiguous") — the single strongest
piece of evidence in this whole audit, stronger than any LLM-judge
score gathered all session. Root cause: angloform otherwise **never**
allows 2 same-category constituents joined by a bare comma with no
conjunction (NP-coordination is banned outright, colon-list only) — so
this construction is the *one place* that shape exists, and it
happens to collide with a reader's general-English prior that a bare
comma-list defaults to coordination.

### Same mechanism, weaker in practice — tested, not just assumed

**N-ary `CoordClause` (ADR 0050): `Clause, Clause, and Clause`.**
Shares the Appositive's exact structural signature (comma-only
junction before the disambiguating conjunction, which arrives only
before the *last* item). Tested with a fresh agent reading
incrementally ("The cache is fast, the database is slow," → then
"and the network is stable."). Result: **not a true garden path** —
the agent held 2 readings open ("comma splice" vs. "list still
building") without wrongly committing to either, and the trailing
comma (not a period) was itself a strong enough cue to lean "list."
Suspended ambiguity, not reanalysis. Lower severity than the
Appositive, worth knowing about, not worth reverting.

### Hypothesized, tested, and falsified — reported because it's honest, not because it held up

**Cross-POS "waived" vocabulary** (`safe` enabled ADJ / waived NOUN,
and ~300 more words like it — a real, large, already-computed list
from `lexgen`'s WordNet check). Hypothesis: a word like "safe" in "the
safe file" might get read with its common NOUN sense (a strongbox)
before "file" forces the ADJECTIVE reading. **Tested and falsified**:
a fresh agent reading "The safe" → "The safe file" reported the
adjective reading was *already* its leading hypothesis at "The safe,"
with the noun reading held only as a weak secondary possibility that
"file" quietly eliminated — no reanalysis, no cost. Lesson: in
angloform's rigid `Det (Adj) Noun` template, the position itself
already biases the reader correctly enough that raw cross-POS
ambiguity mostly doesn't cascade into real garden-pathing. This
category is real but much weaker than its list size (~300 words)
suggests — don't over-flag it wholesale.

### Already eliminated by design — genuine wins worth stating plainly

- **Classic PP-attachment ("I saw the man with the telescope")**:
  structurally impossible. `PREP_N` is exactly one word ("of");
  `PREP_V` is everything else (across, after, at, by, for, from, in,
  on, to, with, without, ...), zero lexical overlap (verified:
  `comm -12` on the lexicon returns nothing). A human reader's own
  strong "of attaches to the noun, everything else to the verb"
  intuition matches this split, so it's eliminated both formally and
  for a human reader, not just formally.
- **Classic reduced-relative garden path ("the horse raced past the
  barn fell")**: structurally impossible — angloform bans Reduced
  Relatives and the Passive outright (see the `participle` domain
  entry: "the language bans every Reduced Relative, so a Participle
  cannot follow a noun").
- **Focus-operator scope ambiguity ("only")**: `only` is confined
  inside the NP it wraps (ADR 0047) — cannot float to scope over the
  verb or clause the way free-floating "only" famously does in real
  English. Lower risk than the classic case, though not empirically
  tested this round.

## Static detection methods

Ranked by how directly they're usable today.

1. **Reuse `peak-open` — already computed, already the right proxy.**
   `diagnose`'s existing peak-open-dependencies metric *is* a garden-
   path severity signal: it measures how many dependencies stay
   structurally unresolved at once, which is exactly what widens a
   reanalysis window. The Appositive nested in a Conditional already
   showed peak-open 5 in this session's own testing — above the
   "comfortable ceiling ≈4" this project's own research doc
   (`docs/research/cnl-design-findings.md`) already cites from
   Gibson's DLT. No new tooling needed: flag any construction that
   consistently pushes peak-open at or above 4 for a second look.

2. **Mechanical grammar-source scan: "comma-only junction, same
   category, no conjunction until late."** Both confirmed/tested
   findings above (Appositive, N-ary CoordClause) share one precise,
   textually-searchable shape in `angloform.lalrpop`: a production
   where 2+ constituents of the *same broad category* (NP, Clause)
   appear separated only by `LComma`, with the disambiguating `LConj`
   (or other marker) appearing only before the *last* one, not between
   every pair. This is a real, mechanical check a script could run
   against the grammar source today: search every production for
   `X COMMA X ... COMMA CONJ X` shapes and flag them for a human-
   reading test before merging. Any *future* construction proposal
   should be checked against this pattern before being built, not
   after a human catches it in the wild.

3. **Naive-heuristic-parser diff (not yet built, the principled
   method).** Garden path's own textbook operational definition: build
   a second, deliberately naive parser that only implements minimal-
   attachment + late-closure (attach new material to the most recently
   opened constituent; never posit extra structure unless forced) and
   diff its output against Tier-1's real parse for every corpus
   sentence. Any sentence where they disagree *is* a garden path by
   definition — this is the correct general-purpose tool, and it is
   real, buildable work (a second, simplified grammar or hand-written
   incremental parser), not implemented this session.

4. **Narrowed lexical-frequency-mismatch check (weaker signal, use
   sparingly).** Given finding 3's falsification, don't flag every
   `waive`-listed word. A tighter, still-static version: flag a waived
   word only when it appears as a **bare** `Det Noun` (no adjective
   present, nothing else pending) *and* general-usage frequency data
   (already available via the project's WordNet/Moby infrastructure
   used in `lexgen`/`triage`) says the waived sense outranks the
   enabled one. That is the specific shape where a reader has no other
   local cue to stay hedged — closer to the tested case's actual
   failure condition than "any cross-POS word, anywhere."

## Update: fixed, not just detected

The user's own framing was sharper than "detect it after the fact":
*"the best tool is to just not allow the construction in the grammar,
if there is a way to do that."* There was. The Appositive's opening
comma now requires the word **"namely"** immediately after it —
`Subj, namely NP, Predicate` — a word real English already uses for
exactly this job (non-restrictive restatement). "Subj, NP," (no
marker) is now grammatically **impossible**; the ambiguous string
simply cannot be produced. Zero LALR(1) conflict, zero regression on
the full corpus, and the fix was verified the same way the original
problem was found — a fresh agent reading the construction
incrementally confirmed it never considered the coordinated-subject
reading with "namely" present, where it had for the bare form. This is
finding #2 from the static-detection list turned into an actual
prevention, not just a flag: the garden path is now structurally
unrepresentable, not merely caught by a linter.

**`scripts/garden-path-scan.py`** (finding #2, built): a mechanical
scanner for the confirmed risk shape — 2 same-category constituents
joined only by a comma, no conjunction between them — searched
directly against `angloform.lalrpop`'s productions. Validated against
both states: run against the *old* (pre-fix) Appositive rule, it
correctly flags `SubjSG COMMA ApposContent` as unmarked/high-priority;
run against the *current* grammar, it finds exactly one remaining
candidate, `CoordClause`'s `Clause COMMA Clause`, correctly marked
lower-priority (a conjunction sits nearby) — matching this doc's own
empirical finding that the CoordClause case is suspended ambiguity,
not a true garden path. Run it on any future ADR before a human has to
catch the next one: `python3 scripts/garden-path-scan.py`.

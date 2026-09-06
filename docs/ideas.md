# Ideas (deferred, not designed)

Parking lot for directions we intend to explore but have deliberately not
committed to. Move an item into an ADR when it becomes a real decision.

## Register parameters — configurable per-document constraints (2026-09-05)

Raised by the maintainer while discussing ADR 0052 (adding "am"). "I"
is already a sanctioned Pronoun usable as the subject of any ordinary
verb everywhere in angloform — the ADR 0052 gap was narrowly "I" +
Copula, an accidental hole, not a deliberate first-person restriction.
But the maintainer wants a *real* first-person restriction available
for some registers (ADRs are the concrete example: read through this
session's own corpus work, every ADR uses "the maintainers decided",
never "I decided" — an existing, unenforced convention, not a rule).

The idea: a **register parameter** — a named, document-level (or
per-run) switch the Linter checks in addition to the fixed Grammar,
not a Grammar-level ban. This matches the project's existing
Grammar-accepts/Linter-verifies split (the same architecture behind
`same_verb_coordination`/`other_domain_membership` in
`crates/diagnose`, and the antiparsers) — a register parameter is
exactly that pattern generalized: the Grammar stays register-agnostic
(sentences that use "I" always parse), and a register-level check
in the Linter can reject them when the active register forbids it,
with its own named, actionable error (e.g. "the ADR register bans the
Pronoun 'I' — name the maintainers or the tool instead").

First candidate parameter: `first_person: allowed | banned`. Not
designed further this session — needs at minimum: how a document
declares its own register (frontmatter? a CLI flag to `just lint`?),
whether `docjudge`/`prejudge` need register-awareness too, and whether
any other existing "soft convention" in the corpus (there may be more
than just first-person) is a second candidate parameter worth adding
at the same time rather than one at a time.

## A fully self-hosting language specification (2026-09-05)

Raised by the maintainer. angloform already dogfoods *some* of its own
description — every ADR since the 2026-09-03 naturalness pass is
written in angloform itself (`docs/dogfood-sweep.md` tracks the parse
rate), and today's `skills/angloform/repair-prompt.md` is a first
system-prompt-level example, 100% self-parsing. What's not yet
self-hosting: `CONTEXT.md`, `README.md`, `skills/angloform/SKILL.md`
(the general onboarding doc — confirmed today at only 4% self-parsing,
see `docs/prompt-ab.md`), and this file itself, `docs/ideas.md`, are
all plain English. The idea is to push toward the *entire* canonical
spec — grammar reference, onboarding doc, and the meta-documents that
describe the project to a newcomer — being written in angloform,
analogous to a self-hosting compiler: the language's own description
becomes the strongest possible dogfood test of whether the language is
actually sufficient for real technical writing, not just for content
someone already chose because it happened to be expressible.

Real tension worth resolving before committing to this, not
glossed over: `SKILL.md`'s prose is *instructions to a reader who does
not yet know angloform* — the repair-prompt.md experiment this session
showed that writing tight, dogfooded instructional prose is possible,
but it required real design work per sentence (see the sentence-by-
sentence rewrite log in this session's transcript) and dropped
scope (Conditional, Step Block) to fit a word budget. A fully self-
hosting `SKILL.md` is the same effort at ~4x the length, and it isn't
free of chicken-and-egg risk: a document teaching someone the language
is unusually exposed to the language's own current expressiveness
gaps (see `docs/language-gaps.md`), since it can't route around a gap
the way a narrower document can. Worth scoping as its own project, not
folded silently into unrelated work.

## Vector embeddings as measurement tooling

Constraint: embeddings may serve *measurement and suggestion*, never the
language definition (ADR 0001's check-don't-choose discipline — model weights
are not auditable data). Candidate uses, ranked:

1. **Meaning-preservation guard for corpus/pairs.tsv** — sentence-embedding
   cosine between the English and angloform sides; catches silent meaning
   loss (e.g. dropped quantifiers) that lexicon-validity checks cannot see.
   Caveat (2026-09-01): pooled embeddings are close to order-blind — a
   subject/object swap lands almost on top of the original — so cosine
   cannot judge role changes. For the faithfulness gate proper, see
   "Structured repair", NLI.
2. **Redirect findability scoring** — rank substitute candidates by
   similarity-to-rejected-sense × frequency, operationalizing ADR 0008.
3. **Homonymy vs polysemy detection** — cluster a word's contextual
   embeddings across corpus uses; distant clusters = dangerous homonymy.
   Better instrument than WordNet sense counts, which overcount.
4. **Pair mining** — retrieve sentences semantically near already-translatable
   ones to raise sampling yield.

Licensing: GloVe vectors (PDDL) and sentence-transformers models (Apache)
are clean; CC BY-SA vector sets fit the existing fetch-not-vendor pattern.

## Core lexicon + jargon addon packs

**Realized 2026-09-02 as the domain model (ADR 0027)**: `domain/model.json` is
the first pack — same schema as the seed plus definitions; adds, never
overrides. Other-domain packs remain future work.

Split the seed into a domain-neutral core (function words + general
vocabulary) and optional domain packs (tech, cooking, …). An angloform text
declares core + packs; lexgen lints the combined set.

Key principle if/when built: **packs add, never override** — a pack may not
shadow a core surface form or re-enable a sense the core rejects, so any
core+packs combination remains one unambiguous language. Core redirects may
only point at core words.

Deferred (2026-08-31) because the lexicon is ~50 lemmas and one layer; the
split earns its keep only once a second genuine domain exists. Justification
when built should cite our own triage data (function words dominate OOV;
current vocabulary is effectively a tech pack), not external CNL specs.

## Curation phasing (active decision, 2026-09-01)

Initial dogfooding period: curate vocabulary for **our own jargon** (the
angloform project's ADRs, docs, software/linguistics register). Dogfood
coverage is the primary curation metric; EWT/general-register triage stays
reported but is telemetry, not a target. Later, **re-center** by testing on
different text types (Simple English Wikipedia, instructions, other
domains) and rebalance the lexicon — the core/packs split (above) is the
likely mechanism when that day comes.

## Compile parsed sentences to pseudocode (2026-09-01)

Mechanical reserialization of the tier-1 parse tree into pseudocode: the
head-annotated tree already carries subject/verb/object/PP roles, so
"if the test fails, then the agent retries the request" folds mechanically
into `if fails(the test): retries(the agent, the request)`. Precedent: the
ACE→first-order-logic tradition — angloform's version targets code-shaped
output instead of DRS/FOL. Motivating use: a self-healing feedback loop for
coding agents — instructions written in angloform are certified unambiguous,
then compiled to pseudocode the agent (or a checker) can execute or diff
against its actual behavior, closing the loop. Cheap to prototype: one
recursive function over `Tree` in the grammar crate.

Extension (2026-09-01): compile to *runnable* analysis code, not just
pseudocode display — each sentence becomes a small executable
check/assertion. Then multi-sentence texts get machine analysis for free:
consistency checking across sentences (two rules that contradict), coverage
("which corpus cases does this policy text actually decide?"), and the
self-healing agent loop (diff stated rules against observed behavior).

## Quotation as a construction (2026-09-01)

ADR 0018 quotes cover verbatim identifiers only (one opaque thing). True
quotation — mentioning sentences/phrases as language — is a separate future
construction: the quoted span is *mentioned*, not used, and could be parsed
recursively (a quoted angloform sentence validated as angloform; a quoted
foreign string held opaque). Needed for meta-prose ("the writer types
\"…\""), error-message docs, and eventually angloform talking about its own
rules. Distinguish delimiters or introduce a marker when designed.

## Linter advice gaps (found building the showcase, 2026-09-01)

1. Banned words with messages: "it fails" → generic WORD flag; ADR 0002
   promised "repeat the noun you mean". Needs the banned-word-with-message
   mechanism (ADR 0008 consequence) — a ban table beside the reject table.
2. Dormant redirects on inflected forms: "the agent files the report" —
   "files" lexes as the enabled NOUN_PL, so the file→submit VERB redirect
   never surfaces. The diagnose layer should notice a noun in verb position
   whose lemma has a rejected-VERB entry and surface the suggestion.
   **Partially fixed 2026-09-02** (slot findings in diagnose: verb after a
   determiner → NOUN redirect; noun between subject and object → VERB
   redirect) — but only the plain-SVO, right-after-the-subject position;
   never re-verified against every bare-verb-form position, so it stayed
   silently broken after a modal, a negation, or sentence-initial
   ("the agent must not file the report" fell through to unrelated,
   sometimes contradictory generic findings instead). **Actually resolved
   2026-09-04**: widened the trigger to every bare-verb position
   (`introduces_bare_verb` in `crates/diagnose/src/lib.rs`) and added
   `suppress_superseded` to drop the now-superseded generic findings for
   the same word. The structural version (role assignment) stays in
   "Structured repair".

## Imperative-input advice gap (agenttest run 4, 2026-09-01)

"Remove the file" went 0/3 first-try: the sentence-initial-capital error
says "a name cannot start a sentence", but the writer meant an imperative
with an OOV verb. The message should offer both readings: "…or if this is a
command, angloform has no bare imperatives — write 'you must <verb> …' or
'do not <verb> …'". Also: whether bare positive imperatives deserve a
sanctioned form is now a data-backed design question.

## Structured repair: structure enumeration + table-driven rewrite (2026-09-01)

Design sketch for turning the linter from a rejecter into a corrector,
deterministic first, model-assisted last. Pruned to the parts that fit this
project; everything else was considered and dropped.

1. **Structure enumerator.** The tier-1 grammar is a cycle-free CFG and
   LR(1)-unique, so parse trees with N leaves ↔ accepted sequences of N
   form-tags, a finite set. Bottom-up DP over the expanded grammar (the
   same shape as diagnose's Tier2 counter) lists them all; brute force over
   38^N is not needed. ~2 h. Yields the complete inventory of sentence
   shapes per length — a testable artifact on its own.
2. **Role assignment on failure.** Each word has a small candidate tag set:
   an enabled word gets its form-tag plus every rejected sense in its
   redirect entry (*files* = NOUN_PL, or VERB via the reject row); an
   unknown word gets any open-class slot. Match the words' tag sets against
   the enumerated structures (permutations of the multiset). Every match is
   parse-valid by construction and names the role each word was playing.
   Gap messages become "needs a verb here", not "not an angloform word".
3. **Table-driven rewrite with explanation.** With roles known, a word in a
   rejected sense takes the redirect synonym inflected to the slot's form
   from its generated paradigm (*files* in a verb slot → *submits*); a
   banned sense leaves a hole with the rephrase advice; an unknown word
   keeps its assigned role. Output: a valid sentence plus one line per
   changed word — role chosen, reject row that fired, curation note. All
   data already lives in the seed. This retires advice gap #2 above rather
   than patching it, and it makes the per-sense-synonym policy (ADR 0023)
   structurally necessary: every rejected sense with a findable synonym must
   carry it, or the transducer has a hole where a suggestion belongs.
4. **Edit-distance repair** (later): add paradigm swaps and closed-class
   insert/delete (*the*, *then*, *does not*, comma), bounded at two edits.
   Covers the failure classes the agent cases actually show (dropped
   *then*, agreement, passive→active shape). Only here does the candidate
   set grow enough to need a ranker.

## Antiparsers: a prototype for steps 1–2 above (2026-09-04)

Working prototype in `crates/antiparse` (isolated, not wired into
`diagnose` yet). Motivating question: the generic linter fallback
("restructure into one of the angloform templates") happens because
`pattern_findings`/`slot_findings` are token-window heuristics — "if the
token before looks like X" — which is exactly why the redirect-check bug
above existed (the guard didn't enumerate every real position) and why it
could misfire (a coincidental adjacency, not a real structural match). An
**antiparser** is a small grammar that recognizes one known-invalid
construction *structurally* — a real parse of the bad shape, not a guess.
This is the standard compiler-construction technique of **error
productions**, applied here as fully separate grammars rather than merged
into one, for a reason this project has hit twice already: ADR 0040 found
real reduce/reduce conflicts trying to add coordinated NPs *inside* the
existing grammar, purely from shared structure. Keeping each antiparser
its own file, its own `extern` block, no shared nonterminals, sidesteps
that: each is independently checked for conflicts by `lalrpop`'s own
build-time gate, and adding one can never reconflict another.

**What was built and verified:**
- 3 independent `.lalrpop` grammars (`AntiBareCoordObject`,
  `AntiNounVerbSlot`, `AntiFreeOnly`) — all build conflict-free together.
- A span-search harness (`antiparse::scan`): tries each antiparser at
  every substring of the token stream, since a bad construction is often
  embedded in a larger valid sentence, not the whole thing. O(n²) per
  antiparser; measured 1.87ms for a 29-token compound sentence — a
  non-issue at real sentence lengths (a bulk pass would bound span length
  per antiparser instead, but didn't need to for this prototype).
- Ranking by proximity to the Tier-1 failure position (`ParseError`
  already carries this for free) — the cheap heuristic for "most likely
  writer intent" when multiple antiparsers match different spans.
- 5 integration tests against the real lexicon, all passing.

**The repair-mapping question** (can a match be turned into a fixed valid
tree, not just explained): empirically, matches split into exactly the 3
categories the `Repair` enum encodes, and they line up precisely with
ADR 0008's Redirect-vs-Ban distinction:
- **Single** (one deterministic fix) — when the underlying issue is a
  *Redirect*: `AntiNounVerbSlot` always has one, because the substitution
  data already lives in the seed's reject table (*files* → *submit*).
  `AntiBareCoordObject` has one *only* when both conjuncts already carry
  their own determiner — then the colon-list construction (ADR 0041)
  is always a valid rewrite.
- **Menu** (several candidates, can't auto-pick) — `AntiFreeOnly`: ADR
  0047 bans the free position specifically because it's scope-ambiguous
  between subject and object; the antiparser can name both candidate
  fixes but not choose, because choosing would be guessing the writer's
  actual intent.
- **None** (no safe repair, explain why) — `AntiBareCoordObject` when a
  conjunct is elliptical ("the old file and reports"): the missing
  determiner/modifier is genuinely not recoverable from the sentence
  alone, so the honest move is to say why, not invent content.

This is the same shape as ADR 0008's Ban policy ("prefer a Ban to a rare
Redirect") applied to whole constructions instead of single words — and
it means an antiparser's own parse tree already carries the role
assignment Structured Repair's step 2 needs (which conjunct, which slot,
which lemma) for free, from a real grammar match rather than a brute-force
permutation search over the enumerated structure set. Building
antiparsers for the highest-value bans would very plausibly retire
Structured Repair's steps 1–2 rather than duplicate them — step 3
(table-driven rewrite) is what's still separately needed, and this
prototype's `Repair::Single` case already does exactly that for the two
constructions it covers.

**Scope found empirically, not assumed:** banned *words* (pronouns,
epistemic hedges, "may") never reach this pipeline — `Lexicon::tokenize`
rejects them via the `bans` table before a token stream exists at all.
Antiparsers are relevant only to *structural* mistakes, where every word
tokenizes fine and the arrangement is what's rejected. That's a real
scope boundary, not a current limitation to remove.

**What the hint-authoring workflow would need to become**, if this is
built out past the prototype: right now a new hint is "notice a failure,
write a Rust token-window check." An antiparser-based system changes the
unit of authoring to "one small grammar file per Ban/ADR, with an
`extern` block naming only the tokens it needs, plus a `repair()`
function returning one of the 3 categories." That's more structured and
far more systematic (every entry in `domain/model.json`'s `Ban`/`Gap`
list is a mechanical candidate to transcribe into a grammar file, since
each already has documented example sentences to test against) — but
it's still a per-construction authoring cost, not free; it converts an ad
hoc "notice and patch" workflow into an enumerable backlog, which is a
real improvement for problem 2 (coverage lag) without claiming to solve
coverage completely.

**Wired in 2026-09-04**: `antiparse::scan()` is now a fourth channel in
`diagnose()` — when `pattern_findings`/`slot_findings` find nothing, the
antiparsers run before the generic fallback, ranked by proximity to
Tier-1's actual failure position (`grammar::parse_tokens` +
`failure_position`, added for this). Confirmed a genuine, previously-
uncovered improvement, not just infrastructure: "the mechanism only
stores the report" (ADR 0047, the free-`only`-position ban) had no
dedicated pattern check and used to fall through to "restructure into
one of the angloform templates" — it now gets `[AntiFreeOnly] ambiguous —
pick one: move "only" before the subject …; move "only" before the
object of "stores" …`. `AntiBareCoordObject`/`AntiNounVerbSlot` mostly
overlap with checks already fixed earlier the same session, so they
don't visibly change those specific cases — expected, not a problem.

**Frequency instrumented, 2026-09-05**: `just finding-frequency`
(`crates/diagnose/src/bin/finding-frequency.rs`) runs every sentence in
`tests/paragraph-cases/*.yaml` + `tests/agent-cases/*.yaml` (6643
sentences — real LLM repair-attempt text, the population that actually
matters for prioritizing antiparsers) plus the UD-EWT corpus (2077
sentences, kept only for comparison — 97.6% fail at the WORD level
before reaching structural analysis at all, confirming EWT is the wrong
source for this question) through `diagnose()`, buckets STYLE findings
by template (quoted spans normalized to `X`), and writes
`docs/finding-frequency-report.md`. The generic fallback itself only
fires 28/6643 times (0.4%) — most rejections already get a specific
explanation; the antiparser backlog should be prioritized off the
ranking in the report, not off the fallback's own (rare) firing.

**`AntiClauseObject` built, 2026-09-05 — the ranking caught a real bug,
not just a coverage gap.** The #2-ranked finding, "a clause cannot be
the object of a verb" (473× — 7% of all near-miss sentences), turned
out to be ~98.5% false positives. The old `pattern_findings` heuristic
("two verb-ish tokens, no connective between them") misfired on
ordinary predicates: do-support negation ("the system **does** not
**have** an anaphora mechanism"), modal + main verb ("agents **must**
not **check** the input"), copula + passive participle ("the file **is**
**stored** in the database") — each is exactly two verb-ish tokens with
nothing between them, and the heuristic had no way to tell that apart
from a genuine embedded clause. Manually checked ~40 real trigger
sentences from the corpus; found one true positive shape ("the report
**shows** the Pronouns **are** banned") in the whole sample. Replaced
with `crates/antiparse/src/anti_clause_object.lalrpop`
(`AntiClauseObject`, a 4th antiparser): the real structural signal is a
subject-like NP *wedged between* the two verbs — that NP is what starts
the second clause, and an aux/modal/copula never has an NP between it
and its own main verb, so the grammar structurally cannot match a single
predicate. Rerunning the frequency tool: the bucket dropped from 473 to
7 genuine matches, and total STYLE findings dropped 1336 → 1284 (the 52
difference: sentences that used to get this spurious "explanation" but
had no other real problem now correctly fall through, mostly to the
generic fallback, which rose 23 → 28 — an honest result, not a
regression: those sentences never had a specific cause to name).
`Repair` category: `None` (never a mechanical fix — which sentence to
split into is a decision only the writer can make, same as
`bare_coord`'s elliptical-conjunct case). This is the concrete case that
validates the whole antiparser thesis from the original design
discussion: a token-window heuristic can't just be "usually right, rare
misfire" — "two verb-ish tokens" is the *normal* shape of most English
predicates, so any heuristic built on adjacency alone was doomed to
misfire on the common case, not the edge case. Only a real structural
match (an actual second subject) tells the difference.

Building this surfaced an unrelated bug: `crates/diagnose` gained a
second `[[bin]]` target (`src/bin/finding-frequency.rs`, next to the
crate's existing `src/main.rs`), which made every `cargo run -p diagnose`
call site ambiguous and silently empty under `|| true` (`scripts/
showcase.sh` regenerated `docs/showcase.md` down to its first 5 lines).
Fixed by pinning all four call sites (`scripts/showcase.sh`, `scripts/
lint-file.py`, `scripts/dogfood-sweep.py`, `justfile`'s `lint` recipe) to
`--bin diagnose` explicitly — a standing gotcha for any future second
binary added to a crate that already has a `main.rs`.

**Determiner-omission and noun-noun-compounds audited, 2026-09-05 — both
clean, no antiparser needed.** Sampled 40 real trigger sentences each
from the near-miss corpus for both checks (same method as the
clause-object audit above). Every single sample in both was a genuine
violation ("clarity"/"naturalness"/"prose"/"anaphora" used bare as mass
nouns; "anaphora mechanism"/"discourse mechanism"/"discourse tools" as
real noun-noun compounds) — zero false positives found in either. So
unlike clause-as-object, these two token-window heuristics turn out to
already be structurally reliable in practice: the adjacency they check
for (immediately-preceding determiner slot; immediately-adjacent noun
pair) has no other common English shape that coincidentally produces
the same token pattern, unlike "two verb-ish tokens" which is the
*normal* shape of most predicates. Lesson: the antiparser thesis
("token-window heuristics misfire because of coincidental adjacency")
doesn't mean every heuristic is suspect — it applies specifically when
the checked-for token pattern is itself common in valid constructions.
Cleaned up the determiner-omission check's two pieces of dead/tautological
code found during the audit (`crates/diagnose/src/lib.rs`): the
`(j == 0 && i > 0 && false)` branch (always false — contributed
nothing) and the `(i > 0 || sentence_initial_bare)` guard (always true,
since `i == 0` forces `j == 0` by construction) — pure clarity cleanup,
verified behavior-identical by the full test suite and an unchanged
`finding-frequency` STYLE count.

**Still not done**: decide whether `Repair::Single` results get
surfaced as "try: X" suggestions only, or auto-applied in a
repair-proposal flow (matching `autofix-paragraphs`'s existing
human-verdict gate, `just verdict`, rather than silently rewriting
anything). The remaining ranked findings ("is a defined term" 121×,
inline-list-should-be-Enumeration 114×, "is transitive and needs an
object" 108×) haven't been audited yet — same method as above before
building anything for them.
5. **Faithfulness gate: bidirectional NLI** (later, and its real value is
   not ranking). A candidate is a faithful rewrite only if input ⊨ candidate
   and candidate ⊨ input; a role swap fails one direction. This mechanizes
   the ADR 0012 loss review that is a human verdict on every dogfood pair
   and agent snapshot today. Local, deterministic given the model, explains
   itself as "does not entail". Dependency-parse role comparison (UD-scheme
   parser over the fetched EWT) is the fallback for unknown-word roles;
   marginal while gaps are being closed by curation.

Dropped: embeddings as a ranker (order-blind, see above; keep only as a
coarse gap detector), cross-encoders (duplicate NLI without the
explanation), LM log-probability ranking (spends API budget on a question
the redirect table already answers), late-interaction retrieval (a search
problem this project does not have).

## Paragraph repair: fix-and-compare in context (agreed design, 2026-09-01)

Sentence-level fixes can be right in isolation and wrong for the paragraph
(given-before-new, topic continuity). Agreed flow, to build after the
coherence measurement has run:

1. `just autofix` sends each **paragraph** to the repair loop with the
   per-sentence linter verdicts, the skill, and the neighbouring sentences.
2. The model returns a rewritten paragraph plus a declared-drops line
   (ADR 0012 contract, as in corpus/pairs.tsv).
3. The tool lints every sentence of the proposal, computes the document
   metrics (parse rate, topic continuity, relation inventory) for original
   and proposal, and writes one YAML per paragraph in `tests/paragraph-cases/`
   (original, every trial's proposal, metrics, `verdict`) — the agent-cases
   pattern at paragraph scale.
4. The report renders original and proposals side by side with metric
   deltas, ranked parse-rate → continuity → cost. Display order only, never
   a gate. Accepted rewrites are applied by hand until verdicts show the
   proposals are trustworthy.

## Medium-aware rendering (spoken vs. written), an accessibility feature (2026-09-03)

The block structures (Enumeration, ADR 0028; Step Block, ADR 0034) and some
sentence shapes carry structure that written text shows with layout — dashes,
line breaks, Gherkin keywords, quotes, capitals for terms. Spoken output
(TTS, screen readers) has no layout. Because every angloform unit is
statically parseable, the conversion can be static too: a renderer walks the
parse tree and emits the form for a *medium*. Examples: an Enumeration
becomes "the language allows 4 pronouns: I, you, my, your" with spoken
pauses or "first / second" cues; a Step Block reads its keywords aloud
("given …, when …, then …"); a quoted Name is announced as a name; a
Capitalized term can be announced as a defined term on first mention.
Design questions: which shapes need a distinct spoken form; whether the
written form is the canonical one and spoken forms are derived (yes, by
default); whether a "medium" is a property of a document or of a renderer.
Not a language change — a rendering layer over the parse tree.

## Domain model as a knowledge graph (maintainer's direction, 2026-09-03)

Recorded verbatim in substance, to be designed through interviews:

1. **Kind of item.** Every domain-model entry must say whether it is a
   *unique item* (Lexgen, the Seed) or a *category of things* (Anaphoric
   Pronoun, Scale Word). A category **must carry a set of examples**.
2. **Membership.** An item can be a member of a category, and a category can
   be a member of a category: "Login Page is an Administration Website
   Feature"; "Administration Website Feature is a Company Asset". The model
   becomes a graph of is-a links, not a flat glossary.
3. **Imaginability.** Experts explain how things work (the curse of
   knowledge); beginners understand how things look. A text is readable and
   engaging when the reader can imagine the things. The lexicon and the
   structures should steer toward descriptions that are easy to imagine —
   concrete nouns, examples, appearance before mechanism. Candidate levers:
   a required example per category (1); a definition shape that leads with
   what a thing looks like; a linter signal for abstract-noun density.
4. **Articles.** Each domain-model item should be a knowledge-base article
   in its own right, interlinkable with other articles — including articles
   that do not participate in the language system at all. The generated
   CONTEXT.md is the seed of that: one entry per term, links by name.

Archetype A21 (definition as an equation) is the first place this bites:
today's definitions are equations ("the Context Need of a sentence is the
prior text of the sentence") because the model has no slot for kind,
examples, or membership.

## Parseability derives an inter-document semantic graph (2026-09-03)

Every angloform sentence has exactly one parse (ADR 0014's guarantee), and
every domain-model term is a named node with a `kind`, `examples`, and a
`member_of` link (ADR 0036). That combination is more than a linter: a
document's parse trees are structured data, so a tool could walk every ADR
and every definition and extract a graph automatically, rather than one
maintained by hand.

Candidate edges, all derivable from the parse tree plus the lexicon, no
new annotation required:
- **is-a / member-of**: already explicit via `member_of`, but also
  recoverable from copula sentences ("X is a Y") anywhere in the corpus,
  not only in domain/model.json.
- **mentions**: a document's parse trees name a Capitalized term → an edge
  from the document to that term's domain-model node (what CONTEXT.md's
  "links by name" gestures at today, but not extracted as data).
- **decides / extends / supersedes**: ADRs already say this about each
  other in prose ("Extends ADR 0003", "supersedes …") — a fixed sentence
  shape for cross-ADR relations would make this a queryable edge instead
  of free text.
- **argument structure**: angloform's fixed slot grammar (subject, verb,
  object, PP) means a sentence like "the Linter rejects the Rejected
  Sense" is already a (subject, predicate, object) triple with no
  extraction ambiguity — closer to RDF than free English ever gets.

This would turn the knowledge base into something closer to a queryable
graph than a set of cross-linked pages: "what depends on ADR 0014", "which
terms have no example", "which ADRs mention Copula" become graph queries
over parse output, not `grep`. Not a language change — a tool that reads
the existing parse trees and the domain model and emits a graph (nodes:
documents + terms; edges: the four kinds above). Natural next step after
the domain-model schema (ADR 0036) settles: a `just graph` command that
walks `docs/adr/*.md` and `domain/model.json` through `grammar::parse_text`
and prints edges, likely as a first cut before any storage or UI question.


## angloform as a code-normal-form target (2026-09-03)

If a compiler/language AST can be mapped to angloform sentences unambiguously
(one AST shape → one sentence shape, invertibly, the same guarantee ADR
0014 gives natural-language sentences), the language stops being only a
prose-writing constraint and becomes a normalization target for code
itself: "the function `parse` takes the argument `text` and returns the
tree" rather than a lossy free-text comment about the function.

Two distinct uses follow from that mapping, not one:
- **Linting code structure through the angloform linter.** Once code is
  projected into angloform sentences, every diagnose() rule (bare-noun
  ambiguity, dangling reference, missing quantifier scope) becomes a
  structural check on the code's own shape, not just its prose comments —
  a naming/structure smell shows up as an angloform STYLE finding on the
  generated sentence for a function or a type.
- **Generating a normalized form of the code.** The reverse direction: use
  the mapping to produce canonical, angloform-vocabulary names and
  descriptions for a codebase's declarations — a forced pass through
  angloform's per-sense-synonym discipline (ADR 0023) as a naming-quality
  gate, the same discipline this project already applies to its own prose.

Open questions before this is more than a note: which AST shapes actually
have a natural angloform sentence (a function call maps cleanly to a
Statement's subject-verb-object; a loop or a closure less obviously so);
whether the mapping needs to be total (every AST node has a sentence) or
only covers a useful subset (declarations and signatures, not arbitrary
expressions); and whether "unambiguous" here means the same thing ADR 0014
means for parsing prose, or a weaker one-way (AST → sentence) guarantee
without the reverse. Not sized yet — a research question, not a plan.

## Code+comment redundancy via the code-normal-form round-trip (2026-09-06)

Raised by the maintainer, extending "angloform as a code-normal-form
target" above. That section covers two uses (lint code structure through
angloform; generate normalized names/descriptions). A third use follows
from the same code AST ↔ angloform AST ↔ code AST round-trip: treat the
generated angloform sentence for a declaration and its **existing
human-written comment** as two independent descriptions of the same
thing, and diff them.

The idea: for a function/type with a doc comment, generate the angloform
sentence from the code AST, then compare it against the comment (also
reduced to angloform, or judged for overlap directly). Content present in
both is redundant — the comment is re-stating what the signature already
says, a real and common code-quality smell. Content only in the comment
is exactly the information the code cannot carry on its own (why, not
what — a constraint, a caller assumption, a workaround rationale). Content
only in the derived sentence but absent from the comment is a gap the
reviewer might want filled the other way. The round-trip turns "does this
comment just repeat the function signature" from a subjective review
judgment into a mechanical diff.

Once redundancy is visible, the natural next step is a repair pass: keep
the code carrying the *what* (already unambiguous from types/names once
normalized per the code-normal-form idea) and rewrite the comment to
carry only the *why*/context that the code genuinely cannot express —
the same content-distribution problem this project already solves for
angloform prose itself (ADR 0012's declared-loss discipline, Structured
Repair's role assignment) applied across the code/comment boundary
instead of within one text.

Open questions, inherited from the parent idea plus new ones: which AST
shapes get a natural angloform sentence at all (signatures are the easy
case); whether "redundant" should be judged by angloform-vs-angloform
sentence equivalence (cheap, structural) or needs the NLI-based
faithfulness gate from "Antiparsers" (more expensive, catches paraphrase-
level redundancy plain string diff would miss); and whether this becomes
a linter check (flag redundant comments) or a repair tool (rewrite them).
Not sized — a research question riding on the parent idea's own open
questions, not yet a plan.

## An algebra of semantics: structured compositional vectors as an alternative to learned embeddings (2026-09-06)

Raised by the maintainer. "Vector embeddings as measurement tooling"
above already flags a blind spot in *learned* sentence embeddings: pooled
vectors are close to order-blind, so a subject/object swap lands almost
on top of the original — useless for judging whether a rewrite changed
who does what. The idea here is a different kind of vector entirely, not
a fix to the learned one.

Each word (or each lexicon sense) gets a **structured, auditable basis
vector** — not learned, not opaque, in the spirit of ADR 0001's
check-don't-choose discipline ("model weights are not auditable data")
that already rules out learned embeddings as anything but measurement.
A one-hot-per-lemma starting point is the simplest version; the real
design question is whether the coordinates should instead track
domain-model categories/roles so the arithmetic means something (see
below). Sentence meaning becomes a combination of its words' vectors
under some fixed, inspectable combination rule — addition/subtraction
being the motivating case: given "a big red vehicle crashed into the
river," subtracting **river** and **red** should land on the same vector
as "a big vehicle crashed"; adding **wall** and **loud** back should land
on "a big loud vehicle crashed into the wall." Two syntactically
different but meaning-equivalent angloform sentences would be required to
reduce to the identical vector — a mechanizable equivalence check no
learned embedding gives for free (per the order-blindness caveat above).

Why this might be uniquely tractable *for angloform specifically*, more
than for general English: angloform already guarantees one parse per
sentence (ADR 0014) and already extracts argument-structure triples for
free from that parse (see "Parseability derives an inter-document
semantic graph" above) — the combination rule could be defined over
parse-tree roles (subject/verb/object/PP-modifier) rather than a naive
bag-of-words sum, which is exactly the missing ingredient the pooled-
embedding order-blindness caveat is missing today.

Open questions, not resolved: what the basis coordinates actually are
(raw lemmas vs. domain-model categories/roles — the choice determines
whether "big" and "loud" combine the same way a modifier always does, or
each needs its own attachment-specific rule); whether addition/subtraction
needs to be role-aware (removing an object PP is a structurally different
edit than removing an attributive adjective, even though the example
above treats them uniformly); and what this buys over directly editing
the parse tree, which already gives exact compositional structure without
reducing to vectors at all — the vector algebra's real value would have
to be in operations tree-editing doesn't give for free (e.g., "does this
independently-written rewrite denote the same meaning," a comparison
question, not an editing one). Not sized — a research question, connects
to the antiparser/repair-mapping faithfulness work as a candidate cheap
equivalence check, not yet a plan.

## Property-based testing with proptest (2026-09-03)

Full survey, prompted by wanting a systematic way to develop the grammar
and linter more effectively — the ADR 0037 restructuring (CoordClause vs.
Causal, an LALR state-merge conflict) was found and fixed by hand, one
sentence at a time; a generator-driven property test would have surfaced
the same conflict from a single generated example, with proptest's
shrinking reducing it to the minimal repro automatically. `proptest = "1.11"`
is now a dev-dependency of `grammar`, `diagnose`, and `lexgen`.

**Built (2026-09-03):**

1. **`crates/lexgen/src/morph.rs`** — crash-freedom on arbitrary Unicode
   input, plus structural invariants regular English morphology always
   satisfies regardless of the input lemma: `pluralize`/`third_singular`
   end in "s", `past` ends in "d", `gerund` ends in "ing", `comparative`
   (when `Some`) ends in "er", and no rule ever shrinks its input. These
   don't check correctness (seedcheck.py's attested-form check against real
   corpora does that) — they check *shape*, and catch the class of bug the
   file's own doc-comment already worried about (a byte-length vs.
   char-count mismatch in `doubled()` on non-ASCII input).
2. **`crates/grammar/tests/fuzz_properties.rs`** — `tokenize`, `units`,
   `is_enumeration`/`is_step_block`, and `parse`/`parse_text` never panic on
   arbitrary text (not just well-formed angloform); `units` is idempotent on
   its own output; the two block classifiers are mutually exclusive. These
   matter because every one of these functions runs on real LLM-repair
   proposals and arbitrary document prose in production use, not just
   curated corpus lines.
3. **`crates/diagnose/tests/diagnosis.rs`** — `diagnose()` never panics on
   arbitrary text, same rationale (agenttest feeds it raw model output).
4. **`crates/diagnose/tests/proptest_generated.rs`** — the highest-value
   one: a generator built from real lexicon words (read live from
   `lexicon.tsv`, never hardcoded) producing valid statements, including
   the `CoordPred`/`CoordClause` shapes from ADR 0037, then asserting every
   generated sentence parses AND that Tier-2 never rejects what Tier-1
   accepts (generalizing the fixed-corpus `tier2_is_a_superset_of_tier1`
   test to a much larger generated sample). Deliberately narrow — one verb
   shape, one NP shape — not a generator for the whole grammar.

**Identified, not yet built, ranked by expected value:**

5. **A full sentence-shape generator** covering every `Sentence` production
   (Conditional, Causal, Prohibition, Imperative, quantified subjects,
   Complements, Enumeration/Step Block), not just the plain-statement slice
   in `proptest_generated.rs`. This is the natural extension of #4 and the
   biggest remaining win — it would make *every* future grammar change
   get this kind of regression coverage for free, not just Coordination.
   Sized at a day or so: mostly plumbing existing lexicon-tag categories
   into `prop_oneof!` strategies, one production at a time.
6. **Round-trip property**: for a tree produced by the generator in #5,
   `Tree::render()` then reparse should reproduce an equivalent tree (or
   at minimum, the same `Diagnosis::Clean`). Catches renderer/pretty-print
   drift that snapshot tests only catch for the fixed corpus.
7. **`units()` round-trip over generated documents**: join N independently
   generated valid sentences (from #4/#5) with realistic paragraph
   structure and assert `units()` recovers exactly N units. Complements
   the idempotence property already built (#2) with a genuine multi-unit
   check.
8. **Lexicon-collision detection as a property test**, not just a
   build-time script assertion: generate seed entries with a deliberately
   induced surface-form collision across categories and assert `lexgen`'s
   collision lint always fires. Lower priority — seed entries are hand-
   curated, not adversarial, so this guards against a future refactor of
   the lint itself weakening it, not against real collisions.
9. **`number_token` (ADR 0022/0024/0029 digit/ordinal/percent/decimal
   parsing) fuzzing**: crash-freedom on arbitrary digit-shaped strings, and
   a round-trip property (format a generated number, parse it back to the
   same value) — this function has the most intricate branching logic of
   any single-purpose parser in the codebase and currently has no fuzz
   coverage at all.
10. **Domain-model schema fuzzing (ADR 0036)**: generate `domain/model.json`
    entries with randomly malformed `kind`/`examples`/`member_of` fields and
    assert `lexgen` always rejects them with a named error, never a panic
    or a silent pass-through. Lower priority than #9; the schema is small
    and hand-validated already.

**Deliberately not pursued:** fuzzing `seedcheck.py` or other Python
tooling — proptest is Rust-only; the Python side would need Hypothesis
instead, a separate tool with a separate learning cost, not justified by
the size of that surface. Property-testing `nd()`'s head-slot invariant
directly was also considered and dropped: it's already a local `assert!`
checked on every call site through the existing corpus and generated
tests (#4), so a standalone property test would mostly duplicate that
coverage without covering anything the assert doesn't already catch.

## Sequence-level surprisal (Markov chain / n-gram) as a naturalness metric (2026-09-05)

Raised by the maintainer while reviewing the naturalness-iteration work
(`docs/naturalness-iteration-2026-09-05.md`). `textcost`'s existing cost
model uses *unigram* surprisal — already known to have a blind spot
(`docs/research/cnl-design-findings.md`, "Translator compression bias,
and the nominalization blind spot": unigram surprisal prices a word by
its own frequency alone, so it cannot see that a *sequence* of common
words can still be highly predictable, or highly surprising, as a
sequence).

That gap is exactly what today's naturalness-ceiling finding is made
of: "The word 'if' opens a Conditional. The word 'do' opens a
Prohibition..." — every individual word is common (low unigram
surprisal, i.e. "cheap" by the current metric), but the *sequence*
repeats the identical template 7 times, which is maximally predictable
and reads as mechanical for exactly that reason. A blind human/LLM
judge caught this immediately; the project's own cost metric currently
cannot, because it never looks past single-token frequency.

**The idea**: a low-order Markov chain (bigram or trigram) trained
over the existing angloform corpus (`tests/paragraph-cases`,
`tests/agent-cases`, the ADRs themselves — already-collected data, no
new corpus needed) gives a cheap, local, fully auditable
per-transition surprisal score, without needing an external LM. Two
uses:

1. **A real naturalness proxy, cheaper than a blind LLM judge.** A
   sentence or paragraph whose *sequence*-level surprisal is
   abnormally low (near-zero entropy, i.e. the reader could predict
   almost every next word from the template alone) is a strong,
   mechanically-detectable signal for the "reads like a list" failure
   this session kept re-finding by expensive blind judging. This could
   flag candidate mechanical runs automatically (a `finding-frequency`-
   style tool), narrowing what needs an expensive blind-judge pass.
2. **An optimization signal for language design**, with the same
   guardrail ADR 0001 and the vector-embeddings section above already
   established: measurement and suggestion only, never a substitute for
   the check-don't-choose discipline that decides what the language
   *allows*. A construction that would raise average sequence-entropy
   corpus-wide is a data point in favor of designing it (e.g. the
   still-missing table/mapping construction from today's session), not
   a rule that auto-generates one.

Not yet built. Needs: (a) confirm the existing corpus is large enough
for a stable bigram/trigram model (angloform's whole-corpus vocabulary
is small and closed, which cuts both ways — sparse data, but also a
much smaller space to estimate over than open English); (b) decide
whether surprisal is computed over surface tokens or over the parse
tree's grammatical categories (the latter would catch "same Sentence
Shape repeated" even when the actual words vary, which is closer to
what today's judges were actually reacting to — see the "varied verb"
trial in `docs/naturalness-iteration-2026-09-05.md`, which still scored
2/5 despite different words, because the *structure* repeated).

# A lexicon authoring format with no ad hoc waivers (design, 2026-09-07)

Requested directly: design a lexicon authoring format satisfying 6
requirements. Not a grilling interview output — a concrete proposal,
written to be argued with. Motivated by a real, found problem: the
current `waive`/`reject`+advice mechanism let an agent bulk-generate
~460 free-text justifications in one pass with no human review,
directly against this project's own "a machine does not curate the
Seed" principle (see `domain/model.json`'s "curate" entry). The fix
below isn't a patch on that mechanism — it removes the thing that made
the bulk-generation possible in the first place: free-text authoring.

## The core move

Every current escape hatch (`waive`, `reject: "<word>"`,
`reject_advice: "<free text>"`) is **free text a human writes once,
per collision, by hand**. That's exactly the surface a machine can
mass-produce without anyone noticing. The fix is structural: **delete
free-text authoring from the per-word schema entirely.** What a
maintainer authors per word is small, structured, and mechanically
checkable (a category, a definition, a sense key). What gets *shown*
to a writer who hits a rejection — the suggestion list — is **computed
at report/lint time** from the whole lexicon's already-curated
definitions, never authored per collision. If bulk-generation is
attempted under this design, there is no free-text field for it to
land in.

## Schema

```json
{
  "lemma": "produce",
  "category": "VERB_TRANS",
  "sense": "produce.v.02",
  "definition": "the Lexer produces the token, so the Lexicon holds the token.",
  "forms": { "third": "produces", "past": "produced", "ing": "producing" }
}
```

Four fields do all the work:

- **`category`** — exactly one, as today. No `reject`, no `waive`, no
  per-word bookkeeping about other senses at all. (Requirement 2.)
- **`sense`** — a WordNet sense key (`produce.v.02`) naming *which*
  synset this lemma's enabled meaning corresponds to. Machine-checked
  against `refdata` at `lexgen` time — must be a real, attested sense
  of this lemma, not a free choice. This is the one new authoring cost,
  and it's small and structured: pick the right sense ID, not write a
  paragraph.
- **`definition`** — the word's meaning, **written in Angloform**,
  checked the same way an ADR is (`just lint`). Every word gets one,
  not just domain-model terms. (Requirement 5.)
- **`forms`** — unchanged.

That's the entire per-word authoring surface. Nothing here can express
"this word also means X in another category, and here's why we're
ignoring that" — there's no field for it.

## Where the "reject" behavior went: computed, not authored

`lexgen` already loads a WordNet index (`refdata.rs`) and already
knows, for any lemma, every POS it's attested under (this is exactly
what today's cross-POS lint already checks). Requirement 4 ("targeted
hints without hand-writing them") falls out of combining that with the
new `sense` field:

1. A writer uses "copy" as a verb. "copy" is enabled as `NOUN` only —
   rejection.
2. The Linter looks up **every synset WordNet attests for "copy" as a
   verb** (there are several — `duplicate.v.01`, `imitate.v.03`, …).
3. For each of those synsets, it searches the **currently-enabled
   lexicon** for any word whose own `sense` field names that synset,
   or a synset related to it (WordNet's own synonym/near-synonym
   edges — no new relation data needed, `refdata` already parses the
   WordNet index this would read from).
4. Every match becomes a suggestion line, and — this is the part that
   makes it free — **the suggestion's own already-curated `definition`
   field is the gloss shown**. Nothing new is written for the hint;
   it reuses text a maintainer already wrote and already lint-checked
   as valid Angloform when *that* word was authored.

Worked example, matching the format you asked for exactly:

```
The word "copy" is not allowed as a verb. Suggestions depending on meaning:
- "duplicate": the tool duplicates the file, so 2 copies of the file exist.
- "imitate": a student imitates the teacher, so the student copies the method of the teacher.
```

Both suggestion lines are literally each word's own `definition` field
— authored once, ever, when "duplicate" and "imitate" were curated as
words in their own right, for their own sake, not as a reject-hint for
"copy." (Requirements 4 and 6, and the "must themselves be valid
Angloform" constraint — they inherit it for free from requirement 5.)

## Requirement 1: no ad hoc waivers, including the real fallback case

Two cases, both structural, neither free text:

- **A synset-neighbor exists among enabled words** → the computed
  suggestion list above. Always current (recomputed from live lexicon
  state, never goes stale the way a hand-written note can).
- **No enabled word covers that sense** → this is a real, nameable
  **Gap** — the project already has this concept (used for deferred
  capability elsewhere, e.g. ADR 0022's units/negative-numbers gaps).
  The Linter reports it structurally: *"no Angloform word currently
  expresses this meaning of 'copy'"* — a fixed, templated sentence
  naming the lemma and POS, not a hand-authored explanation of *what*
  the missing sense means. That last part matters and is worth being
  honest about below.

No field anywhere holds free text a maintainer writes per collision.
A Gap is a computed absence, not an authored waiver.

**The honest limit of this design**: the fixed Gap message *cannot*
describe the missing meaning itself in the way today's hand-written
advice does (*"the noun sense (an assistant to a vicar or rector) is
not enabled"*) without either (a) quoting a raw WordNet gloss, which is
not Angloform and reintroduces exactly the free-text-authoring surface
this design removes, or (b) a maintainer hand-writing that description,
which is authoring-per-collision again, just relabeled. I'd rather say
this plainly than paper over it: **a Gap's fixed message is genuinely
less informative than today's best hand-written advice text.** What it
buys back is that it can never be bulk-generated, never goes stale,
and costs zero authoring. If a specific Gap turns out to matter a lot
in practice, the real fix is what this project already does elsewhere
— curate a new word for that sense (a genuine ADR-worthy decision), not
write a better waiver.

## Requirement 3: rare true exceptions go to the grammar, not the lexer

The schema above has no per-word "except in this structural position"
field, on purpose — that pressure needs to go somewhere, and the
project already has a working pattern for it: a second, dedicated Form
Tag reused by a specific grammar rule, not a lexicon-level dual-sense
flag. Precedent already in the grammar: ADR 0016's "one" (`NUM_SG`
distinct from ordinary determiners), ADR 0056's superlative split
(short-inflecting vs. long-periphrastic, two Form Tags, one grammar
slot). A genuinely dual-behaving word gets a **second `category`+`sense`
entry under a distinct Form Tag**, consumed by name in exactly the
grammar productions that need it — same mechanism already used
throughout `angloform.lalrpop`, not a new concept. The lexicon-level
invariant ("one category per lemma," requirement 2) stays intact
because this is a second *lemma+tag pair*, structurally visible at the
grammar level, not a hidden second sense living inside one entry.

## What this costs, honestly

- **Every one of ~980 existing lemmas needs a `sense` key and an
  Angloform `definition` retroactively.** This is a real, large
  migration — bigger up front than the thing it replaces. It's a
  one-time structured cost (pick a sense ID, write one lint-checked
  sentence) rather than a recurring free-text cost, but it is not
  small, and it should not be done in one bulk automated pass either
  — that would just reproduce the exact problem this design exists to
  prevent. It needs the same treatment as any other curation work:
  reviewed a batch at a time, by a maintainer, same as the original
  seed always was.
- **The suggestion engine is new code**, not free: a synset-neighbor
  search over the live lexicon at lint/report time. Bounded and cheap
  (the lexicon is ~1000 words, WordNet lookups are already loaded in
  memory for `lexgen`), but it's a real component to build and test,
  not just a schema change.
- **Sense-key correctness becomes a real curation surface.** Picking
  the wrong WordNet sense ID for a word is a new failure mode this
  design introduces — worth a lint check (e.g., does the definition's
  own content plausibly match the sense gloss?) rather than assuming
  the sense key is always right just because it's structured.

## Validated by trial (2026-09-07)

Before committing to this design, tried it on 7 real existing words
(produce, hold, copy, duplicate, society, file, rare) — hand-write a
non-self-referential Angloform definition, check it with `just lint`,
iterate on real errors. Result: **7/7 eventually passed**, 1–4 attempts
each. The friction was real and useful, not noise:

- Ordinary words assumed to already exist often don't: plain "name"
  (noun — only the capitalized domain term "Name" is enabled), "live"
  (verb) and the verb sense of "share" aren't in the lexicon at all;
  "large", "under", "low" aren't enabled.
- A real grammar-coverage bug, unrelated to the lexicon format: a bare
  plural noun inside an of-PP doesn't parse ("the group of people" ✗,
  "the group of the people" ✓).
- Self-reference is an easy, real trap (caught once, defining "rare"
  using "rare").
- The weakest result, "society" (*"the group of the people has a
  place"*), is exactly the failure mode the next section exists to
  close structurally: technically valid, substantively thin — no
  differentia at all, just a genus with a throwaway predicate.

Conclusion: the constraint is realistic, but only once base vocabulary
is padded out — confirms the doc's original migration warning
(batch-reviewed, not bulk-generated) from direct evidence, not just
prior reasoning.

## A dedicated grammar entry point for definitions (2026-09-07)

The trial's weakest result exposed the real gap: `pub Sentence` answers
"is this valid Angloform," not "is this an adequate *definition*." Those
are different questions, and conflating them is how "the group of the
people has a place" passed as a definition of "society" — every word
in it is fine, the *shape* just isn't a definition.

**The fix: dedicated start symbols, used only by tooling, never by
ordinary prose** — the same move already made for `pub Intro`,
`pub Step`, and `pub Item` in `angloform.lalrpop`, not a new
architectural pattern. **Correction (2026-09-08)**: the sketch below
originally assumed a reusable `RelClauseSG`/`RelClausePL` fragment —
that's not what ADR 0059 actually shipped. The relative clause is
`RelSentence`, a *Sentence-level* construct requiring a full `Clause`
(Subject + Predicate) before the comma, using `PredRel` for the
embedded predicate — not an NP-internal modifier fragment (see
`docs/relative-clause-design-2026-09-06.md` for why: the fragment
version hit a real LALR(1) conflict and was abandoned). `NounDef`
below is corrected to match what's actually built, and skips the
`"word" means "…"` recursive-quote wrapper — not needed for the actual
near-term use case (`lexgen` validating the `definition` field
directly, as its own plain string, category already known from the
same JSON entry), so it's deferred rather than built speculatively:

```
pub NounDef: Tree = {
    // genus + MANDATORY differentia. REVISED (2026-09-08): originally
    // wrapped the genus in a full copula clause ("Subj is a Genus,
    // which…"), reusing RelSentence's shape. Dropped the wrapper
    // entirely — real dictionary style needs no restated subject
    // ("a group, which has a history," not "a society is a group,
    // which..."). Safe to attach the differentia straight to a bare
    // NP here specifically because NounDef is a standalone `pub`
    // symbol, never reachable from `Sentence`'s own shared machinery —
    // the exact same bare-NP shape caused ADR 0059's real LALR
    // conflict when it was embedded inside VPn's object slot,
    // competing with CoordClause/Causal for the same comma at the
    // Sentence level. Verified conflict-free here (`cargo build -p
    // grammar`), not assumed safe just by analogy.
    <genus:NPSGCore> <cm:LComma> <r:LWhich|LWho> <p:PredRel>,
};

// a bare verb phrase, no subject — real dictionary style ("to build a
// file from parts"), reusing VBaseP (ADR 0019's Imperative nonterminal).
pub VerbTransDef: Tree = VBaseP;
```

`lexgen` picks which `pub` entry point to call based on the word's own
`category` field — a `NOUN` entry's `definition` string must parse via
`NounDefParser`, not just via the generic `SentenceParser`.

**Built and verified (2026-09-08), in an isolated worktree
(`../minglish-def-grammar`, branch `def-grammar-entry-points`), same
lesson as ADR 0059 — not just reasoned about.** `cargo build -p
grammar`: zero conflicts. The antecedent-ambiguity question above
turned out moot, not because it was answered but because the whole
open question was cut — see below.

**`VerbTransDef` was built, tested against real words, and rebuilt
twice before landing here — worth the full story, not just the final
shape.**

*Attempt 1* (in this doc originally): `<svo:Clause> <cm:LComma>
<s:LSo> <purpose:Clause>` — mandatory purpose/result clause, "no bare
'X builds a thing,' it has to say what for." Built, tested against the
full 7-word trial set: **0/7 passed unchanged** (expected — none were
written for this template). Rewriting them surfaced the real problem:
*"a tool builds a thing, so a new thing exists."* **passes** — still
just as vague, the mandatory clause is gameable — while *"a tool
builds a file from parts."*, a genuinely more concrete definition,
**fails**, purely for lacking a purpose clause it didn't need. The
template was both too weak (didn't prevent vagueness) and too strong
(rejected good definitions).

*Attempt 2*: drop the mandatory Causal tail, require only a plain
transitive `Clause` — fixes the too-strong half, leaves the real
vagueness problem (`"a thing"`, `"an action"`) exactly where it was,
unaddressed by grammar, same split ADR 0054's Consequences already
drew for its own vague-noun problem (a CFG can't tell "term" from
"Function Word" apart — that's `diagnose::vague_appositive`'s job, a
Tier-2 lint, not a grammar rule).

*Attempt 3, shipped*: **drop the subject entirely.** Real dictionary
entries never restate a placeholder subject (*"to build a file from
parts,"* not *"a tool builds a file from parts"*) — reusing `VBaseP`
(the exact nonterminal `Imperative` already uses, ADR 0019, zero new
grammar) makes `"a tool builds…"`-style filler subjects **not just
discouraged but ungrammatical**: `VBaseP` starts on a bare verb, so a
determiner can never occupy that position. This is the stronger fix,
matching this project's own established principle for exactly this
class of problem (`docs/garden-paths-2026-09-06.md`: *"the best tool
is to just not allow the construction in the grammar, if there is a
way to do that"*) — not a Linter check catching vague subjects after
the fact, a construction that can't produce one. A vague *object*
("a thing") still can't be caught this way (no CFG can distinguish
"thing" from "file" as nouns) — that half stays a genuine, deferred
Linter item, `diagnose::vague_appositive`'s job, unchanged from
Attempt 2's honest assessment.

This also **resolved the antecedent-ambiguity open question above by
elimination**, not by answering it: `VerbTransDef` no longer touches
`Clause`/`RelSentence`/`PredRel` at all, so the question of whether a
copula's Complement dodges ADR 0059's antecedent gap only applies to
`NounDef` now, not to verb definitions — one fewer place that gap can
bite, for free.

**Final verified state, the full original 7-word trial set, every one
rewritten and passing — pushed one round further (2026-09-08) after
real critique**: the first "final" pass still restated a subject for
every verb and wrapped every noun in a copula clause. Neither is
necessary — `NounDef` doesn't need the copula wrapper (see the
revision above), and the verb definitions were over-narrowed to a
specific domain object ("a file") for two verbs whose actual meaning
isn't file-specific at all ("produce", "hold" — generalize to "a
thing"). One hard limit found and worth stating plainly: a fully
object-less verb definition ("build from parts") is **not** achievable
for any `VERB_TRANS` word — transitivity is fixed per-word in this
language (one category per word), so an object is always grammatically
mandatory, definitions included; the shortest available form is a
maximally generic object, not no object at all.

| Word | Original (failed the strict template) | Final, shortest correct form |
|---|---|---|
| produce | *a tool builds a thing from parts.* | *build a thing from parts.* |
| hold | *the folder keeps the file.* | *keep a thing.* |
| copy | *a second file is identical to a first file.* | *a file, which is identical to the first file.* |
| duplicate | *a tool makes a second file. the second file is identical to the first file.* | *make a copy of a thing.* |
| society | *the group of the people has a place.* | *a group, which has a history.* |
| file | *a tool stores the data with a Name.* | *a Name, which holds the data.* |
| rare | *a word has a small frequency in the corpus.* | *a rare word is not common.* |

This turns "does this read like a real dictionary entry" from a
per-word human judgment call into a mechanical, per-category grammar
check — the single highest-leverage addition to this whole design,
since it's what actually prevents another "society"-style thin
definition from silently passing, verified end to end on real words,
not just in the abstract.

## Three more things enforced structurally, not left to discipline (2026-09-08)

Real critique of the 14-word set surfaced real problems, all fixed the
same way this whole design insists on — grammar/lint-level, not human
trust:

1. **Trailing period, banned outright.** Previously tolerated either
   way (the tokenizer drops one if present, never required it). Now
   `parse_definition` rejects any definition ending in `.` — a
   definition is a fragment, not a sentence, and every entry point
   already produces a fragment (`NounDef`'s bare NP, `VerbTransDef`'s
   subjectless VP, `AdjDef`'s bare adjective) — restating sentence
   punctuation on a fragment was always inconsistent, just never
   enforced.
2. **Self-reference, banned by lemma, not by eyeballing.** Two of the
   14 trial definitions were caught being genuinely, embarrassingly
   circular: *"sit in a position"* (defining "sit") and *"fail without
   a reason"* (defining "fail") both used the headword as their own
   definition's main verb. `parse_definition` now takes the `lemma`
   being defined as a real parameter and rejects any definition
   containing a word whose lemma matches — caught its own second
   mistake immediately (an early attempt at this fix accidentally
   defined "small" as `"small"`). Real replacements: "sit" → *"stay in
   a position"*, "fail" → *"end without a result"*.
3. **`AdjDef` dropped its subject and copula too** — the exact same
   simplification already applied to `NounDef` (dropped the copula
   wrapper) and `VerbTransDef` (dropped the subject), now applied
   consistently to the third category. The original *"a rare word is
   not common"* forced picking an arbitrary, often wrong-domain subject
   noun ("word") just to have somewhere to hang the copula — the same
   over-narrowing problem the verbs had with "a tool"/"a file". Now:
   `pub AdjDef: Tree = { LAdj, <n:LNeg> <a:LAdj> }` — bare, optionally
   negated, nothing else. "rare" → *"not common"*, "small" → *"not
   big"*. A comparative/PP-standard form ("bigger than X") isn't
   supported yet — add it only once a real word needs it, not
   speculatively.

4. **Domain-model-jargon leakage, banned by term lookup.** Two more
   real trial mistakes, both defining an ordinary content word using
   Angloform's own internal meta-vocabulary — a category error, not a
   style nitpick: *"a file is a **Name**, which holds the data"*
   (defining "file" using ADR 0018's own term for an unquoted
   identifier) and *"a person, who curates the **Seed**"* (defining
   "maintainer" using the project's own name for its curated word-list
   source file). `Lexicon` already exposes `term(lowercase) ->
   Option<&str>` (domain-model lookup, ADR 0027/0049); `parse_definition`
   now rejects any definition word that resolves to a real domain term.
   Real replacements: "file" → *"a thing, which holds the data"*,
   "maintainer" → *"a person, who chooses a word"*. Found immediately
   after being built — the very first full-set test run caught "Seed"
   on its own, unprompted.

Verified: `cargo build -p grammar` clean, `cargo test --workspace`
zero regressions, all 14 definitions rewritten to the new, stricter,
period-free, self-reference-free, subject-free, jargon-free form.

## Extended to 14 words, plus VerbIntransDef (2026-09-08)

Two more things settled before extending the trial: **no trailing
period is required** (the tokenizer already drops one if present but
never demands it — confirmed by test, no grammar change needed), and
**`VerbIntransDef` needed zero new grammar**, same story as
`VerbTransDef` — `VBaseP` already has both a transitive and an
intransitive branch (that's how `Imperative` already handles either
kind of verb), and the transitive/intransitive split is already
enforced elsewhere: a word's Form Tag is fixed (one category per
word), so the wrong branch is simply unreachable for a given word's own
head verb. `pub VerbIntransDef: Tree = VBaseP;` — a distinctly-named
entry point purely for `lexgen`'s dispatch-by-category clarity, not a
different grammar.

7 more real words, spanning all 4 categories now (2 nouns, 2
VERB_TRANS, 2 VERB_INTRANS — the first real use of the new entry
point, 1 adj):

| Word | Category | Definition |
|---|---|---|
| maintainer | NOUN | *a person, who curates the Seed.* |
| report | NOUN | *a file, which names a result.* |
| delete | VERB_TRANS | *remove a file.* |
| read | VERB_TRANS | *parse a file.* |
| sit | VERB_INTRANS | *sit in a position.* |
| fail | VERB_INTRANS | *fail without a reason.* |
| small | ADJ | *a small file is not big.* |

**All 7 passed on the first attempt — zero retries**, a real contrast
to the original 7 (1-4 retries each). That's evidence the templates
themselves had converged after the earlier rounds of critique and
revision, not just that these particular words happened to be easy.

## The synset-membership data exists, already fully licensed (2026-09-08)

`data/wordnet/index.*` (vendored today) is only word→synset-*count* —
today's code can attest "produce has 6 verb senses" but not what any
of them actually are. The full synset membership, glosses, and
hypernym/hyponym/similar-to pointers live in `data.noun`/`data.verb`/
`data.adj`/`data.adv`, from the exact same already-fetched, already-
checksummed Princeton `WNdb-3.0.tar.gz` — confirmed by pulling the
archive fresh and finding all 4 `data.*` files sitting right next to
the already-vendored `index.*` files. Not a new source, not a new
license decision — the archive was already trusted, just partially
unpacked.

Tried it for real on the trial words. "copy" as a verb has 4 real
WordNet senses:

```
{copy, re-create}         — make a replica of
{replicate, copy}         — reproduce or make an exact copy of
{imitate, copy, simulate} — reproduce someone's behavior or looks
{copy}                    — copy down as is
```

This resolves the ranking question below with real evidence, not a
guess, and the answer is less simple than hoped: "copy" and "imitate"
**share a synset** (`{imitate, copy, simulate}`) — an exact match, the
easy case a naive same-synset lookup handles for free. "copy" and
"duplicate" (the other half of the worked example earlier in this
doc) **never share a synset** — they're separate, topically-related
synsets, findable only via at least one hop of hypernym-chain
proximity. **Exact synset co-membership alone would silently under-
suggest** — it'd surface "imitate" and miss "duplicate," even in the
example this whole feature was scoped around. The suggestion engine
needs graph-distance, not just set membership.

## A fifth schema field: `antonym`, curated not derived (2026-09-08)

Trying to translate WordNet's own glosses directly (not just look up
synonyms) surfaced this. Literal translation doesn't work — every
gloss needed real restructuring, never a word-for-word substitution
(`"not widely known"` → "widely" isn't enabled; `"make or do or
perform again"` → "performs"/"again" aren't enabled either). But one
rewrite landed on something better than anything gloss-guided
composition produced: *"a rare word is not common."* — simpler, exact,
non-self-referential.

Checked whether WordNet's own antonym pointer (`!`) would have
surfaced "common" mechanically: **it doesn't — none of "rare"'s 6
adjective senses carry an antonym pointer at all**, even though this
is about as obvious a pairing as English has. WordNet's antonym
relation is real but sparse; it can't be relied on as a free,
computed field the way `sense` mostly can.

So: a fifth schema field, structured (a lemma reference, checked to be
a real enabled Angloform word — not free text), curated by the
maintainer in the same pass as `sense`+`definition`, not a separate
sweep:

```json
{
  "lemma": "rare",
  "category": "ADJ",
  "sense": "rare.a.02",
  "definition": "a rare word is not common.",
  "antonym": "common"
}
```

**Why this doesn't reopen the original bug**: it's optional (most
nouns and many verbs have no clean binary antonym — "file," "produce,"
"society" don't get one), and it rides the same batched, human-
reviewed migration process `definition` already requires — never a
free-text field a script could bulk-fill unsupervised. **Where it's
genuinely weaker than the rest of the schema, worth saying plainly**:
`sense` is machine-checkable against real attestation data (a binary
yes/no); `antonym` mostly isn't — check it against WordNet's `!`
pointer when one happens to exist, but for the common case (no
pointer, exactly like "rare"), there's no oracle at all, just the
curator's judgment, with `lexgen` only able to verify the named word
is a real, enabled lemma — structural validity, not semantic
correctness.

**Feeds the `AdjDef` grammar shape directly**: when `antonym` is
present, the negated-contrast form (*"a rare thing is not common."*)
becomes the grammar-recognized *preferred* shape for `AdjDef`, not an
afterthought — needs no new grammar mechanism, `Compl` already
supports `LNeg?` on a copula.

## "delete" and a real, honest structural gap (2026-09-08)

*"remove a file"* had the same over-narrowing problem the earlier
verbs had (produce/hold's "a file" → "a thing"), but a generic object
alone loses something real: "delete" specifically means *permanent*
removal, unlike plain "remove." Checked whether that nuance is
expressible at all before proposing a fix, rather than assuming: it
isn't, for two compounding, real reasons — `VerbTransDef` (bare
`VBaseP`) **has no adverb slot at all** (even ADR 0044's tiny medial-
adverb set only applies to the full `Predicate`, never to bare
`VBaseP`), and **no word for "entirely"/"completely"/"permanently"/
"forever" exists in the lexicon** — "whole" exists, but its adverbial
sense is explicitly banned (*"angloform has no adverb category
regardless"*). Not a phrasing gap, a real structural one, worth stating
plainly rather than forcing an awkward workaround.

Fixed by paraphrase instead, using only already-real vocabulary:
*"remove a thing without a copy"* — captures the permanence (no
recoverable duplicate left behind) precisely by cross-referencing
"copy," a word this same trial already defined. Not a universal fix
for "how do you express permanence" — a one-off solution for this one
word, using what already existed.

## "maintainer" over-narrowed to this project's own activity (2026-09-08)

*"a person, who chooses a word"* — technically not a domain-model term
(passed that check clean), but still project-specific: describes
curating vocabulary, this project's own maintainers' actual daily
work, not the general English meaning of "maintainer" (upkeep of any
project). A quieter version of the same over-narrowing pattern as
produce/hold/delete's object, just at the differentia level instead of
the object. Fixed: *"a person, who decides things in a project"* —
general, no project-specific activity implied.

## Five more definitions critiqued on meaning, not just grammar (2026-09-08)

Every earlier round caught *structural* problems (jargon, self-reference,
over-narrow objects). This round is different: five definitions were
grammatically fine but semantically off — accurate-sounding but wrong or
misleading about what the word actually means. None of these are caught
by any check in `parse_definition`; only a human reading the definition
against the word caught them.

- **society**: *"a group, which has a history"* — too generic; almost
  any group has a history, so it fails to pick out what makes a
  society a society. Fixed: *"a group, which follows rules"* — uses the
  existing `follow` verb + `rule` noun, captures the shared-governance
  sense that "has a history" missed.
- **hold**: *"keep a thing"* — grammatical, but loses hold's specific
  sense (a fixed grasp/position), collapsing it into a near-duplicate
  of "keep" itself. Fixed: *"keep a thing in a position"* — reuses the
  `position` noun already validated for "sit," adds back the
  fixed-in-place sense "keep" alone doesn't carry.
- **produce**: *"build a thing from parts"* — overspecified in the
  wrong direction: true for manufacturing, false for "produce a
  report" or "produce evidence." A definition that's accurate for one
  narrow case reads as *misleading* for the general word. Fixed:
  *"make a thing"* — the actually-generic core sense, matching how
  "duplicate" is already defined ("make a copy of a thing").
- **delete**: *"remove a thing without a copy"* — the previous round's
  cross-reference to "copy" was creative but reads backwards (sounds
  like removing something that *lacks* a copy, not removing something
  *and* leaving no copy behind). The underlying gap (no way to express
  permanence — see the "structural gap" section above) is still real
  and still unsolved; the paraphrase just wasn't a good enough fix for
  it. Simplified instead to the honest, unembellished *"remove a
  thing"* — correct as far as it goes, silently short of capturing
  permanence. The gap stays open rather than papered over.
- **read**: *"parse a file"* — wrong for the general word: a human
  reading a book doesn't "parse a file." Fixed: *"get a meaning from a
  text"* — generic across human and machine reading, using `get`,
  `meaning`, and `text`, none of which were in the lexicon-scan for
  this feature before now.

Verified: all five re-parse under the same grammar/lint checks as
before (10/10 tests, `cargo test --workspace` clean). No grammar or
lint change was needed for this round — every fix was a wording change
using only already-real lexicon words. This is the round that most
clearly shows the format's remaining weak point: definitions can be
100% structurally valid and still be a *bad* definition. That check has
no mechanical substitute — it needs a human (or an LLM judge) reading
definition against word, every time, same as this round did.

## Round 2: two more real gaps, one "don't touch this" (2026-09-08)

- **hold**: proposed *"keep a thing in a position or place"* — rejected.
  Checked the grammar: NP-level "or" coordination only exists in one
  narrow, colon-marked construction at the whole-object position
  (`V: N1 or N2`), not inside an ordinary `PPv`. There is no way to
  write "in X or Y" as a prepositional phrase anywhere in Angloform
  today. Real structural gap, same class as the missing adverb slot —
  left as *"keep a thing in a position"*, undocumented alternative
  dropped rather than forced.
- **duplicate**: *"make a copy of a thing"* → *"make a second copy of a
  thing"*. "another" doesn't exist in the lexicon — Angloform has no
  general indefinite determiner beyond a/an/the — but the ordinal
  `second` (already in the lexicon, ADR 0029's ordinal NP shape) fills
  the same role and parses cleanly.
  - Raised: should "something" become a valid generic object,
    alongside/instead of "a thing"? **Recommendation: no.** "a thing"
    already fills exactly this role, and adding a second, grammatically
    different way to say the same thing (a bare pronoun vs. an NP) is
    the identical problem this trial just flagged for delete/remove
    below — a synonym with no distinct meaning. Not adopted.
- **society**: *"a group, which follows rules"* → *"a human group,
  which follows rules"*. The fix wasn't a new mechanism — `NPx` (the
  production every determined NP goes through) already carries an
  optional adjective slot (`<a:LAdj?>`), unused by every definition in
  this trial so far purely because none of the genus nouns needed
  narrowing. `human` was already in the lexicon.
- **file**: *"a thing, which holds the data"* → *"a thing, which holds
  items"*. The forced "the" wasn't a wording choice — `data` is
  lexically `NOUN_SG` and Angloform has no bare-singular/mass-noun
  production (only bare *plural* nouns can drop their determiner,
  `BarePl`). So "holds data" with no article is currently
  ungrammatical, full stop. Fixed by using a real bare plural
  (`items`) instead of fighting the gap.
- **delete vs. remove**: after the previous round's fix, both words'
  definitions read as pure synonyms ("remove a thing" describes both).
  Checked whether to resolve this by removing "remove" from the
  lexicon entirely — real usage check first, not a guess: `remove`
  appears 71 times and `delete` 92 times across the already-dogfooded
  ADRs and docs. Both are established, load-bearing words; deleting
  either is a large breaking change to real prose, completely out of
  scope for a definition-format trial. **Recommendation: leave the
  lexicon as-is.** The actual problem is narrower than "which word to
  cut" — it's the same missing-permanence-adverb gap from the previous
  round, and it stays open for the same reason: no adverb mechanism
  exists yet to say what "delete" adds over "remove". A prototype
  definition field colliding on two words is an acceptable, documented
  shortfall; rewriting the real lexicon's word inventory is not this
  trial's call to make.
- **read**: *"get a meaning from a text"* → *"get the meaning from a
  thing"*. Two problems in one: "a meaning" read worse than "the
  meaning" (the specific meaning that thing carries), and "text"
  excluded machine-reading of files the same way the original
  file-specific "parse a file" excluded human reading — the same
  over-narrow-object mistake found and fixed for produce/hold/duplicate
  earlier, just one round later on a different word. Generalizing the
  object down to "a thing" (the same placeholder every other
  over-narrowed definition in this trial converged on) fixes both at
  once.

Verified: 10/10 tests, `cargo test --workspace` clean, no grammar or
lint change needed — every fix and every declined fix in this round
came from either real lexicon words or a real, checked grammar
limitation, not a guess.

## Round 3: "human" wasn't the point, and the "or" gap resurfaces (2026-09-08)

- **society**: *"a human group, which follows rules"* — missed the
  point. "human" answers "a group of what," but the flagged nuance was
  breadth/generality (a society is a broad, general grouping, not any
  small human group — a family is a human group too). `NPx`'s
  adjective slot only holds one adjective, so stacking "general" and
  "human" together wasn't an option. Fixed by changing the genus noun
  itself instead of the adjective: *"a community, which follows
  rules"* — `community` already carries the breadth "group" lacked, no
  adjective needed.
- **read**: proposed *"get the meaning from a file or a text"* — the
  same NP-level "or" gap from Round 2 (hold's "position or place")
  applies here too: no PP can hold a disjunction, so this exact phrase
  doesn't parse. The underlying ask was fair, though — "a thing" was
  criticized (correctly) in Round 2 for being too vague to reassure
  that file-reading is covered, but a genuine two-word disjunction
  isn't available to fix that. Compromise: *"get the meaning from a
  document"* — a single word covering written material generally
  (files included, in the loose sense a computer file is a kind of
  document) without either the bare vagueness of "a thing" or an
  ungrammatical "or". Flagged as a compromise, not a confirmed-exact
  fit — worth a second look before this trial is trusted as final.

Verified: 10/10 tests, `cargo test --workspace` clean.

## A real, simple, uncontroversial gap: "of" + a bare plural (2026-09-08)

Asked directly: is there an uncontroversial *grammar* gap behind
society being hard to define well, rather than just a wording problem?
Checked instead of guessing again. Found one.

`OfPP` (the "of ..." modifier every genus NP can carry, e.g. "a group
of X") routes its object through `NPInner`, which requires a
determiner on every branch — `NPi<LDet, LNounPl>` demands "the
people," never bare "people." So "a group of people" (the single most
natural way to say what a society is made of) was **ungrammatical**,
not just unwritten — the same missing-bare-plural gap already found
for `file`'s "holds items," but in a different NP position (`OfPP`'s
object, not the direct object).

Unlike the adverb/OR gaps (Rounds 2-3), this one was cheap and
low-risk to actually close: `BarePl` (the bare-plural production
already used at the top level, e.g. `file`'s "holds items") added as
one more alternative to `NPInner`. Built clean, zero LALR conflicts;
full workspace suite (all crates, not just `definition_grammar`) still
green afterward. This isn't a definitions-only fix — any ordinary
sentence with an "of"-PP over a generic plural now parses too (a real
capability gain, not just a definitions-format patch).

With that fixed, **society**: *"a community, which follows rules"* →
*"a group of people, which has a tradition"* — states what it's made
of ("of people," now grammatical) and what holds it together over time
("a tradition"), instead of leaning on a single vague adjective or
swapping to a near-synonym noun.

Verified: 10/10 `definition_grammar` tests, `cargo test --workspace`
clean across every crate.

## 10 more words, chosen to stress specific edges (2026-09-08)

Picked deliberately, not at random — each targets something the first
14 didn't exercise:

| lemma | kind | definition | what it stresses |
|---|---|---|---|
| write | VERB_TRANS | make words | plain object, sanity check |
| compare | VERB_TRANS | find a difference between things | a 2-argument verb, expressed via object + `PPv` |
| choose | VERB_TRANS | take a thing from things | `PPv` object is a bare plural (already legal at top level) |
| build | VERB_TRANS | make a thing from parts | **deliberate near-synonym of `produce`** ("make a thing") |
| avoid | VERB_TRANS | prevent a thing | plain object, sanity check |
| exist | VERB_INTRANS | stay in the world | **kept in as an honest miss** — see below |
| arrive | VERB_INTRANS | come to a place | plain intransitive + `PPv` |
| agent | NOUN | a person, who helps a person | genus/differentia, no domain-jargon leak |
| output | NOUN | a thing, which comes from a process | genus/differentia via intransitive `PredRel` |
| different | ADJ | not identical | **antonym via an existing ADR** — ADR 0023 bans "same" outright and names "identical" as the correct substitute for two things matching; this definition is that substitution rule turned directly into an Angloform definition |

All 10 parse; 11/11 `definition_grammar` tests, `cargo test --workspace`
clean. No grammar or lint change needed for this round.

Two results worth calling out:

- **build vs. produce**: last round's `delete`/`remove` pair collapsed
  to identical definitions because the real distinguishing feature
  (permanence) has no expressible mechanism yet. `build`/`produce`
  don't have that problem — `build` keeps the "from parts" differentia
  `produce` shed for being over-narrow, and that's *correct* here:
  "build" really does specifically mean assembly from parts (you build
  a house from materials, you don't build a decision), where "produce"
  is the more general verb. Same surface pattern as delete/remove
  (two near-synonyms, one specific one general), opposite outcome —
  proof the collapse isn't inevitable, it depends on whether the
  language actually has an expressible differentia.
- **exist**: *"stay in the world"* is weak and left in deliberately,
  not polished. "stay" implies duration/continuation, not existence
  itself — a real paraphrase gap, not a typo. This is the same failure
  mode Round 3 named for `read`/`hold`: a definition can be 100%
  structurally valid (parses, no self-reference, no jargon) and still
  be a bad definition, and "exist" is arguably a harder case than
  those — it may be close to a semantic primitive in this lexicon's
  current vocabulary (no word for "real," "being," or copula-as-content-
  verb exists to define it non-circularly with). Not fixed here on
  purpose, so it stays visible as a real open question rather than
  getting quietly smoothed over.

## Round 4: fixing what's fixable, naming what isn't (2026-09-08)

- **compare**: *"find a difference between things"* → *"find the
  difference between 2 things"*. Two real corrections: "the
  difference" (definite — there's exactly one relevant difference
  between the two things being compared) and, per ADR 0022, counts are
  written as digits, not number words — "two" is banned outright,
  "2" is required. Both already-legal.
- **build**: *"make a thing from parts"* → *"merge parts into a
  thing"*. The proposed *"combine parts to make a new thing"* doesn't
  parse — **Angloform has no infinitival purpose clause at all** ("to
  V ...", the "in order to" sense). Checked the grammar directly: no
  `LTo`/infinitive production exists anywhere, for any construction,
  not just definitions. A real, load-bearing gap, not a
  definition-format limitation — but "combine" also isn't in the
  lexicon at all, so the phrasing needed to change regardless.
  `merge` (already real, `VERB_TRANS_BASE`) captures the same
  differentia — parts becoming one thing — without needing either the
  missing word or the missing construction.
- **write**: *"make words"* → *"mark a thing with letters"*. "make
  words" was too abstract — true of speech, thought, anagram-solving,
  anything word-shaped. "write" specifically means the *physical
  marking* act. `mark` and `letters` were both already in the lexicon,
  unused until now.
- **exist**: *"stay in the world"* → *"occur in the world"*. Still not
  a clean fit ("occur" leans toward "happen at a time," "exist" toward
  "have being") but closer than "stay" (which wrongly implies
  duration/continuation as the defining trait). Left flagged as
  imperfect, same as before — this is very plausibly close to a
  semantic primitive for this lexicon's current vocabulary, not a
  wording problem waiting for the right synonym.
- **agent**: the real critique — "an agent is a person *or* thing,
  that acts" — hit two compounding gaps at once, checked directly
  rather than guessed around:
  - **No generic verb for "acts" exists.** No `act`, `perform`,
    `operate`, or usable `run`/`function` in the lexicon. Plausibly
    deliberate, not an oversight — this project already refuses vague
    light verbs elsewhere (`do` is auxiliary-only, `NEG_AUX_BASE`, not
    a content verb). Adding one just for this definition would cut
    against that.
  - **No NP-level "or" exists at the genus/subject position, at all.**
    Checked `Subj`/`SubjSG`/`SubjPL`: none support coordination. The
    *only* place two NPs can be joined with "or" anywhere in Angloform
    is the one narrow colon-marked object-list construction found in
    Round 2/3 (`V: N1 or N2`) — which only ever appears after a verb,
    never at a genus/subject position. Unlike the `OfPP`/bare-plural
    gap fixed for "society," this would mean touching `NPSGCore`
    itself, which is shared by ordinary sentences project-wide, not a
    narrow, low-traffic sub-production — a real, broader-impact
    change, not a "simple" one.
  - Landed on a compromise for now, not a real fix: *"a thing, which
    causes a thing"* — drops the explicit person/thing disjunction
    entirely (accepting that "thing" alone doesn't convey "could be a
    person or a bot" the way the real definition needs to), but
    captures the causal-actor core of "agent" with existing vocabulary
    (`cause`, already `VERB_TRANS_BASE`). **Flagging this one
    explicitly rather than presenting it as solved** — worth deciding
    whether genus-level "or" is worth building before trusting this
    word's definition.

Verified: 11/11 `definition_grammar` tests, `cargo test --workspace`
clean. No grammar change made this round (unlike Round 4's predecessor
in the "society" section) — every fix here used only existing
vocabulary and existing constructions; the two real gaps found
(infinitival purpose clauses, genus-level "or") were named, not built.

## Round 5: one clean fix, one confirmed dead end (2026-09-08)

- **write**: *"mark a thing with letters"* → *"make a text"*. Simpler
  and more accurate — a text is, by definition, the kind of thing
  writing produces; "mark ... with letters" over-specified the
  mechanism (handwriting on paper) in a way a general "write" (typing,
  writing code, writing a note) doesn't require. `text` was already in
  the lexicon (used for `read`'s Round 1 definition too).
- **exist**: tried *"be in the world"* — doesn't parse, and not for a
  fixable reason. `be` is lexically tagged `BE`, a distinct token
  category from ordinary intransitive verbs (`LViBase`) — it's the
  copula, wired only into copular predicate constructions ("X is Y"),
  never into `VBaseP`'s bare-intransitive-verb slot. Making it usable
  there wouldn't be a small grammar tweak like the `OfPP` fix; it would
  mean giving the copula a second, structurally different role
  (subjectless bare-verb-phrase), which cuts against what the copula
  *is* in this grammar. Left as `occur in the world` (Round 4),
  imperfect but real, rather than force a deeper redesign for one
  word's definition.

Verified: 11/11 `definition_grammar` tests, `cargo test --workspace`
clean.

## Round 6: "or" already exists — at the predicate, not the genus (2026-09-08)

Asked directly: is there a practical fix for agent's "acts" problem
using multiple concrete verbs joined by "or" instead of one abstract
one? Checked rather than assumed — and yes, partially, using a
mechanism already built for something else.

`NounDef` already ends in `<t:Tailn<PredRel>?>`, and `Tailn<P>` is
`<c:LConj> <p:P>` — a *predicate*-level coordination (`LConj` covers
"and"/"but"/"or"). This is the same tail every `Sentence`/`Statement`
already uses for same-subject predicate coordination (ADR 0004) — it
was sitting unused in every `NounDef` definition so far because none
of them needed a second predicate. **agent**: *"a thing, which causes
a thing"* → *"a thing, which builds a thing or removes a thing"* —
approximates "acts" as a disjunction of two concrete, already-real
verbs instead of one missing abstract one. Parses with zero grammar
changes; verified 11/11 tests, `cargo test --workspace` clean.

This only partially closes Round 3's finding, and the boundary is
worth being precise about:
- **Fixed**: the "acts" verb gap — a concrete two-way disjunction now
  stands in for the missing abstract verb.
- **Still open**: the "person or thing" genus gap. `Tailn` coordinates
  *predicates* sharing one subject/genus — it cannot join two
  different genus nouns. The `NPSGCore`-level "or" gap named in Round
  3 is untouched by this fix and remains a separate, bigger question.
- **Still binary-only, by explicit design (ADR 0004), not oversight**:
  checked the grammar comment directly — `Tailn`'s predicate
  coordination is "binary only" by name in the code. A third "or
  writes a thing" isn't reachable through this mechanism at all;
  ADR 0050's n-ary coordination is a different, narrower mechanism (a
  run of 3+ *different-subject* clauses), not applicable here. Two
  disjuncts is the ceiling this construction offers, not a starting
  point to extend casually.

## 10 more words, round 2 — 35 total now (2026-09-08)

| lemma | kind | definition | what it stresses |
|---|---|---|---|
| error | NOUN | a thing, which is wrong | **copula inside a relative predicate** — `PredRelCore` includes `CopPredn<LCopSg>` ("is") alongside the verb branches; never exercised until now |
| tool | NOUN | a thing, which helps a person | domain-relevant word (this project's own docs use "a tool" as the canonical vague-subject example — defined cleanly anyway) |
| word | NOUN | a thing, which has a meaning | Angloform's own meta-vocabulary, defined without circularity or domain-jargon |
| team | NOUN | a group of people, which has a goal | reuses the `of`-bare-plural fix from the "society" round |
| create | VERB_TRANS | make a new thing | see below — near-synonym cluster |
| cause | VERB_TRANS | produce a result | cross-references `produce`, specific object differentiates it |
| describe | VERB_TRANS | name a thing with words | reuses `name` (from `report`'s definition) plus a PPv |
| depend | VERB_INTRANS | come from a thing | |
| belong | VERB_INTRANS | come from a group | |
| empty | ADJ | not full | clean antonym pair, same pattern as rare/small/different |

All 10 parse; 12/12 `definition_grammar` tests, `cargo test --workspace`
clean, no grammar change needed.

**create** is the interesting one: first attempt was *"make a thing"* —
identical, word for word, to `produce`'s own definition. Unlike
delete/remove (where the missing differentia genuinely doesn't exist
in the grammar/vocabulary), this collision *was* avoidable: `new`
(already `ADJ`) slots directly into the object NP's existing optional-
adjective position, giving *"make a **new** thing"* — distinct from
`produce` without inventing anything. Checked before accepting the
collision, rather than accepting the first thing that parsed — the
same discipline that caught delete/remove, applied one step earlier
this time.

35 words trialed total across 8 rounds.

## Round 3: a real grammar fix (verb "or"), and 5 new confirmed gaps (2026-09-08)

Checked every proposal against the grammar directly before answering,
same discipline as every round before this one.

**Fixed — a real, low-risk grammar addition**: `belong`'s proposed
*"come from a community or stay in a place"* failed for a fixable
reason: `VerbTransDef`/`VerbIntransDef` (both just `= VBaseP`) had no
`Tailn` tail at all, unlike `NounDef`. Added the exact same
`Tailn<VBaseP>?` mechanism `NounDef` already uses (ADR 0004's binary
same-subject predicate coordination) to both verb-definition entry
points. Built clean, zero LALR conflicts, full workspace green.
**belong**: → *"come from a community or stay in a place"* — now
parses for real, not approximated.

**Landed, using words already checked to exist**:
- **word**: *"the smallest part of a document, which has a meaning"*
  — exercises the superlative-NP genus (ADR 0056), unused until now.
- **cause**: *"give a reason for a thing"* — the proposed *"be the
  reason of a thing"* hit the same `be`-is-copula-only wall as `exist`
  (Round 5); `give` + `reason` (both already real) capture the same
  idea without it.
- **describe**: *"give a meaning for a thing"* — the proposed *"say
  how something looks or behaves"* failed for two independent reasons,
  not one: **"how" isn't an Angloform word at all** (no embedded
  question/wh-clause vocabulary exists anywhere), and "something" is
  the same missing-indefinite-pronoun gap from Round 2 (duplicate's
  "something" question). `say`/`give` both already real; `give a
  meaning` sidesteps both gaps.
- **depend**: *"need a thing"* — the proposed *"does not exist without
  a thing"* doesn't parse: negation on a bare, subjectless `VBaseP`
  isn't available. Checked why: the only negation production
  (`NegVPn<D>`) always requires a person/number-agreeing `do`-auxiliary
  ("does"/"did"/"do") as `D` — which would force every other
  subjectless definition's whole "no restated subject, no agreement"
  convention (established across every round so far) to break for this
  one word. Real, structural, and arguably *correct* to leave alone —
  a subjectless negated verb phrase isn't a shape English dictionaries
  use either. `need` (already real, transitive) sidesteps it entirely.

**Confirmed real gaps, left as-is — the working prior definition kept**:
- **error**: *"the result of failing"* doesn't parse. `failing` exists
  only as `VERB_INTRANS_ING` (a participle), and no NP position
  anywhere in the grammar — not `NPAny`, not `OfPP`'s `NPInner` even
  after this session's earlier fix — accepts a bare `-ing` form as a
  nominalized gerund. No nominalization mechanism exists in Angloform
  at all. Kept: *"a thing, which is wrong"*.
- **tool**: *"a thing, which a person uses or an agent uses"* doesn't
  parse. `PredRel` (everything after "which"/"who" in a `NounDef`) only
  supports **subject-gapped** relatives — the genus must be the
  *subject* of what follows, never the object of a separately-stated
  subject. This is the same limitation `output` hit in Round 4 ("a
  thing, which a thing produces" wasn't reachable either — fixed then
  by flipping to subject-gapped "comes from"). Kept: *"a thing, which
  helps a person"* (already subject-gapped, already covers the same
  idea from the other direction).
- **team**: *"a group of people with a goal"* doesn't parse. Every
  `NPSGCore`/`NPx` branch's only optional post-modifier is `OfPP`
  ("of ..."); there's no equivalent `WithPP` (or any other
  preposition) attachable directly to a genus noun — "with" only
  exists at the `PPv` (verb-phrase) level, never as an NP modifier.
  Unlike the earlier `OfPP`/`BarePl` fix, generalizing this would mean
  touching `NPx` itself, used by ordinary sentences project-wide, not
  a narrow addition — flagged, not built. Kept: *"a group of people,
  which has a goal"* (relative-clause form, already correct).
- **empty**: *"contains nothing"* doesn't parse, for two independent
  reasons: `nothing` isn't an Angloform word (no negative-indefinite-
  pronoun vocabulary exists, matching the "something"/"another" gaps
  already found), and even if it were, `AdjDef` has no mechanism for
  an embedded verb clause at all — it's adjectival content only, by
  design (Round 2's simplification). Kept: *"not full"*.

Verified: 12/12 `definition_grammar` tests, `cargo test --workspace`
clean (grammar crate rebuilt this round, not just the test file).

Running gap tally, all independently confirmed by direct grammar
checks rather than assumption: no infinitival "to V" (Round 4), no
NP-level "or" at genus/subject position (Round 3/6), copula (`be`) not
usable outside its own construction (Round 5), no nominalized gerunds,
no object-gapped relatives, no NP-attached "with", no negation on a
subjectless verb phrase, no indefinite pronouns ("something"/
"nothing"/"another"), no embedded wh-clauses. None of these block the
25-and-growing word set from having *a* working definition — every
single word trialed so far has one — they only block certain specific
phrasings some words would ideally use.

## 25 more in one batch, all first-try (2026-09-08)

Following `skills/lexicon-definitions/SKILL.md`'s process (draft with
already-real, already-general vocabulary → verify every word exists →
test): all 25 parsed on the first attempt, no grammar changes needed.

| lemma | kind | definition |
|---|---|---|
| event | NOUN | a thing, which comes at a time |
| feature | NOUN | a part of a system, which helps a person |
| memory | NOUN | a part of a system, which holds items |
| message | NOUN | a text, which comes from a person |
| model | NOUN | a thing, which describes a system |
| number | NOUN | a thing, which names a value |
| system | NOUN | a group of parts, which helps a person |
| sentence | NOUN | a group of words, which has a meaning |
| test | NOUN | an event, which checks a thing |
| send | VERB_TRANS | give a thing to a person |
| receive | VERB_TRANS | take a thing from a person |
| check | VERB_TRANS | find an error in a thing |
| open | VERB_TRANS | start a thing |
| close | VERB_TRANS | stop a thing |
| accept | VERB_TRANS | allow a thing |
| reject | VERB_TRANS | ban a thing |
| fix | VERB_TRANS | remove an error from a thing |
| update | VERB_TRANS | change a thing |
| stay | VERB_INTRANS | wait in a place |
| wait | VERB_INTRANS | stay for a thing |
| win | VERB_INTRANS | come before a group |
| conflict | VERB_INTRANS | come against a thing |
| sharp | ADJ | not blunt |
| clear | ADJ | not ambiguous |
| false | ADJ | not correct |

Verified: 13/13 `definition_grammar` tests, `cargo test --workspace`
clean.

One pair worth flagging rather than presenting as clean: **stay**
(*"wait in a place"*) and **wait** (*"stay for a thing"*) each use the
other as their own main verb. Not circular in the sense the headword
check forbids (different lemma each time, so it's structurally legal),
but it's the same *kind* of weakness as delete/remove — two words
leaning on each other instead of each having an independent
differentia — just with different PPs attached rather than identical
text, so it slipped past the "check for a collision" step. Left in
deliberately rather than quietly smoothed over; worth a second look
before treating either as settled.

60 words trialed total across 10 rounds.

## Round 4 on the 25-word batch: 9 real fixes, and the stay/wait cycle broken (2026-09-08)

**Fixed, using words already checked to exist**:
- **event**: *"comes at a time"* → *"occurs at a time"* — more precise verb, same already-real `occur`.
- **feature**: *"which helps a person"* → *"which changes a system"* — the original was an exact predicate collision with `system`'s own definition; this fixes the collision, not just the wording, and is arguably more accurate (a feature modifies the system it belongs to, rather than serving a person directly).
- **sentence**: *"a group of words"* → *"a sequence of words"* — `sequence` was already real and unused; more accurate than `group` (a sentence's word order matters; a group's doesn't).
- **test**: *"an event, which checks"* → *"a tool, which checks a thing"* — avoids defining one still-fresh word (`event`) in terms of another, uses the already-real, already-general `tool`.
- **send**: *"give a thing to a person"* → *"write a message to a person"* — narrower but more accurate, and both `write` and `message` (this session's own earlier definition) were already validated.
- **update**: *"change a thing"* → *"get a new version of a thing"* — `version` was already real and unused; more specific and more accurate than the generic "change."
- **stay / wait, the flagged pair, actually broken this round**: `stay` → *"exist in a place"* (an independent differentia, no longer routed through `wait`); `wait` → *"stay until a time"* (now safely one-directional — depends on the newly-independent `stay`, the ordinary way real dictionary definitions build on simpler words, not a mutual cycle). `until` (already `PREP_V`) and `exist`/`time` (already real) did the work; no new vocabulary or grammar needed.
- **win**: *"come before a group"* → *"come before every group"* — `every` (`NPEvery`, already part of `NPAny`) makes "ahead of absolutely everyone" explicit instead of just "ahead of one unspecified group."

**Kept as-is, proposals checked and rejected for real reasons**:
- **event** (`"a thing or a gathering, which comes at a time"`): fails twice — `or` at the genus position is Round 3's confirmed gap, and `gathering` only exists as a participle (`VERB_TRANS_ING`), not a noun (the same gerund gap `error` hit).
- **system** (`"many parts, in one thing"`): `many` **is not an Angloform word at all** — quantification only exists via `every`/digits/`some` (ADR 0014), no vague-quantity words.
- **receive** (`"take a thing, which a person gives"`): object-gapped relative clause again — same wall `tool`/`output` hit.
- **check** (`"search for errors in a thing"`): `search` isn't a word, and even if it were, `VBaseP` allows exactly one `PPv` — "for errors in a thing" is two chained PPs, structurally unreachable regardless.
- **open** (`"make a thing not closed"`): no resultative/small-clause complement exists (`make X ADJ` isn't a `VBaseP` shape) — and moot anyway, since `open`'s own *adjective* sense is separately disabled in the lexicon (redirects to `unlocked`).
- **accept** (`"do not reject a thing"`): `Prohibition` (`do not V`) is a wholly separate top-level production, unreachable from `VerbTransDef`'s `VBaseP`. Also worth naming even if it had parsed: defining `accept` as "not reject" and (symmetrically) `reject` as "not accept" would trade one circularity (stay/wait) for a worse one — two words with no independent meaning at all, only each other's negation.
- **false** (`"not true"`): **`true` is not an Angloform word.** Checked directly, not assumed — no entry in the lexicon at all. Possibly deliberate (the project may standardize on `correct`/`valid` throughout rather than `true`/`false` generally), possibly a real gap; noted here rather than guessed at. Kept: `not correct`.
- **conflict** (`"do not agree in every part"`): fails three ways — subjectless negation (the same gap `depend` hit), and neither `agree` nor bare `all` (`all` is explicitly banned by ADR 0014 in favor of `every` + singular) exist as words.

**message/number, no change needed**: the proposed alternatives (`someone`, `a count`) don't exist — `someone` is the same missing-indefinite-pronoun gap as `something`/`nothing`, and `count`'s noun sense is explicitly disabled in the lexicon (redirects to `value`, which the existing definition already uses).

Verified: 13/13 `definition_grammar` tests, `cargo test --workspace` clean.

## Round 5: two real over-narrowings, one reverted (2026-09-08)

- **feature**: *"which changes a system"* — called horrible, correctly.
  "Changes" was picked purely to dodge the earlier text collision with
  `system`'s own definition, not because it's what a feature actually
  does. Found the real word instead: `ability` (`NOUN_SG`, unused until
  now). **feature**: → *"a part of a system, which gives an ability"* —
  accurate on its own terms, and still collision-free with `system`.
- **send**: last round's *"write a message to a person"* was itself an
  over-narrowing — flagged directly: sending isn't only messages,
  parcels get sent too. Reverted to the original, genuinely general
  form: **send**: → *"give a thing to a person"*. Worth naming as a
  pattern: this round's fix for `feature` and this round's *un*-fix for
  `send` are the same lesson from opposite directions — a differentia
  should narrow toward what's *true*, not toward whatever avoids an
  unrelated problem (a collision, a vague genus). When a "fix" narrows
  the meaning itself, that's a sign to look for a different fix, not to
  keep the narrowing.
- **win**: *"come before every group"* — flagged for assuming team
  competition; winning applies to individuals too. `every group` also
  wasn't reachable in its intended stronger sense anyway (`every other
  X`-style exhaustive comparison isn't part of `NPAny`, only the
  narrow colon-list construction has it). Fixed to the more neutral
  **win**: → *"come before people"* — a bare plural (already
  established from `society`/`team`), general enough to cover both
  individual and team contexts without asserting either.

Verified: 13/13 `definition_grammar` tests, `cargo test --workspace`
clean.

## Round 6: send/win fixed, and 20 random words as a lighter-weight pass (2026-09-08)

**send/win, per direct feedback**:
- **send**: *"carry a thing to a person"* — "ask to deliver a thing to a
  person" was proposed and checked; fails on two already-known gaps at
  once (`ask` doesn't exist, and it's an infinitival complement besides
  — Round 4's gap again). `carry` (already real) captures the transport
  nuance without either.
- **win**: *"beat every person"* — stronger and more accurate than
  "come before people": `beat` (already `VERB_TRANS_BASE`) is the real
  verb for it, and this is also proof `VerbIntransDef`'s "identical
  grammar to `VerbTransDef`" design note (`angloform.lalrpop`) is a
  real, usable feature, not just a technicality — a transitive-shaped
  definition (`beat` + object) is exactly the right fit for an
  intransitive headword here.

**A policy change, not just a word list**: asked directly whether every
trialed word needs a permanent regression test — no. A `#[test]` earns
its place when it exercises a grammar edge worth guarding (a new
production, a boundary case); a word that only reuses already-proven
shapes and already-real vocabulary doesn't need one. The 25-word suite
stays (each round changed something structural); this round's 20-word
batch does not get a matching test function — logged here as a record
only.

**20 more, sampled at random rather than curated, lower polish
expected and found**: 18 of 20 landed on a working definition; two
stayed open as genuine gaps rather than being forced.

| lemma | kind | definition |
|---|---|---|
| claim | NOUN | a sentence, which comes from a person |
| noise | NOUN | a thing, which confuses a person |
| work | NOUN | a task, which gives a result |
| note | NOUN | a text, which gives a fact |
| table | NOUN | a thing, which shows items |
| algorithm | NOUN | a sequence of steps, which gives a result |
| author | NOUN | a person, who writes a text |
| expand | VERB_TRANS | add a part to a thing |
| weigh | VERB_TRANS | measure a thing |
| enforce | VERB_TRANS | cause a rule |
| assert | VERB_TRANS | give a claim |
| enable | VERB_TRANS | give an ability |
| coin | VERB_TRANS | make a new word |
| catch | VERB_TRANS | take a thing |
| tie | VERB_INTRANS | match a group |
| drift | VERB_INTRANS | come without a goal |
| alone | ADJ | single |
| compound | ADJ | not single |

**Two left open, real gaps, not forced**:
- **reappear**: `come again` fails — `again` isn't a word. The natural
  fallback, `come a second time`, fails too, for a sharper reason: a
  bare temporal NP ("a second time") isn't reachable as a `VBaseP`
  modifier at all — `PPv` always requires a preposition first, and
  there is no bare-adverbial-NP slot anywhere in the intransitive
  branch. A third, different-shaped gap from the `PPv`/`OfPP` family
  already catalogued.
  `grow` (already known-open from an earlier session) also stayed
  open this round — not re-attempted, no new angle found.
- **deliberate**: no antonym pair exists for intentional/accidental at
  all — `random`, `accidental`, `intentional`, `careful` are all
  absent; `planned` exists only as a participle (`VERB_TRANS_ED`), not
  an adjective. A real vocabulary gap, not a phrasing problem.

Verified: 13/13 `definition_grammar` tests (unchanged — this batch
added none), `cargo test --workspace` clean.

75 words engaged with total (60 previously fully landed + this
round's 20, 18 of which landed) across 11 rounds.

## Round 7: a real capability found unused, one added then reverted (2026-09-08)

**Fixed, using an existing capability nobody had combined yet**:
- **noise**: *"a thing, which confuses a person"* → *"a thing, which
  confuses a person **or does not have a meaning**"*. `PredRelCore`
  already lists `NegVPn` (negation) right alongside its plain-verb
  branches — negation inside a `NounDef` differentia was always legal,
  just never tried together with `Tailn`'s "or" until this word asked
  for both at once. Zero grammar change; added a small regression test
  (`tailn_predicate_may_itself_be_negated`) since this combination is
  worth guarding now that it's known to work.
- **tie**: *"match a group"* → *"match a result"* — more accurate
  (a tie is equal *outcomes*, not equal *groups*); both already real.
- **note**: *"the smallest part..."* pattern → *"a small document,
  which holds the information"*. Directly answers a question raised
  mid-round: Angloform is **not** missing a word for "information" —
  `information` (`NOUN_SG`) already exists, just unused until now.
- **table**: → *"a thing, which shows the information"* — same newly-
  surfaced `information` word, in the already-proven "holds the X"/
  "shows the X" shape (`the` required — `information` is a mass noun,
  same bare-noun gap `data` hit).
- **claim**: → *"a sentence, which names a fact"* — parses, and is the
  practical answer within this grammar's limits, but flagged as an
  honest imprecision rather than presented as clean: a claim need not
  be *true*, while "fact" nominally implies it is. The literal proposal
  (*"a sentence, which a person says is true"*) fails outright anyway
  — `true` still isn't a word (Round 4), on top of being an
  object-gapped relative (Round 3's gap).

**Tried, built, and reverted — a real grammar experiment, not a
dead end quietly dropped**: extended `AdjDef` with an optional `PPv`
tail (mirroring every verb/noun definition already having one),
specifically to fix `alone` ("not in a group" / "separate from a
group"). Built clean, zero conflicts — but neither phrasing that
motivated it actually worked once tried: *"not in a group"* has no
adjective at all for the negation to attach to (`AdjDef` still needs a
head adjective, tail or not), and *"separate"*'s adjective sense is
itself disabled in the lexicon (one-sense-per-word — it's a verb only).
With no word actually needing the tail, it was reverted rather than
kept as speculative, unused surface area — the original comment's own
bar (*"add it only once a real word needs it"*) still isn't met. Kept:
`alone` = *"single"* (still imprecise, still the best available).

**Confirmed real gaps, kept as-is**:
- **expand** (*"add a part to a thing, which already has parts"*):
  fails on a **new gap** — an object NP inside a verb definition (or
  any `VBaseP`) cannot itself carry a relative-clause modifier at all;
  the comma right after the object has no legal continuation. A
  distinct wall from the already-catalogued subject-gapped-only
  relative clauses (Round 3/4) — this one is about where a relative
  clause can attach at all, not which noun it attaches to. Kept:
  *"add a part to a thing"*.
- **enforce** (*"check consistently to prevent a conflict"*): two
  already-known gaps stacked — no adverb category (`consistently`
  isn't a word) and the infinitival-purpose-clause gap (Round 4)
  again. Kept: *"cause a rule"*.
- **assert** (*"enforce a claim"*): parses, but flagged as a semantic
  drift rather than adopted — "enforce" carries a compulsion/legal
  sense (enforcing compliance) that doesn't match "assert" (stating
  firmly). A case where "it parses" isn't sufficient on its own. Kept:
  *"give a claim"*.
- **catch** (*"notice a thing or hold a moving thing in a place"*):
  `notice` isn't a word, and separately, `moving` (only
  `VERB_TRANS_ING`) can't function as a prenominal adjective — no
  general participle-as-adjective mechanism exists (the same class of
  gap as `error`'s gerund wall, in adjective position instead of noun
  position). Kept: *"take a thing"*.
- **drift** (*"move in different directions"* / *"come in different
  directions"*): `directions` isn't a word — a genuinely missing
  vocabulary item, not a phrasing problem. Stays open, alongside `grow`
  and `reappear`, as this design's small but real cluster of
  motion/change words the current lexicon can't express well.

**Two scope questions, not definition questions**: `coin` and
`compound` were both flagged as possibly not belonging in a *minimal*
vocabulary at all. Correct observation, but out of scope for this
exercise — whether a word should be *in* the lexicon is a curation
decision for the maintainers, separate from writing the definition of
a word that's already there. Left both definitions as they stood.

Verified: 14/14 `definition_grammar` tests, `cargo test --workspace`
clean (grammar crate rebuilt twice this round — once for the AdjDef
addition, once for its revert).

## Round 8: the lexicon itself gets a new word ("dimension") (2026-09-08)

**note**, redundancy caught correctly: *"a small document, which holds
the information"* — "holds information" restates what "document"
already means; the differentia added nothing "small document" didn't
already say. `NounDef` still can't drop the differentia clause entirely
(the mandatory-differentia check this whole feature exists to enforce
— confirmed again: *"a small document, which a person keeps"* fails
too, object-gapped, same Round 3 wall). Fixed by going back to a
genuinely independent differentia already validated earlier this
session: **note**: → *"a small document, which gives a fact"* — real
content, not a restatement.

**table**, and a real first: this project's vocabulary got a new word.
`dimension` didn't exist anywhere in Angloform. Added it for real —
`seed/seed.json` (the hand-curated source of truth, ADR 0001), then
`cargo run -p lexgen` to regenerate `lexicon.tsv` deterministically —
not a hand-edit of the generated file. `lexgen` itself caught a real
issue on the first attempt: `dimension` also has a rare engineering-
verb sense ("to dimension a drawing") attested in the reference data,
and refused to proceed without a curator decision on it (`lexgen: 1
error(s): cross-POS...`). Added an explicit `reject.VERB.advice` entry
by hand, same as every other disabled-sense entry in the seed — a
single, reviewed addition, not a bulk sweep, consistent with the
project's own "a machine does not curate the Seed" principle.
**table**: → *"a thing, which organizes the information in 2
dimensions"*.

One real slip caught and fixed before it mattered: a first attempt at
editing `seed/seed.json` via a Python `json.dump` reformatted the
*entire* 8000+-line file (267 lines touched for what should've been an
8-line addition) — reverted immediately (`git checkout --
seed/seed.json`) before regenerating, and redone as a surgical `Edit`
matching the file's existing per-entry formatting exactly. A small
version of the same mistake this whole design has been guarding
against at the definition level, this time at the tooling level.

Verified: 15/15 `definition_grammar` tests (one new: `table_uses_the_
newly_added_dimension_word`, since a real vocabulary addition is worth
guarding), `cargo test --workspace` clean, `cargo run -p lexgen
--check` confirms `lexicon.tsv`/`docs/lexicon-report.md` are in sync
with the seed.

## Open, not decided here

- Exact suggestion ranking when multiple enabled words match a
  rejected sense's synset neighborhood — narrowed above from "which
  metric" to a real requirement (must include ≥1-hop hypernym
  proximity, not just same-synset), but the actual cutoff/ranking
  formula is still open.
- Whether `sense` should be optional for closed-class/function words
  (determiners, conjunctions) that don't meaningfully map to WordNet
  synsets at all — closed classes are already "fiat" elsewhere in this
  codebase (`lexgen`'s existing "closed classes are fiat; reference
  data can't judge them" comment), likely the same exemption applies
  here.
- Migration order: whether to do the ~980-word retrofit in
  frequency-ranked batches (fix the highest-traffic words' hints
  first) or systematically A→Z.
- ~~`VerbTransDef`'s mandatory purpose/result clause may be too
  strict...~~ **Resolved 2026-09-08**: tested against real words, found
  both gameable and too strict, replaced with a bare subjectless
  `VBaseP` (see the full revision history above) — verified, not just
  reasoned about.
- `VerbIntransDef` not built yet — do the same trial-then-refine pass
  `NounDef`/`VerbTransDef` got before trusting a shape for it.
  `AdjDef`'s shape (plain predicative or negated-contrast via
  `CopPredn<LCopSg>`) is built and passes both of its real test cases,
  but hasn't had the same adversarial "does this template actually
  prevent the problem it's for" stress test `VerbTransDef` just got —
  worth doing before trusting it as settled.
- Whether recursive quotation, once built for `LexEntry`, should stay
  scoped to definitions only or generalize to the fuller "mentioning
  a sentence" idea `docs/ideas.md` originally described.

This is a design, not an ADR yet — worth grilling before it's built,
the same way the relative-clause construction was, given it touches
every word in the language.

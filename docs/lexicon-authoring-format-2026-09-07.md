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

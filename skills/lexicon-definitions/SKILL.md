---
name: lexicon-definitions
description: Write a `definition` field for an Angloform lexicon word — a genus/differentia noun definition, a bare-verb-phrase verb definition, or a bare-adjective definition, checked against `parse_definition`. Use when writing, testing, or fixing a word's definition under the lexicon-authoring-format.
---

# Writing Angloform lexicon definitions

A definition is not a sentence. It is a fragment, checked by its own grammar
entry point — `parse_definition` in `crates/grammar/src/lib.rs`, on the
`def-grammar-entry-points` branch (worktree
`/home/kantord/repos/minglish-def-grammar` — not yet merged to `main`; that
is where the code below actually lives right now). Full round-by-round
history and rationale for every rule here: `docs/lexicon-authoring-format-2026-09-07.md`.

## Shape, by category

| category | shape | example |
|---|---|---|
| NOUN | `<genus NP>, which <predicate>` | `a group of people, which has a tradition` |
| VERB_TRANS | `<verb> <object> [<PP>]` — no subject, no "to" | `merge parts into a thing` |
| VERB_INTRANS | `<verb> [<PP>]` — no subject | `come to a place` |
| ADJ | `<adjective>` or `not <adjective>` | `not full` |

The genus NP may carry one optional adjective (`a human group`) and one
optional `of <bare plural or NP>` (`a group of people`). Every shape may
carry one optional `and`/`or`/`but` tail joining a second predicate/verb
phrase to the same genus or headword (`a thing, which builds a thing or
removes a thing`) — binary only, never a third.

Structural bans, already enforced, don't fight them: no trailing period; the
headword itself cannot appear in its own definition; a domain-model term
(`Name`, `Seed`, …) cannot appear in any definition.

## The process

1. **Draft** using the most general noun/verb that stays true — reach for
   `a thing` / `a person` / `a result` / `a group of people` before an
   invented or narrower noun. A definition that is only true of one
   project-specific use of the word is wrong, even if it parses.
2. **Confirm every content word is real** before using it:
   `grep -iP '^word\t' lexicon.tsv`. Never assume a word exists or guess its
   tag — the tag decides which branch of the grammar it can fill.
3. **Test it** — add a case to
   `crates/grammar/tests/definition_grammar.rs` and run
   `cargo test -p grammar --test definition_grammar`, or call
   `parse_definition` directly. A definition that "should" parse and hasn't
   been run is not a finished definition.
4. **On rejection, read the parser error to localize the failing
   production**, then diagnose which of three things happened, in this
   order: wrong word choice (fix and retest) → missing vocabulary (grep for
   a real synonym before giving up) → a genuine grammar gap (check the list
   below first; if it's not there, name the gap plainly in the design doc
   rather than forcing an awkward workaround).
5. **Check for an accidental collision** with another word's definition
   before accepting it — a duplicate is only acceptable when the language
   truly has no expressible differentia (`delete`/`remove`, documented as a
   known gap); otherwise a real differentia exists and is worth finding
   (`create` vs. `produce` → `make a **new** thing`).
6. **Run `cargo test --workspace`** before calling a grammar change to
   support a definition done — a fix for one word must never regress the
   rest of Angloform.

## Confirmed grammar gaps — checked, not guessed

Each of these was hit and verified directly against the grammar during this
format's design; don't spend time re-discovering them:

- No infinitival `to V` anywhere in Angloform (`to make a thing` doesn't
  parse; `merge parts into a thing` does).
- No NP-level "or" at a subject/genus position (`a person or thing` doesn't
  parse). Predicate-level "or"/"and"/"but" *is* available, binary only (see
  Shape above) — reach for a disjunction of two concrete verbs instead of
  one missing abstract one.
- The copula (`be`/`become`) is only ever inflected, bound to subject
  agreement — usable inside a NounDef's differentia (`a thing, which is
  wrong`) but never as a `VerbIntransDef`'s own head (`be in the world`
  doesn't parse).
- No bare singular/mass noun without a determiner (`holds data` doesn't
  parse — `data` needs "the"). Bare *plural* is fine (`holds items` does).
- `of` is the only preposition a genus noun can carry directly
  (`a group of people`). `with` and every other preposition exist only
  attached to a verb, never to a noun (`a group of people with a goal`
  doesn't parse).
- Relative clauses are subject-gapped only — the genus must be the subject
  of what follows "which"/"who", never its object (`a thing, which a person
  uses` doesn't parse; flip it — `a thing, which helps a person`).
- No embedded wh-clauses — "how" isn't an Angloform word at all.
- No indefinite pronouns (`something`, `nothing`, `another`) — use `a thing`,
  a specific bare plural, or an ordinal (`a second copy`) instead.
- No negation on a subjectless verb phrase (`does not exist without a
  thing` doesn't parse — negation always needs a person-agreeing `do`
  auxiliary, which conflicts with a definition's subjectless form). Find a
  real positive verb instead (`need a thing`, not `does not exist without a
  thing`).
- No nominalized gerund — an `-ing` form is a participle only, never usable
  as a noun (`the result of failing` doesn't parse).

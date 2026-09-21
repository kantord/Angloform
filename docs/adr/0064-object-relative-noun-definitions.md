# 0064 — Object-relative definitions: WHOM and a NounDef-only object role

Date: 2026-09-20
Status: proposed (tentative). Extends ADR 0059 and ADR 0061. Prototype
already built and validated; this ADR documents the decision.

## Context

The decision "0061" gave every NOUN a definition file. The definition
of a NOUN reuses the relative clause of the decision "0059": a genus,
then a comma, then the word "which" or the word "who", then a
predicate. The word "which" or the word "who" always takes the subject
role of the predicate. No grammar path exists for the object role.

Some real words name the object of an action, not the subject. The
word "referent" names the thing a word names — the referent takes the
object role of the verb "name", not the subject role. A definition
like "a thing, which a word names" failed to parse before this
decision: the parser expected a verb right after "which" and found a
determiner instead ("a word").

The decision "0003" bans the passive voice everywhere. A definition
like "a thing, named by a word" is passive, so the ban already forbids
it. The gap is real: some words have no active-voice subject-relative
definition and no other honest definition either.

The maintainers built a first version of an object-relative predicate
and ran it against the real Grammar build. The build found no
conflict. Every existing predicate after "which"/"who" starts with a
verb (a Form Tag like VERB_TRANS_3SG or MODAL_MUST). The new predicate
starts with a Noun Phrase (a determiner or a pronoun) instead. The 2
kinds of predicate never share a first token, so the parser tells them
apart with one token of lookahead — the same mechanism the decision
"0059" already uses to tell "which"/"who" apart from "because"/"so"
after a comma.

## Decision

The Grammar gets a new predicate shape for the NounDef grammar only
(never for ordinary prose, never for the Sentence a writer submits): a
real subject Noun Phrase, then a transitive verb, then nothing. The
antecedent (the word "which" or the word "whom") already fills the
object role, so the verb takes no following object — the same pattern
the existing intransitive predicate already uses for a verb with no
object at all.

The word "which" already exists (decision "0059"). The Grammar reuses
"which" for this new, object-relative role too, for a non-human
antecedent. No new word exists for a non-human antecedent because none
is needed: the 2 readings of "which" never conflict.

The Grammar adds one new word, "whom", for a human antecedent in the
object role. The word "who" already exists for the subject role
(decision "0059") and cannot double for both roles the way "which"
does, so a second word is necessary. Real English already keeps this
exact distinction (who/whom), so the new word is not an invention.

The change touches the NounDef grammar only. The decision "0061"
already keeps NounDef outside the Sentence a writer submits. The
decision "0003"'s ban on the passive voice stays exactly as it was
everywhere else.

## Consequences

- A NOUN whose real meaning names the object of an action, not the
  subject, can now get an honest, active-voice definition. The word
  "referent" is the first real example: "a thing, which a word names"
  (`seed/definitions/referent.yaml`).
- An earlier assessment named 4 ADJ words ("hidden", "generated",
  "nested", "capitalized") as stuck for the same reason as "referent":
  a participle of a passive action, with no active-voice definition.
  The assessment was wrong for 3 of the 4. The ADJ definition grammar
  already accepts a bare, optionally-negated adjective phrase (no
  clause, no subject, no verb at all), so a real antonym is enough:
  "generated" is "not handwritten"
  (`seed/definitions/generated.yaml`), "hidden" is "not explicit"
  (`seed/definitions/hidden.yaml`), and "nested" is "not flat"
  (`seed/definitions/nested.yaml`). None of the 3 needed this
  decision's new grammar at all — the real block was never the passive
  voice; it was never checking whether a plain antonym already existed
  in the vocabulary. The word "capitalized" still has no enabled
  antonym and stays a real gap.
- Not every previously-stuck NOUN gains a definition just from this
  change. The word "design" was tried with the new shape ("a
  structure, which a person plans") and reverted: the word "design"
  and the already-curated word "purpose" share a real WordNet synset.
  The block on "design" is a real collision with an existing word, not
  a Grammar gap.
- 3 places in the codebase matched every `Tok` variant by name, with
  no fallback case: `crates/diagnose/src/lib.rs` (2 places) and
  `crates/wasm/src/lib.rs` (1 place, 2 match expressions). The new
  `Tok::Whom` variant broke all 3 at compile time — a real, caught
  failure, not a hypothetical one. Each now names `Tok::Whom`
  explicitly, following the same pattern already used for `Tok::Who`.

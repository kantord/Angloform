# 0027 — Domain model: defined terms as a capitalized pack

Date: 2026-09-02
Status: proposed (tentative). Realizes the "core lexicon + jargon packs"
idea; supersedes the hand-written CONTEXT.md glossary.

## Context

The maintainers used the repair on the decision "0002". The 5 runs
produced 6 valid proposals. The maintainers judged the 6 proposals. The
wording was unnatural, so the 6 proposals needed a repair. The
language did not have a signal for a concept of the project, so the
proposals decomposed the concept into general words. The pattern has 2
examples:
- "the ambiguity of the reference"
- "the layer of the discourse"

The decision "0015" requires a definition for an opaque coinage. The
tools did not enforce the definitions or use the definitions. A term can
be a verb or can be an adjective. The project has 3 terms of the kind:
- "waive"
- "attest"
- "anaphoric"

The 3 terms are not nouns, so a list of the nouns does not cover every
term.

## Decision

The file "domain/model.json" is a pack with the shape of the Seed. Every
entry of the model has the 6 fields of the Seed:
- "lemma"
- "category"
- "forms"
- "reject"
- "waive"
- "note"

Every entry must have a definition in Angloform. Lexgen merges the 2
files into one Lexicon. The pack adds words but does not override a word
of the Seed. A Lemma of the model cannot repeat a Lemma of the Seed in
one Category. An entry of a Name has a definition but does not have
Surface Forms. The model has 4 Names:
- Angloform
- Lexgen
- Triage
- WordNet

A noun of the model is capitalized. A term can be one word or can be a
phrase. The Lexer gives one token to a term. The token is singular or is
plural. The rule has 2 examples:
- "the Linter bans Anaphoric Pronouns"
- "every Surface Form has one Form Tag"

If the model does not contain a capitalized word, then the word is a
Name. The loud rules of the decision "0018" cover every Name. The
capital does not mark 2 Categories of the model:
- the verb
- the adjective

If a term does not have a capital, then the Linter shows the advice.

The tools use every definition. The command "just check" lints every
definition. The prompt of the repair shows every term with a definition.
The command "just define" shows one definition. Lexgen writes the file
"CONTEXT.md" from the model. If the file "CONTEXT.md" drifts from the
model, then the command "just check" fails.

The maintainers moved the terms of the glossary into the model. The Seed
contained the jargon of the project, so the maintainers moved the jargon
into the model. The pack got 24 nouns and got 4 Names. The pack got 5
verbs and got 4 adjectives. The maintainers changed 5 texts:
- the corpus
- the decision "0001"
- the examples of the skill
- the showcase
- the advice of the Linter

The maintainers put a capital on every term of the 5 texts. A definition
replaces the attestation for a term, because the data does not attest a
coinage. Lexgen checks a term for a Collision. If a term has 2
Categories, then Lexgen rejects the term.

## Consequences

- The Language Model sees one unit with a meaning, so the repair does
  not decompose a concept of the project. The capital marks a term for a
  reader, so the reader finds every definition in the file "CONTEXT.md".
- The capital carries a meaning, so the orthography of the decision
  "0018" has 2 exceptions. The first exception is a Name. The second
  exception is a term. A term can open a sentence, because the term is a
  Surface Form of the Lexicon.
- A definition is a claim of the project, so the maintainers review
  every definition. The decision "0012" covers a Translation Pair and
  covers a definition.
- The maintainers deferred 3 questions. The first question is a signal
  for the verbs of the model. The question covers the adjectives of the
  model. The second question is a pack for a different domain. The third
  question is a word with 2 roles. The word "Triage" is a Name of the
  tool but is a noun in the lowercase.

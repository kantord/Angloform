# 0055 — A prepositional phrase after an adjective complement

Date: 2026-09-06
Status: proposed (tentative). Closes a deferral of ADR 0023. Extends ADR
0031.

## Context

The decision "0023" banned the word "same". The Redirect of the Ban names
the word "identical". The decision "0023" deferred the phrase "identical
to the report", because the Complement did not take a Prepositional Phrase
after an adjective.

The decision "0031" added a Prepositional Phrase after the Complement of a
Noun Phrase. The Copula does not have an object, so the Prepositional
Phrase has one attachment. The Complement of an adjective has the
identical property.

## Decision

The Grammar gains one alternative of the rule "Compl". The Complement of an
adjective can take one Prepositional Phrase. The sentence "the copies are
identical to the report" is one example.

The object of a verb keeps the attachment of the decision "0011". The Ban
of the decision "0031" keeps the subject.

The Grammar does not limit the adjective. The Grammar does not limit the
preposition. The decision "0031" did not limit the Complement of a Noun
Phrase, so the decision "0055" repeats the choice. The sentence "the
design is consistent with the project" is a second example. The sentence
"the report must be safe from the error" is a third example, because a
modal takes the identical Complement.

## Consequences

- The phrase "identical to the report" is not a Ban of the decision
  "0023". The maintainers can revert the rewrite of a sentence with the
  phrase.
- The Linter drops 2 old rules. The old rules named the phrase "an
  adjective cannot take a prepositional phrase yet". The old rules named
  the decision "0023".
- A legal shape does not make a good sentence, so the warning of the
  decision "0031" stays.

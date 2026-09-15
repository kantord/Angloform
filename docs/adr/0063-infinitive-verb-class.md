# 0063 — A closed class of verbs before the infinitive

Date: 2026-09-15
Status: proposed (tentative). Needs a real test before a final decision.

## Context

The tool "triage" named a problem. The corpus has 375 sentences. Every
one of the 375 sentences has the word "to" after a verb.

The object of the verb "desire" is a Noun Phrase. The sentence "the
agent desires the file" is valid. The sentence "the agent desires to
read the file" does not parse.

The maintainers checked a repair. The repair writes 2 sentences. The
first sentence is "the agent desires the file". The second sentence is
"the agent reads the file". The repair changes the meaning. The 2
sentences describe a fact: the agent reads the file. The original
sentence does not describe the fact. The agent may still fail.

The decision "0059" builds 2 sentences from a relative clause. A
relative clause describes a fact of a Noun Phrase, so the new sentence
keeps the meaning. The word "to" after a verb, the word "desire" for
example, does not have the property.

## Decision

The maintainers add a Category: "VERB_INF".

A verb of the Category "VERB_INF" takes an object: the word "to"
before a bare verb.

The Category "VERB_INF" stays a small list. The list stays closed. The
Category "MODAL_CAN" is a second example of a small, closed list. The
Grammar does not give every verb the new pattern. A verb, which is not
a member of the Category "VERB_INF", cannot take the word "to" before
a verb. A person must still write 2 sentences for every other verb.

A bare verb after the word "to" cannot belong to the Category
"VERB_INF". The Grammar bans a chain of the word "to".

## Consequences

- A person gains a pattern for a desire or a plan before a fact.
- The Category "VERB_INF" stays closed, so the Grammar avoids a
  general rule for every verb.
- The maintainers must review every future member of the Category
  "VERB_INF".
- The decision requires a real test before a status change. The
  decision "0059" sets a precedent for the test.

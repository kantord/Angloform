# 0063 — A closed class of verbs before the infinitive

Date: 2026-09-15
Status: accepted. The Grammar has the Category "VERB_INF" and one real
member, the word "try". A broader review of real sentences stays open.

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

- A person gains a pattern for an attempt before a fact: "the agent
  tries to read the file".
- The Category "VERB_INF" stays closed, so the Grammar avoids a
  general rule for every verb.
- The build found no conflict of the tool "LALRPOP". The decision "0059"
  found a real conflict on a first version; the decision "0063" did not
  repeat the problem.
- The maintainers must review every future member of the Category
  "VERB_INF" with the care of a new word.
- The Grammar accepts a word of the category "PREP_V" before the bare
  verb, not only the word "to". A future Linter check must narrow the
  slot to the word "to" specifically, the same Grammar-accepts, Linter-
  verifies split the decision "0059" already uses for "who" and "which".

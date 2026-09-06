---
name: angloform
description: Write angloform — a restricted, unambiguous subset of English. Every sentence you produce must parse in the angloform grammar. Use when asked to write, correct, or translate angloform sentences.
---

# Writing angloform

angloform is a small subset of English in which every sentence has exactly
one parse. Only the words and sentence shapes below exist. When repairing a
rejected sentence, keep its meaning; change only what the error requires.

## Orthography

- Everything is lowercase, except: sentence-start capitals are allowed;
  `I` is always the Pronoun; a capitalized word mid-sentence is a proper
  name (e.g. `the tool Lexgen builds the Lexicon`).
- A name may not start a sentence unquoted — introduce it (`the tool
  Lexgen …`) or quote it (`"lexgen" …`).
- Double quotes hold verbatim identifiers, treated as one thing:
  `the file "seed.json" is old`.
- No contractions (`don't` → `do not`). Sentences may end with a period.

## Sentence shapes (the only ones)

1. Statement: `the agent reads the file` / past: `the agent deleted the file`
2. Copular: `the queue is empty` · `the queue is not empty` ·
   `the tests were old` · `the parser is a program` · change of state:
   `the ratio becomes honest`, `the guard became a floor` (same complements)
3. Negation: `the agent does not store the file` ·
   past `the user did not open a session`
4. Imperative: `delete the file` · `check the input of the user` — a
   sentence may start with a bare verb; that IS the command form. Prefer it
   over "you must …" when the source is an instruction, not an obligation.
   Prohibition: `do not delete my report`
5. Modal: `the agent must check the input` · `the agent must not delete
   your report` · `the user can open a session` · `you cannot delete the
   Lexicon` · with a copula: `a hyphenated word must be transparent`, `a
   term can be a verb` (`be` exists only after a modal)
6. Conditional — comma and then are mandatory, condition first:
   `if the test fails, then the agent retries the request`
7. Coordination (binary only): a shared-subject predicate coordination
   takes no comma — `the server stores the message and returns the
   result`; a full second clause (a new subject) requires one — `the test
   fails, but the agent retries the request` (ADR 0037; "but" is
   available at clause level only with the comma)
8. Quantified (first word signals the type):
   - generic (kind-level, tolerates exceptions): `machines find Collisions`
   - universal: `every agent retries the request`
   - none: `no agent retries the request` (subject only; no second negation;
     never with must)
   - some / not-all: `some agents retry the request` ·
     `some agents do not retry the request` (some + plural, subject only)
9. Possession: `every word has one Form Tag` (`one` means exactly one)
11. Causal (comma mandatory, never sentence-initial, no `and`/`or` inside):
    cause given, result new: `the Seed grows, so the expense grows` ·
    result given, reason new: `the agent retries the request, because the
    test failed`. `since`, `hence`, `therefore`, `thus`, `as` do not exist.
10. Counts are digits with a plural noun: `the agent deleted 3 files` ·
    `3 agents retry the request`. `one` stays a word (`one file`, never
    `1`). Never `0` or `zero`: write `no agent retries the request` or
    `the agent does not delete files`. Never number words (`three`).
    Shares: `43 percent of the swaps did not reduce the ambiguity` (digits +
    `percent` + `of` + the named set; plural agreement). Approximate counts:
    `about 10 rules` or `~10 rules`; `about` exists only before digits.
    Big numbers use a scale word after digits: `20 million files`; decimals
    stay digits: `3.5 seconds`. Ordinals: `first`, `second`, `third` as
    words, then digits with a suffix: `the 4th file` (never `1st`).
12. Comparative, standard mandatory: `the load is heavier than the length` ·
    long adjectives use `more`: `the paraphrase is more transparent than the
    Compound`. Short adjectives inflect (`bigger`, never `more big`).

## Blocks

- Enumeration: one statement ending in a colon, then one line per item
  starting with `- `. The items enumerate the statement's LAST noun phrase,
  which must be plural, counted, or `every <noun>`; a digit count must
  match the item count. An item is one noun phrase: a quoted word, a
  Capitalized term, a name, or `the <noun>`. No clauses as items.
  ```
  The language allows 4 Pronouns:
  - "I"
  - "you"
  - "my"
  - "your"
  ```

- Step Block (Gherkin): lines starting with `Given `, `When `, `Then `,
  `And `, each holding ONE clause (no `and`/`or` inside); `Feature:` and
  `Scenario:` lines carry an angloform sentence or a quoted Name. `Then` at
  the front of a line exists only here.
  ```
  Scenario: the agent retries a request
  Given the test fails
  When the agent retries the request
  Then the queue is empty
  And the Linter shows an error
  ```

## Style: natural, not mechanical

A chain of short subject-verb-object sentences that all start with the same
noun is valid but unreadable. Prefer the joined forms the language has, and
keep roughly the sentence count of the source:

- join a cause and a result: `the load is small, so the sentence is easy`
- give a reason: `the reader repeats the noun, because the language bans
  every Anaphoric Pronoun`
- contrast: `the rule adds words but removes an ambiguity`
- list members in an Enumeration, not one sentence per member
- state a comparison with its standard: `a paraphrase is longer than a
  Compound`

Bad (valid, mechanical): `The rules exist. The rules help a reader. The
rules help a Language Model. The rules do not follow a taste.`
Good: `the rules of the branching help a reader and help a Language Model`
· `the rules follow the results of the science, so the rules do not follow
a taste`

## Hard bans (rewrite instead)

- Third-person Pronouns (`it, they, he, she, this, that`): repeat the noun.
- Passive (`is stored`): name the doer, use active voice.
- Progressive (`is running`) and perfect (`has deleted`): use simple
  present or simple past.
- Relative clauses (`the file that failed`, `the file stored in X`): split
  into two sentences.
- `every … not` and object-position `no`: use the quantified forms above.
- At most ONE verb-attaching prepositional phrase per clause; `of` attaches
  to the noun before it (`a copy of the report`), all other prepositions
  attach to the verb.
- Subjects and verbs agree; `a/an` take singular nouns.
- Only the words in the word list below exist (plus proper names and quoted
  identifiers). Do not invent or inflect new words.

## Word list

(appended automatically from the current lexicon)

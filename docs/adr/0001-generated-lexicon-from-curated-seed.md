# 0001 — Lexicon is a generated artifact from a curated Seed list

Date: 2026-08-31
Status: accepted (rewritten in angloform 2026-09-01; meaning preserved, see docs/dogfood-adr-0001.md)

## Context

The language needs a Lexicon. The Lexicon contains every Surface Form of the
language and contains a Redirect for every Rejected Sense. Every Surface Form
has one Form Tag. The maintainers considered 3 options.

1. The first option derives the Lexicon from the corpora, so a machine
   chooses the words.
2. The second option is a handwritten Lexicon. People write every Surface
   Form into the file `lexicon.tsv`. A tool checks the handwritten file.
3. The third option is a small Seed. People curate the Seed. A machine
   builds the Lexicon from the Seed.

The file `docs/research/cnl-design-findings.md` describes the prior
research. The algorithms optimized the rarity of the words but did not
optimize the clarity of the words. One generator replaced common words with
rare words. The rarity of the new words explained the whole improvement, so
the improvement was not real. 43 percent of the swaps did not reduce the
ambiguity. The choice of the words must stay with people.

Every verb has about 5 Surface Forms. A handwritten Lexicon has typos and
has hidden holes. No Surface Form can have 2 Form Tags. The Paradigm of the
verb "leave" collides with the Paradigm of the noun "leaf", so the Surface
Form "leaves" is a Collision. People cannot find every Collision, so a
machine must find the Collisions.

## Decision

The maintainers chose the third option. The Seed is a handwritten file.
The Seed is the single source of the Lexicon. Every Lemma has one entry in the Seed.
The entry names the Category of the Lemma. An entry can contain 4 fields:
- the irregular Surface Forms
- the Redirects
- a Waiver
- a note

The note is a free text and records the rationale.

The tool Lexgen is a program in Rust. The folder `crates/lexgen` holds the
program. The program expands the Paradigms with about 10 rules. The program
checks 3 invariants. No Surface Form has 2 Form Tags. If the data attests a second
Category of a Lemma, then the entry must reject the second Category. A
Rejected Sense has a Redirect or has a Waiver. The data attests every
Surface Form. If the Seed breaks an invariant, then the
program returns an error. The output of the program is deterministic. The
program writes the file `lexicon.tsv` and writes the file
`docs/lexicon-report.md`. The maintainers save the generated files in the
repository. No person edits the generated files.

The maintainers store the data in the repository. The data has 3 parts:
- the database WordNet
- the file `mobypos`
- the table of the frequencies

The file `data/checksums.sha256` pins the data. The Linter checks the words
with the data. The data attests the words but does not choose the words.

## Consequences

- People choose the words and write one entry for every word. The machine
  writes the inflections and finds the Collisions.
- A commit of the Seed changes the generated files, so the diffs show the
  effect of the commit. The tool Git keeps the history of the curation.
- The generated files must match the Seed, so the maintainers need a future
  test. If a person edits the file `lexicon.tsv`, then the future test
  fails.
- A late reversal moves the curation into a database or moves the curation
  into small files. A reversal migrates every entry of the Seed, so the
  expense of a reversal grows with the Seed. The maintainers record the
  decision, because a reversal is expensive.

# Vocabulary ratchet: findings (2026-09-11)

Working notes from turning the thesaurus-derived vocabulary pipeline
(`docs/vocabulary-rebuild-2026-09-09.md`) into a controlled, repeatable
process. Captures what changed and what the numbers actually showed,
before scaling the process further.

## The ratchet

`scripts/vocab-candidates.py N` walks the first `N` unique,
WordNet-resolvable lemmas by frequency and mechanically buckets each one
(exactly 1 sense in its best category = admit-eligible; the rest deferred).
Left alone, that produces a backlog: hundreds of accepted-by-default
candidates nobody has actually turned into a word or explicitly rejected —
the ledger's own "no `rejected` field = accepted" default silently
tolerates this.

**The fix: `N` only ever grows by a small step, and the fresh batch it
introduces gets fully cleared — every entry becomes a real
`seed/definitions/*.yaml` file or a documented rejection — before `N`
grows again.** Never jump to a large `N` and leave most of it unprocessed
(this session did exactly that once, reaching `N=500` with only ~18 of 142
admit-eligible candidates actually resolved — the corrective reset is what
prompted writing this down). State lives in `docs/.vocab-ratchet-n`
(the codified process is `.claude/skills/vocab-ratchet/SKILL.md`).

## `Gap`: a word isn't its hardest sense

Early in this pass, a missing cross-POS redirect was treated as grounds to
drop the whole word (`human`, `social`, `while` were deleted outright).
That was wrong, for a reason the project's own design already named:
`seed.json` is *deliberately* kept around as bootstrap scaffolding —
this system is mid-bootstrap by design, so "no substitute exists **yet**"
is expected, not disqualifying.

Fixed by adding `gap: true` as a real third option in
`seed/definitions.schema.json` (alongside `redirects`), wired through
`lexgen` (`RejectTarget::Gap`, a `## Gaps` report section) and
`scripts/word-check.py`. A word keeps its clean, useful sense; the
uncovered one is marked as an honest, structural absence — revisited
later via `docs/word-sense-provenance.yaml` (which logs the WordNet gloss
that would justify a future redirect, even when none exists yet), not
silently reopened once a real redirect does exist.

**The decision rule, now stable**: never use `advice` (hand-written
micro-definitions produced real semantic errors this session — e.g.
"military" (adj) described as "not common", picked only because "common"
happened to already exist as a word). Every cross-POS sense is exactly one
of: a real `redirects` word, or an honest `gap`. Dropping the whole word is
reserved for when the word's *own* definition can't be written honestly —
missing building blocks, or a winning sense that's false/misleading — not
merely because a redirect is hard to find.

## Strong vs. weak: what the numbers actually show

- **strong**: the winning category is the *only* one attested at all
  (every other POS count is 0) — zero cross-POS collision, ever.
- **weak**: the winning category still wins cleanly, but at least one
  other category has senses too — a redirect/gap decision is needed.

Strong bucket is cheaper to process (no collision to resolve). It is
**not** automatically higher-yield — being "strong" only rules out one
failure mode (collision complexity), not the other (not actually being a
content word). Checked directly rather than assumed:

```
N = 2000 → strong = 250 candidates
  142 rejected mechanically:
    - ADV winning category: Angloform has no open adverb category at all
      (structurally impossible — a large fraction of "strong" hits are
      -ly adverbs precisely because adverbs are reliably monosemous)
    - proper-noun folding (WordNet has zero capitalized entries — names,
      places, abbreviations hide inside lowercase lemmas)
    - months/days of the week (same as the earlier "june" finding)
    - pronouns/determiners/function words (same class as the
      already-rejected "both"/"either"/"another")
   21 rejected on inspection:
    - near-synonyms of an already-enabled word (awesome/wonderful/
      excellent/incredible ≈ "good"; tiny ≈ "small"; vast ≈ "huge";
      obvious ≈ "clear"/"apparent"; dad ≈ "father") — redundant, no
      distinct meaning to add
    - comparative/superlative/possessive inflections, not base lemmas
      (greater, greatest, larger, smaller, men's)
    - interjections (hello, oh)
    - previously-established blockers recurring at the larger N (wife,
      mom, daughter, river, financial, datum — same missing-vocabulary
      walls found earlier in this session)
   81 real survivors — genuine nouns/verbs/adjectives worth writing
      real definitions for (aircraft, airport, apartment, factory,
      hotel, kitchen, lawyer, scientist, software, illegal, achieve,
      download, deserve, ...)
```

**Real yield: ~32% of the strong bucket (81/250).** The other 68% is an
artifact of how WordNet+frequency mechanically select "monosemous," not a
signal about word quality — worth knowing before scaling `N` further:
raw strong-bucket size overstates how many real words are actually in
reach at a given `N`.

## Reusable rejection taxonomy

The patterns above are stable enough to apply as a fast first pass on any
future batch, strong or weak, before the slower per-word review:

1. Winning category is ADV → automatic, unconditional reject (no open
   adverb category exists).
2. Proper-noun folding (country/city/person names, abbreviations/acronyms
   hiding as lowercase lemmas).
3. Months and days of the week (capitalized-Name-like in real English,
   regardless of what WordNet's lowercase entry looks like).
4. Pronouns, determiners, quantifiers (same/similar shape to the
   already-rejected `both`/`either`/`another`/`whatever`).
5. Near-synonym of an already-enabled word — check the candidate/redirect
   list `scripts/word-check.py` prints; redundant synonyms add no real
   coverage.
6. Comparative/superlative/possessive surface forms are not base lemmas.
7. Interjections are not content words.
8. A definition needing a building block that doesn't exist yet (checked
   directly against `lexicon.tsv` and the word's *actual* meaning, not
   just its existence — `state` meaning "condition" not "nation" was a
   real bug caught this way) is a real, structural rejection, not a
   wording problem to paper over.

## Result: the full strong-bucket pass, N=2000

Of the 81 real survivors, 21 were actually admitted with working
definitions; 60 hit one of two remaining walls, both structural rather
than fixable by rewording:

- **`AdjDef` is bare-synonym/antonym-only.** Most ADJ candidates
  (`famous`, `successful`, `relevant`, `random`, `helpful`, `horrible`,
  `additional`, `administrative`, `solar`...) have no existing ADJ to
  redirect through. Only `illegal` ("not legal") worked, because `legal`
  already existed. This is the single biggest wall — worth deciding
  whether to build a small set of foundational antonym-pair adjectives
  deliberately, rather than hoping the mechanical process surfaces them.
- **`PredRel` is subject-gapped only.** A NOUN's differentia clause must
  have the genus as the *subject* of what follows ("a thing, which uses
  X" — not "a thing, which X uses"). Blocked `software` and a few others
  the same way the original trial's `tool` was blocked.
- Everything else was a genuine missing-vocabulary gap (food, sport,
  music, geography, aviation, family-relation, law-practice, health)
  rather than a grammar wall — each is a nameable, specific future target
  once the corresponding domain gets its own foundational words.

**21 admitted this pass**: `apartment`, `artist`, `asset`, `citizen`,
`cloth`, `consumer`, `customer`, `download`, `employee`, `factory`,
`household`, `illegal`, `income`, `math`, `museum`, `opportunity`,
`photo`, `poverty`, `scientist`, `stadium`, `statistic` (plus `up` from
the earlier N=50 batch, and the 7 pre-existing canonical words).

**Scope note**: this pass deliberately covered only the *strong* bucket at
N=2000 (see "Strong vs. weak" above) — the weak bucket at this N (338
candidates) is untouched and will show up as backlog the next time
`vocab-ratchet` runs. That's intentional, not an oversight: weak-bucket
words additionally need a redirect/gap decision per collision, a
different, slower kind of batch than this one.

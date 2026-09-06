# Resources

Free, well-licensed resources for building angloform — a formalized, minimal,
unambiguous subset of English. Grouped by what we'd use each for.

License flags: ✅ fine for an MIT project · ⚠️ restriction, read the note

## Lexical resources (lexicon curation, ambiguity checks)

### WordNet (Princeton) ✅
- **What**: Hand-built lexical database. Per word: its meanings (synsets), and
  per meaning: synonym set, gloss, example sentences. Split by POS
  (`index.noun` / `index.verb` / `index.adj` / `index.adv` — one line per
  (word, POS) with sense count as the third field).
- **Use here**: The core data source for the lexicon linter — cross-POS
  membership check (ban/redirect trigger), per-POS sense counts, and per-sense
  synonym sets for picking unambiguous replacement spellings.
- **License**: WordNet License (BSD/MIT-style permissive). Vendor the data with
  its own `LICENSE` file + one attribution line in the README.
- **Get**: <https://wordnetcode.princeton.edu/3.0/WNdb-3.0.tar.gz> · via
  `nltk.corpus.wordnet`
- **Caveats**: Oversplits related senses (treat sense counts as upper bounds;
  *make* has 49). No function words. Adjectives are the weakest section.
  Frozen since ~2006.

### Open English WordNet ✅
- **What**: Actively maintained fork of Princeton WordNet; yearly corrections
  and new vocabulary. Same synset structure.
- **Use here**: Drop-in upgrade over Princeton WordNet if we hit missing/stale
  entries.
- **License**: CC-BY 4.0.
- **Get**: <https://github.com/globalwordnet/english-wordnet> · pip package `wn`

### Moby Part-of-Speech (`mobypos`) ✅
- **What**: Word → POS membership codes (e.g. `build\NV` = noun+verb).
  Membership only, no sense counts.
- **Use here**: Fast second opinion on cross-POS membership. Noisier than
  WordNet's indices.
- **License**: Public domain.

### Moby Thesaurus ✅
- **What**: One flat synonym list per entry — no POS, no sense grouping,
  loose editorial standards (all senses mixed in one pile).
- **Use here**: Bulk polysemy proxy only (synonym count correlated with
  frequency better than WordNet sense counts in our experiments). Never for
  sense-level work — WordNet does that.
- **License**: Public domain.

### wordfreq (zipf frequencies) ✅
- **What**: Word frequency data (zipf scale) across many corpora.
- **Use here**: Frequency guard in the linter — block swaps that replace a
  common word with a drastically rarer one (measured: mean 11× rarer, worst
  case *need*→*necessitate* at 1,349×).
- **License**: Apache-2.0 (library code); **bundled data CC BY-SA 4.0** — not
  MIT-compatible, so derived tables are kept out of the repo and fetched or
  regenerated at build time (`scripts/fetch-data.sh`).
- **Get**: pip package `wordfreq`

## Corpora and treebanks (ambiguity measurement, test sets)

### LinGO Redwoods Treebank ✅
- **What**: ~85k sentences with **all** grammar-licensed parses recorded plus
  the annotator-preferred one (grammar-based/dynamic treebank over the ERG).
  The only free resource with full parse forests, not just one gold tree.
- **Use here**: (1) Parse-count-per-sentence = empirical ambiguity ranking of
  English constructions → what to ban. (2) Its coverage gaps = adversarial
  "can angloform paraphrase this?" test set.
- **License**: Open source.
- **Get**: <https://github.com/delph-in/docs/wiki/RedwoodsTop> · tooling:
  pydelphin + ACE parser
- **Caveats**: POS layer is HPSG lexical types, not Penn/UD tags.

### WeScience (in Redwoods) ✅
- **What**: 100 Wikipedia NLP articles, hand-treebanked with full forests.
- **Use here**: Closest thing to "AI/tech domain with parse forests."

### The Cathedral and the Bazaar test suite (DELPH-IN) ✅
- **What**: E. Raymond's open-source essay as a multilingual shared test suite.
- **Use here**: Tech-prose paraphrase test material.

### UD_English-EWT ✅
- **What**: Universal Dependencies treebank; web/newsgroup/email text; one
  gold dependency tree per sentence, UPOS + XPOS tags.
- **Use here**: Clean default corpus for POS + tree statistics.
- **License**: CC BY-SA 4.0.
- **Get**: <https://github.com/UniversalDependencies/UD_English-EWT>

### UD_English-GUM ⚠️
- **What**: UD treebank spanning many genres incl. CS academic writing and
  wikiHow how-tos.
- **License**: **CC BY-NC-SA 4.0 overall** — commercial / non-open use of some
  texts prohibited. Avoid unless the NC restriction is acceptable.

### NLTK large grammars: ATIS + CommandTalk ✅
- **What**: Two large CFGs with test sentences and known parse counts. ATIS:
  5,517 rules, 98 sentences, max 36,122 trees for one sentence (flight
  requests). CommandTalk: 28,851 productions, 162 sentences (spoken command
  language).
- **Use here**: Ambiguity sandbox; both are imperative/command style —
  structurally the closest existing data to agent instructions.
- **Get**: `nltk.data` (`grammars/large_grammars/`)

### Simple English Wikipedia ✅
- **What**: Millions of words of register-restricted content (incl. AI/tech
  topics) with aligned full-English counterparts.
- **Use here**: Cheap existence proof for paraphrase adequacy — evidence that
  restricted-register rewrites of real content are possible at scale.
- **License**: CC BY-SA.
- **Get**: <https://dumps.wikimedia.org/simplewiki/>

### Wiktionary dumps ✅
- **What**: Sense-grouped synonyms; covers function words and new vocabulary
  WordNet misses. Wikitext is painful — use pre-extracted JSON
  (wiktextract / kaikki.org).
- **License**: CC BY-SA / dual GFDL.

## Prior art (read, or reuse components)

### Attempto Controlled English (ACE) ✅
- **What**: The most thoroughly worked-out controlled English (Zurich, since
  1995): unambiguous, translates to first-order logic, looks natural. APE
  parser is open source (Prolog) with a ~100k-entry lexicon in a separate repo.
- **Use here**: Steal design decisions: fixed interpretation rules (e.g. PP
  attaches to verb), the paraphrase-back loop, anaphora resolution by fiat
  (most-recent-matching-noun).
- **Get**: <https://github.com/Attempto/APE>

### Grammatical Framework + ACE-in-GF ✅
- **What**: GF: abstract syntax + per-language concrete linearizations.
  ACE-in-GF: an ACE subset ported to ~20 natural languages, built to not
  overgenerate (drives a look-ahead editor).
- **Use here**: Relevant if "universal" ever means multilingual; also the
  look-ahead-editor pattern for writer tooling.
- **Get**: <https://github.com/Attempto/ACE-in-GF>

### Non-open controlled-language specifications ⚠️ DO NOT USE
- **Policy**: Several industrial controlled-language specifications exist
  under proprietary, non-open licenses. Do not copy, consult, cite, or
  derive from any of them. All angloform design decisions must be grounded
  in open resources and our own measurements (triage reports, research
  findings, dogfood cases).

### English Resource Grammar (ERG) ✅
- **What**: Broad-coverage precision HPSG grammar of English; MIT license;
  parses to full forests + MRS semantics; bidirectional (can generate from
  semantics).
- **Use here**: Parse our own agent-domain text and dump complete parse
  forests; reference point for what a serious formal grammar of English covers.
- **Get**: <https://github.com/delph-in/erg> · ACE parser + pydelphin

### Minimal English / NSM (Wierzbicka & Goddard)
- **What**: ~65 semantic primes claimed cross-linguistically universal, plus a
  small extension vocabulary. A research tradition, not a dataset.
- **Use here**: The tradition that seriously stress-tests paraphrase adequacy;
  a floor for "how small can the vocabulary go."

### Other specs worth skimming
- Common Logic Controlled English (CLCE)
- SBVR Structured English
- Ogden's Basic English (1930) — as a **counter-example only**: built from
  the most polysemous verbs in the language (its 18 operators average 19.7
  WordNet senses vs 2.6 for verbs generally — our own measurement of open
  data). The opposite of our strategy. Analysis/citation use only; we do not
  reproduce its text or word list, and nothing in angloform derives from it.

## Tooling (not data, but identified alongside)

| Tool | Role | License |
|---|---|---|
| LALRPOP + logos (Rust) | Strict tier-1 grammar; LR(1) conflicts as ambiguity alarm | MIT/Apache |
| Rustemo (Rust, GLR) | Dev-time forest counting: "exactly one parse" tests | Apache |
| chumsky + ariadne (Rust) | Writer-facing error recovery/diagnostics, if needed | MIT |
| Earley (tier 2) | Dirty-grammar diagnostics: named STYLE/AMBIGUOUS errors | — |
| spaCy | POS-in-context tagging, dependency metrics | MIT |
| Stanza | Constituency trees when needed | Apache 2.0 |
| pydelphin + ACE | Driving the ERG | MIT |

Avoid: pest / PEG tools and nom/winnow for the grammar — PEG's ordered choice
silently resolves ambiguity, which is the opposite of what a CNL needs.

## Known gap

Nothing above is a corpus of **agent instructions specifically** — ATIS and
CommandTalk are the nearest relatives. This is why the hand-written seed
corpus (`corpus/accept.txt` / `corpus/reject.txt`) is a deliverable, not
something to download.

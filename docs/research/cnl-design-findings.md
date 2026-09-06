# Research findings: designing angloform (a minimal, unambiguous English subset)

**Provenance note:** These findings come from exploratory Claude web sessions
(August 2026) that combined literature review with ad-hoc experiments (WordNet /
Moby / frequency analysis over ~7,000 verbs; parser prototypes in Lark/Python).
The experiment scripts were **not preserved**. Each claim is tagged:

- `[literature]` — backed by published work; citation given.
- `[experiment]` — measured in-session; numbers reported as observed, but not
  yet reproduced in this repo. Treat as strong hypothesis until re-run.
- `[retracted]` — asserted in-session, later checked, and found unsupported.

## 1. Vocabulary and lexical ambiguity

### Frequency and polysemy correlate positively `[experiment]`
Common words have *more* meanings, not fewer. Verb frequency (zipf) vs. mean
WordNet sense count, n≈7,000, monotonic across every band: 1.90 senses in the
2–3 zipf band up to 12.53 in the 6–8 band (6.6×). Correlations r = +0.39
(WordNet senses) to +0.62 (Moby synonym counts).
**Consequence:** "unambiguous vocabulary" and "familiar vocabulary" are
structurally in tension; every disambiguating word choice tends to cost
frequency/readability.

### Moby beats WordNet as a polysemy signal `[experiment]`
Moby Thesaurus synonym counts correlated with frequency better than WordNet
sense counts (+0.618 vs +0.389 for verbs), despite Moby being the cruder,
flat-list resource.

### Ogden's Basic English uses the *most* polysemous verbs `[experiment]`
Ogden's 18 operator verbs average 19.7 WordNet senses vs 2.6 for verbs
generally (7.5×). *make* = 49 senses, *give* = 44, *take* = 42, *get* = 36.
Basic English's strategy (few, highly general words) is the exact opposite of
angloform's (one form, one meaning).

### Lexical swap algorithms find rarity, not disambiguation `[experiment]`
A swap generator (replace ambiguous word with rarer synonym, e.g.
*need*→*necessitate*) reduced mean sense count 5.4 → 2.5 — but randomly chosen
rare words of the same zipf band scored 2.0, i.e. **better**. On
within-POS homonym clusters the algorithm exactly tied random rare words
(1.7 vs 1.7). The apparent disambiguation was a side effect of rarity.
- The one genuine effect: **cross-POS ambiguity** dropped 1.62 → 1.10 while
  random rare words stayed at 1.64. That is the parser-relevant axis.
- Cost: 84% of swaps made the word rarer, mean drop ≈11× frequency; worst
  cases *need*→*necessitate* 1,349×, *ask*→*enquire* 269×.
- 43% of swaps fired on words that were already POS-unique — pure cost, zero
  benefit.

**Consequence (design decision):** use a **redirect table**, not spelling
swaps, as the primary mechanism: `present: VERB accepted; NOUN → "gift";
ADJ → "current"`. The common word stays, the parser stays deterministic, and
the writer gets a precise error. Spelling swaps are justified only for
homonymy *within* a single POS (e.g. *bank* = tilt an aircraft / deposit
money), which a POS-keyed redirect cannot separate.

### POS-uniqueness must be checked over full inflection paradigms `[literature/experiment]`
Two unambiguous lemmas can collide after inflection: *leaves* (leaf+PL /
leave+3SG), *saw*, *found*, *left*, *means*, *lives*, *read*. The lexicon
linter must generate each candidate's full paradigm and re-check the whole
lexicon on every addition.

### English noun/verb conversion is near-total in technical vocabulary `[literature]`
*file, log, link, input, output, access, process, cache, buffer, queue, index,
run, build, test, commit, push, merge, branch, release, …* — a strict ban list
for cross-POS words deletes most needed vocabulary; renaming/redirecting, not
banning, is the viable lever.

## 2. Grammar and structural ambiguity

### POS-uniqueness eliminates only lexical ambiguity `[literature]`
"Put the block on the table in the box": every word has exactly one POS, and
the sentence still has two parses. PP attachment, coordination scope, compound
nouns, and relative-clause attachment survive any lexical discipline and must
be handled by grammar restrictions plus fixed interpretation rules (cf.
Attempto ACE's "PP attaches to the verb" rule + paraphrase-back loop;
https://github.com/Attempto/APE).

### A CNL grammar can stay context-free `[literature]`
The classic beyond-CF argument (Shieber 1985, Swiss German cross-serial
dependencies) concerns constructions a designed subset simply excludes.
Agreement = finite features; selectional restrictions = type checking on the
AST; anaphora = discourse pass; quantifier scope = fixed rule. All four move
out of the grammar.

### The two-tier architecture is forced, not optional `[experiment]`
Merging "clean" and "banned-but-diagnosable" productions into one grammar
re-introduced the identical LALR(1) reduce/reduce conflict the ban existed to
remove — annotating banned productions cannot work, because the parser must
choose a path before it can annotate. Hence: tier 1 = strict LALR(1) (linear,
provably unambiguous); tier 2 = superset grammar parsed with Earley, run only
on tier-1 rejection, whose job is diagnosis:
- 1 parse in tier 2 → **STYLE** error (e.g. center-embedding): name the
  construction, suggest the fix.
- N>1 parses → **AMBIGUOUS** error: enumerate the readings; the ambiguity
  itself is the error message.
Maintenance obligation: tier 2 must remain a strict superset of tier 1 —
property-test by generating from tier 1 and asserting tier 2 accepts.

### Banned structures contain their own rewrite `[experiment]`
A center-embedded sentence ("the committee that reviewed a report approved a
system") was mechanically rewritten from its own parse tree into two clean
sentences. The escape hatch ("break it into multiple sentences") is a
derivation from the tier-2 tree, not a fallback.
Implementation trap: Lark filters inline string literals out of parse trees —
a rewriter silently loses words unless every terminal is declared explicitly.

### Ambiguity migrates instead of disappearing `[literature]`
Splitting sentences requires cross-sentence reference, and pronouns
re-introduce ambiguity ("The engineer reviewed the report. It failed.").
Attachment ambiguity is traded for reference ambiguity. Options: legislate
resolution (ACE: most-recent-matching-noun) or ban pronouns and repeat the
noun. The discourse layer, not the grammar, is the
underestimated cost.

### "Ban left-branching" is the wrong rule; bound the load instead `[experiment/literature]`
A Brown-corpus sentence with 92% of its material before the main verb was
effortless (peak open dependencies = 2). The rule that survived measurement:
ban left-branching **that carries load** — a weight bound (fronted material up
to N words / one clause), expressible in a CFG — not a structural prohibition.
Style guides (Pinker, *The Sense of Style*) give defaults with exceptions; a
categorical grammar ban is stricter than the advice it encodes.

### Peak open dependencies is the best "hard to unpack" predictor `[experiment/literature]`
Grounded in Gibson's Dependency Locality Theory. Comfortable ceiling ≈4
simultaneously open dependencies; center-embedding depth ≥2 and dependency
spans ≥10 words are the other reliable flags. Computable from a dependency
parse alone (no constituency parse needed).

## 3. Corpora and methodology

### Grammar-based treebanks record full parse forests `[literature]`
Redwoods (over the ERG) records *all* grammar-licensed analyses plus the
annotator-preferred one, stored as discriminant decisions (dynamic treebank).
Two uses for angloform: parse-count-per-sentence as an empirical ambiguity
ranking of constructions (what to ban), and coverage gaps as an adversarial
paraphrase test set. Caveat: validating "angloform can express anything"
against a grammar-filtered corpus pre-selects away the counterexamples — the
*failures* are the deliverable.

### No agent-instruction corpus exists `[literature]`
Nearest relatives: NLTK's ATIS (5,517 rules; one sentence with 36,122 parses)
and CommandTalk grammars — command/imperative style. The hand-written seed
corpus (`corpus/accept.txt` / `corpus/reject.txt`) is therefore a project
deliverable, not a download.

## 4. Retracted claims — do not build on these

- **~2.5× expansion ratio** for rewriting into the subset `[retracted]` —
  came from 22 hand-rewrites by one author; presented as data, actually
  judgment.
- **"Defined terms make documents opaque to newcomers"** `[retracted]` — no
  measurement existed; the reader-cost metric built on it is an assumption.
- **Frequency and ambiguity "inversely correlated"** `[retracted wording]` —
  the correlation is *positive* (common = more ambiguous); only
  frequency-vs-monosemy is inverse.
- **PEG parsers for CNLs** — rejected on principle `[literature]`: ordered
  choice silently resolves ambiguity, hiding exactly the readings a CNL must
  surface.

## Re-verification queue

The `[experiment]` numbers are cheap to reproduce (WordNet + Moby + wordfreq
are all in `docs/resources.md`). Priority order if we re-run:
1. Swap-vs-random-rare-words control (grounds the redirect-table decision).
2. Frequency–polysemy band table.
3. Ogden operator sense counts (one afternoon; purely mechanical).

## 5. Findings from building the tier-1 grammar (2026-09-01)

### Lexicalized disambiguation makes the grammar trivially deterministic `[experiment]`
The LALRPOP grammar over form-tags compiled LR(1)-clean on the first attempt —
no conflicts, no precedence declarations. Every classic conflict source had
been moved into the token inventory beforehand (PREP_N/PREP_V for attachment,
DET_SG for number, dedicated COPULA/MODAL/NEG_AUX classes). Rule: when
structure depends on which word it is, make that word its own token class.

### Grammar-writing discovers language gaps corpus testing misses `[experiment]`
- First person has no copula: *am* is absent, so "i am X" is inexpressible
  (Pred1 has no copular branch). Decision pending.
- Ambitransitive verbs are trapped by the one-tag-per-surface invariant:
  *stop* (VERB_TRANS) cannot also be VERB_INTRANS without its whole paradigm
  colliding, so "the process stops" cannot be said. A merged category with an
  optional object is LR-clean and unambiguous — but it is an ADR-worthy
  change to the invariant's interpretation.

### Metric artifact: clause markers inflate peak-open `[experiment]`
Conditionals measured peak-open 5 (ceiling ≈4), but if/comma/then each arc to
the main-clause verb under the current head rules, stacking three
function-word arcs on one genuine held dependency. Re-derive arcs (markers →
local clause head; punctuation excluded) before calibrating any bounds.

### Agreement duplication is the predicted rule blow-up `[experiment]`
PredSG/PredPL/Pred1 are ~90% identical. Collapse with LALRPOP parameterized
nonterminals before adding the next construction.

### Below-grammar invariants exist
*a/an* selection is phonological — enforceable only in the lexer/validator,
never in the CFG.

### Structural unambiguity does not imply semantic unambiguity `[experiment]`
Dogfooding ADR 0001 produced translations that parsed uniquely and scored
cheaply yet were *more* ambiguous than their originals: mandatory
determiners removed English's genericity device (bare plurals, process
nominals), forcing generic statements into false definites ("the words") or
generic/existential-ambiguous indefinites ("a person"). No existing metric
detects this — it was caught by human review under the ADR 0012 process.
"Generics" was already on the research list of CNL semantic failure modes;
this is its first concrete observation in our own system. Genericity needs a
sanctioned form (candidates: a GENERIC determiner by fiat, or sanctioned
bare plurals) before technical policy prose is expressible.

### Transparency promoted from unverified claim to design value (2026-09-01)
The retracted "defined terms make documents opaque to newcomers" claim
(§4) remains empirically unmeasured, but ADR 0015 now adopts its direction
as an explicit design value: vocabulary is chosen transparency-first
(existing word with matching everyday meaning > transparent paraphrase >
transparent hyphenated entry > unfold the concept into a sentence; opaque
coinages never enter the lexicon). A value does not require the
measurement; the distinction from a verified finding is recorded here.
Immediate corollary discovered while testing: unfolding "form-tag" into
"every word has one tag" requires *has* (possession) and *one* (numeral) —
both still undecided, now with a concrete blocked sentence attached.

### Translator compression bias, and the nominalization blind spot (2026-09-01)
Third ADR 0012 catch: the translator (the model) optimized the cost ratio,
producing short renderings that dropped each sentence's point (a human-vs-
machine contrast; a reliability claim). Faithful retranslations flipped the
dogfood cost ratio from 0.56 to 1.13 — revealing a metric blind spot:
unigram surprisal prices nominalizations ("word choice", "a human
judgment") as cheap because their words are common, while their reader
decoding cost is invisible. angloform unfolds nominalizations and gets
billed for it. Consequences: (a) the cost ratio understates angloform
precisely where it helps most; (b) the ratio is telemetry, never a target
for translators; (c) a future cost model should price syntactic/semantic
decoding, not just lexical frequency (LM-based surprisal would partly
capture this).

### Parseability bias in rewrites (2026-09-01, 4th metric-chasing catch)
Auditing the in-place ADR 0001 rewrites as English showed two of three had
traded precision for angloform-validity ("committed"→"has"; "owns
safety"→"finds the collisions") with the loss under-declared. The flow's
rule is: rewrite into well-written English; when well-written English still
fails to parse, that is GAP EVIDENCE (here: *we*, commit-as-verb), never a
license to flatten the prose. The rewriter — human or model — must not
treat parse-validity as the quality target; ADR 0012 review applies to
rewrites exactly as to translation pairs.

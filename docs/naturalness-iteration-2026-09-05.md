# Naturalness iteration, round 2 (2026-09-05): a grammar construction, a wrong conclusion, then a real fix

Continuation of `docs/prompt-ab.md` (same day, same session). **Read
the "Correction" section before trusting anything under "What was
tested" below it** — this file's own first conclusion was wrong, and
staying wrong for most of the session is itself part of the record,
not edited away.

## What was built

`docs/judge-report.md`'s failure notes were read directly (not
re-guessed) to rank failure shapes by real frequency. The dominant
shape, by a wide margin, is a **run of 3+ consecutive sentences that
share a subject-verb template but not the actual subject** — e.g.
`docs/adr/0014-universal-and-no.md`'s "The word 'if' opens a
Conditional. The word 'do' opens a Prohibition. The word 'no' opens a
universal Negation..." (7 sentences in a row, same template).

The grammar had no way to combine 3+ *different-subject* clauses into
one sentence — confirmed empirically (`just lint` rejects both the
Oxford-comma and pure `and`-chain forms). ADR 0037 gave a binary
(exactly-2) shape; extending it seemed like the obvious fix.

**ADR 0050** (`docs/adr/0050-nary-clause-coordination.md`, itself
100% dogfooded angloform) adds a 3-clause Oxford-comma shape to
`crates/grammar/src/angloform.lalrpop`'s `CoordClause` rule. Built as
explicit bounded alternatives (2-ary, 3-ary), not a generic `+`
repetition macro — the macro version produced a genuine LALR(1)
shift/reduce conflict with the existing Causal/Conditional lookahead
discipline (see `docs/controversial-decisions-2026-09-05.md`).
Verified: clean build, all existing tests pass, `./scripts/check.sh`
clean, `dogfood-sweep`/`parse-report` numbers unchanged from before
the grammar edit (confirmed by `git stash`-comparing the exact same
sweep with and without the change). ADR 0050 is a real, kept,
zero-regression capability gain independent of everything below.

## What was tested (superseded conclusion — see Correction)

`docs/adr/0014-universal-and-no.md`'s 7-sentence run was rewritten
using the new construction, then judged 7 separate times against 5
different interventions (grammar coordination, word choice, a raw
markdown table, and re-judging with document context included). Every
single trial scored **2/5**, and combined with `docs/prompt-ab.md`'s 2
earlier trials that made 7/7 blind judgements at 2/5 across 2
documents. The conclusion drawn at the time — stated flatly, and
wrongly — was that "dense, defined-term-heavy enumerative content has
a real naturalness ceiling that sentence-level interventions do not
move, and it is not a testing artifact."

**That conclusion was wrong, and the "not a testing artifact" claim
was the specific error.** All 7 of those trials shared one thing: the
judge prompt used in every one of them was a shortened paraphrase of
the project's own real judge instruction in `docs/prejudge.md`, and it
silently dropped the one line that instruction leads with: *"Read
CONTEXT.md for the Capitalized terms."* Every trial this session ran
judged the paragraph with no access to the project's own term glossary
— judging a paragraph dense with Capitalized project-specific terms
(Sentence Shape, Bare Plural, Indexical Pronoun, First Token) against
a judge that has never been told what those terms mean. The "reads
like undefined jargon" and "catalog-like" complaints those judges kept
returning were, at least in part, a direct artifact of that missing
context, not a property of the prose.

## Correction: re-run with the real protocol

The same 3 paragraph versions (original, 3-way-coordinated, and a
plain Enumeration of the 7 tokens) were re-judged using the *actual*
`docs/prejudge.md` naturalness-judge prompt verbatim, with the judge
given `CONTEXT.md` to read first, exactly as the real pipeline does:

| version | naturalness (no CONTEXT.md, 7/7 trials) | naturalness (real protocol, with CONTEXT.md) |
|---|---|---|
| original, 7 flat sentences | 2/5 | 2/5 |
| 3-way coordinated (ADR 0050) | 2/5 | **3/5** |
| plain Enumeration (7 tokens, mixed granularity) | 2/5 | **3/5** |

Real movement, on 2 of 3 versions, from the protocol fix alone. The
plain Enumeration's *remaining* complaint under the correct protocol
was specific and new: the list mixes quoted literal words ("if", "do")
with abstract grammatical categories (a Bare Plural, a determiner) in
one flat list — inconsistent item granularity, not "reads like a
list" in general.

That complaint pointed at a concrete, fixable content restructuring:
split into two internally-homogeneous Enumerations — one for the 4
literal-word triggers, one for the 3 category triggers — rather than
one mixed list. Rewritten as:

```
The First Token of a sentence announces the Sentence Shape. 4 First
Tokens are words:
- "if"
- "do"
- "no"
- "every"

3 First Tokens are Categories:
- a Bare Plural
- a determiner
- an Indexical Pronoun
```

Judged with the real protocol: **naturalness 4/5.** Checked for
fidelity too (the real protocol's other required gate, telephone
explainer + rater against the original 7-sentence paragraph, run
independently, neither agent seeing both texts): **fidelity 4/5** —
meeting `docs/judge-report.md`'s actual pass bar (naturalness ≥4.0,
fidelity ≥4) on a paragraph that failed all day. This version is now
applied to `docs/adr/0014-universal-and-no.md` and dogfoods clean
(`./scripts/check.sh` and the full test suite pass, corpus parse
numbers unchanged from before this edit).

**The fidelity check did find real, non-fabricated loss**, not a false
alarm: the split version drops the explicit "a Bare Plural opens a
generic statement" / "an Indexical Pronoun opens a plain statement"
outcome mappings that the original stated and the new version doesn't
— it names the 7 First Tokens but not what each individually triggers
beyond the word/Category split. The rater scored this a 4, not a 5,
specifically because of that loss; it did not call it disqualifying.
This is flagged, not quietly accepted — see
`docs/controversial-decisions-2026-09-05.md`.

## What this actually shows

1. **The judge protocol matters as much as the content.** Half of this
   session's headline finding ("naturalness has a hard ceiling")
   was an artifact of a shortened, incorrect judge prompt, not a
   property of the language. Any future naturalness experiment must
   use the real `docs/prejudge.md` prompt verbatim, with `CONTEXT.md`
   attached, not a hand-written approximation of it.
2. **Once judged correctly, both tested interventions helped**: ADR
   0050's grammar construction moved this exact paragraph 2→3, and
   restructuring mixed-granularity list content into homogeneous
   groups moved it 3→4, past the pass bar. Neither alone would have
   been enough; both together did it on this one case.
3. **This is one paragraph, not a corpus-wide result.** 1 of the 34
   failing documents (`docs/judge-report.md`) had 1 paragraph tested
   and moved from below-bar to passing. Whether the same 2 moves
   (homogeneous grouping + ADR 0050 where applicable) generalize
   across the other ~33 failing documents' other failure paragraphs is
   untested — a real next step, not a corpus-wide claim this session
   supports.
4. **The associative/table-mapping gap identified earlier is still
   real** (angloform has no construction for a genuine key→value
   mapping — confirmed empirically, Enumeration items must be single
   noun phrases) but turned out not to be necessary for *this*
   paragraph: splitting into 2 homogeneous flat lists, accepting a
   small, disclosed fidelity cost, was enough. It may still matter for
   other paragraphs whose mapping content can't be split this way.

## Recommendation

1. **Re-run `docs/judge-report.md`'s full 37-document pass with the
   real protocol** was never in question — that's what it already
   does. What's new here: any *ad hoc* judging done during future
   design iteration (as this session did, twice, incorrectly) must use
   the real prompt from `docs/prejudge.md` verbatim, CONTEXT.md
   included, or its results are not trustworthy — this session is the
   proof.
2. Apply the "split mixed-granularity lists into homogeneous
   Enumerations" pattern to the other flagged "run of same-template
   sentences" paragraphs in `docs/judge-report.md` and re-judge with
   the corrected protocol before concluding whether it generalizes.
3. The table/mapping construction gap (see "What this actually shows,
   item 4) is still worth a real design pass, but is no longer the
   only lever — treat it as one option among several, not the
   necessarily-required fix this file originally concluded it was.

## What stays, what doesn't

- **ADR 0050 stays** — a real, verified, zero-regression capability
  gain, and (correcting the earlier record) it did measurably help
  naturalness once judged correctly, just not on its own.
- `docs/adr/0014-universal-and-no.md`'s "first-token telegraph
  principle" paragraph is rewritten to the homogeneous-split version,
  verified 4/5 naturalness and 4/5 fidelity against the real protocol,
  and dogfoods clean. The 2 lost per-item outcome mappings are a known,
  disclosed trade-off, not an oversight.

## Round 3 (`/goal make at least 10 documents pass`): a second, bigger correction

The goal after this file's first correction was to raise the corpus
pass count from 3/37 to 10+. This section is itself another
in-place correction, following the same discipline as the first: the
initial approach looked promising, then real re-testing surfaced two
problems bigger than the original plan accounted for.

### What was done

5 near-miss documents were targeted (0002, 0014, 0025, 0028, 0031 —
each blocked by only 1-2 of the pass bar's 4 dimensions, not a broad
naturalness failure), matching the same read-the-real-notes,
fix-the-specific-thing discipline as the rest of this file:

- Real content fixes to specific flagged sentences (mechanical
  repetition, unclear referents, undefined "archetype" codes,
  inconsistent list granularity), verified against each document's
  original English source where one exists — this caught and fixed a
  real, serious error: an earlier "clarifying" edit to
  `docs/adr/0014-universal-and-no.md` (the object-position quantifier
  scope rule, and the Ban→replacement mapping) directly **contradicted**
  the original source, which states both were already decided
  ("surface order = scope order"; "each excluded combination has an
  unambiguous home") — the rewrite had invented "the maintainers did
  not decide this" where the source says the opposite. Caught by
  running the real fidelity rater against the actual original, not
  by inspection; corrected in place.
- Every edit verified against `just lint-file` (100% parse) and the
  full regression suite (`cargo test`, `./scripts/check.sh`) — no
  language-level regressions anywhere in this round.

### Two things this round found that are bigger than "fix 5 documents"

1. **`docs/judgements.yaml` is substantially stale.** Re-running
   `docjudge.py report` after recording this round's judgements shows
   **18 of 37 documents flagged `*stale*`** — their recorded scores no
   longer match their current file hash, including `0030` and `0035`,
   both counted in this file's own "3/37 passing" baseline earlier
   today. That baseline was itself partly stale by the time it was
   read. This is the same class of problem as `docs/dogfood-sweep.md`'s
   staleness (found earlier this session, `docs/controversial-
   decisions-2026-09-05.md` item 8) — recorded reports drifting from
   the corpus without anything flagging it. **"10 documents pass"
   cannot be reliably answered without first refreshing judgements
   across the stale half of the corpus**, not just fixing 5 targeted
   documents — a materially bigger task than this round's scope.
2. **Single-run LLM naturalness judging has real, large variance —
   large enough to flip a document's pass/fail on its own.** The same
   5 documents were judged twice this round (before and after a second
   pass of fixes). Naturalness means moved: 0002 3.89→4.44 (real
   improvement, consistent with the actual edits), but 0014 4.08→3.83,
   0025 3.71→3.29, and 0028 4.22→3.67 **all dropped**, including on
   paragraphs that were not touched between the two runs. `0031` fell
   3.67→2.67. This is not evidence the edits made things worse — it is
   evidence that **a single judge run is not a reliable pass/fail
   signal**, full stop. `docs/prejudge.md`'s own protocol does not
   currently call for averaging multiple runs; this round's data says
   it should, at least near the pass/fail boundary (mean within ~0.3
   of 4.0).

### Result

Recorded via the real pipeline (`docjudge.py record` + `report`, not
hand-tallied): **1 of 37 documents pass** (`0013` — untouched by this
session, unaffected by staleness). None of the 5 targeted documents
are confirmed passing by the official system as of this recording,
though `0002` (naturalness 4.44, min 3, fidelity 4, unclear 1) is one
fixable "unclear" span away from a genuine pass and is the strongest
candidate to close first.

### Recommendation

"10 documents pass" is not reachable with confidence in a single
further session on the current approach. Before more document-level
naturalness rewriting:

1. **Refresh the 18 stale judgements first** — they may already be
   fine (the file content changed for reasons unrelated to naturalness,
   e.g. earlier grammar/vocabulary ADRs), which would move the honest
   baseline count with zero rewriting work.
2. **Judge near-pass-bar documents at least twice and average**, or
   pick the median, before recording a result as final — single-run
   scores this close to the 4.0 line are not trustworthy alone, per
   the variance found above.
3. Continue the targeted-fix method from this round (it works — 0002's
   real improvement is genuine, confirmed by content-level reasoning
   about the specific edits, not just the score) but budget for the
   multi-run judging overhead this finding implies, not single-pass
   scoring.
4. Do not repeat this session's mistake in `0014`: always check a
   document's original English source (`docs/dogfood-adr-*.md` or the
   bundle's `originals/`) before "clarifying" an ambiguous angloform
   sentence — the ambiguity may be a lossy compression of something the
   source already resolved, not open information to invent an answer
   for.

### Stale-judgement naturalness refresh (informal — not recorded)

Per the user's direction, all 18 stale-flagged documents were re-judged
for naturalness only (no content edits, no telephone/fidelity pass —
that's the remaining work). **This data is deliberately NOT recorded
into `docs/judgements.yaml`**: recording naturalness alone would carry
each document's *old* telephone/fidelity judgement forward under a
*new* hash, making it look current when it isn't — a worse state than
staying honestly marked stale. Treat the table below as a scouting
pass, not an official result.

| doc | mean | min | n | naturalness-only candidate (≥4.0, min>2)? |
|---|---|---|---|---|
| 0003 | 3.30 | 2 | 10 | no |
| 0004 | 3.38 | 2 | 8 | no |
| 0005 | 4.22 | 3 | 9 | **yes** |
| 0006 | 3.29 | 2 | 14 | no |
| 0007 | 3.56 | 3 | 9 | no |
| 0009 | 3.67 | 2 | 9 | no |
| 0010 | 3.22 | 2 | 9 | no |
| 0016 | 3.40 | 3 | 5 | no |
| 0019 | 3.33 | 2 | 6 | no |
| 0023 | 3.18 | 1 | 11 | no — ¶1 scored 1/5, looks like a genuine text corruption (a stray unmatched `)`, a subjectless fragment), not a style issue; check the raw file |
| 0026 | 2.80 | 2 | 10 | no |
| 0030 | 4.50 | 4 | 6 | **yes** |
| 0032 | 2.80 | 2 | 5 | no |
| 0033 | 4.20 | 3 | 5 | **yes** |
| 0034 | 3.33 | 2 | 9 | no |
| 0035 | 4.00 | 3 | 4 | **yes** |
| 0036 | 3.75 | 3 | 8 | no |
| model | 2.61 | 2 | 89 | no |

**4 of 18 clear the naturalness bar alone** (0005, 0030, 0033, 0035) —
still need unclear=0 and fidelity≥4 (where applicable) confirmed before
counting as an official pass, but they are the cheapest remaining
candidates: no rewriting needed if the other two dimensions hold.
**14 of 18 do not** — most in the 3.0–3.7 range, the same "mechanical
repetition" shape found everywhere else in this session's work, not
new failure modes. One likely-genuine defect found in passing:
`0023` ¶1 (score 1/5) looks like real text corruption, not a style
complaint — worth a direct look before its next naturalness pass.

One recurring judge complaint across nearly all of `model`'s 89
paragraphs — "sentence starts lowercase, reads like a typo" — is very
likely a **false complaint**, not a real defect: angloform's own
orthography rule makes a lowercase sentence-initial word valid
(`skills/angloform/SKILL.md`: "sentence-start capitals are allowed",
not required), and the entire corpus uses this convention
deliberately. Do not "fix" this in a future pass; it is the judge
being wrong about the language's own rules, not the prose being wrong
— a good candidate for a `docs/prejudge.md` prompt clarification
(tell the judge lowercase sentence starts are valid angloform) so this
false signal stops recurring.

### Round 4: chasing the 4 naturalness-candidates to a full pass — decisively worse, not better

The 4 documents that cleared naturalness alone (0005, 0030, 0033, 0035)
plus 0002 (closest overall) were each given targeted fixes for their
specific unclear spans, using the same proven techniques as the rest of
this session (archetype-code glosses, disambiguating a vague causal
link, checking the original source before asserting anything for 0002/
0005). Every fix individually verified: `just lint-file` 100%, full
regression suite clean, and — checked directly this time, learning from
the `0014` mistake — consistent with each document's own internal logic
and, where one exists, its original English source.

Re-judged with the identical combined naturalness+telephone protocol.
**All 5 got worse, not better, on both axes simultaneously**:

| doc | naturalness before | after | unclear before | after |
|---|---|---|---|---|
| 0002 | 4.44 | 3.11 | 2 | 3 |
| 0005 | 4.22 | 3.33 | 2 | 4 |
| 0030 | 4.50 | 3.33 | 3 | 4 |
| 0033 | 4.20 | ¶4 scored 1/5 | 4 | 3 |
| 0035 | 4.00 | 2.25 | 2 | 4 |

5 for 5. This is no longer attributable to run-to-run judge noise alone
(the earlier, smaller-magnitude finding) — the pattern is systematic
and points at something structural: **making an ambiguous claim
explicit enough to resolve it (the fix "unclear" demands) tends to add
exactly the kind of extra clause, repeated template, or spelled-out
enumeration that naturalness scoring penalizes.** `0002`'s clearest
case: replacing an ambiguous 4-item Form-Tag list with 4 fully explicit
"the word X has the Form Tag Y" sentences resolved the ambiguity
completely — and scored **1/5** for reading "like a mechanically
generated table dump," worse than the original list it replaced.
Naturalness wants compression and variety; unclear-resolution wants
explicitness and completeness. On short technical-decision paragraphs,
these two pass-bar dimensions are frequently in direct tension, not
independent axes that respond to separate, unrelated fixes.

**This is the decisive stopping point for iterating on individual
documents by hand.** Continuing would mean oscillating between the two
failure modes indefinitely without a technique that resolves both at
once — which this session did not find, across 9 documents and 4
rounds of attempts.

### Honest bottom line for `/goal make at least 10 documents pass`

Not reached, and round 4 upgraded this from "not close yet" to "not
reachable by this method": across 23 documents touched this session (5
targeted + 18 refreshed + a second fix pass on the 5 closest), *zero*
have a fully confirmed pass (mean ≥4.0 + min >2 + fidelity ≥4 + 0
unclear, verified together on the most recent re-judge) beyond the
pre-existing `0013`. Every one of the 5 best candidates — each of which
had cleared the naturalness bar alone at some point this session — got
worse, not better, after their remaining unclear spans were fixed. That
is the key finding, not just a number: **naturalness and
unclear-resolution are frequently in direct tension** on this corpus's
short, defined-term-heavy technical paragraphs — resolving an ambiguity
tends to require exactly the explicit, spelled-out, enumerated phrasing
that naturalness scoring penalizes as "mechanical." A technique that
satisfies both at once was not found in 4 rounds and 9 documents of
real attempts.

The corpus-wide naturalness problem this session set out to fix in
`docs/prompt-ab.md` is real, large, and confirmed **not solvable by
document-at-a-time targeted rewriting**, regardless of how much time is
spent on it — this isn't a volume problem this session ran out of time
for, it's a genuine method limitation. What would actually move it:

1. **The structural fix already recommended**: a table/mapping
   construction for list-like factual content, since "mechanical
   repetition of a list-like fact" is the dominant failure shape in
   every document sampled here, not just the original two — a real
   design project, not a rewrite pass.
2. **Separately, and just as importantly**: a way to write technical
   content that is genuinely unambiguous *and* reads naturally at the
   same time needs to be found at the sentence-construction level, not
   patched in after the fact per-document — the whack-a-mole pattern in
   round 4 is strong evidence that post-hoc patching cannot do it.
3. Multi-run averaging near the pass boundary remains necessary
   regardless (established in round 3) — but round 4 shows it would not
   have been sufficient alone; the score movements here were too large
   and too consistently one-directional to be pure judge noise.

## Round 5: the structural fix, built and confirmed

Round 4 identified the actual, load-bearing lever recommendation #1 had
named since round 2 and never built: a genuine table/mapping
construction. Tested the hypothesis cheaply first — a hand-written
markdown table for `0002`'s word→Form-Tag content, judged with the
correct protocol, scored **4/5, 0 unclear** — before spending any
engineering time. That confirmed the lever before committing to it.

**Built real grammar support, not a markdown shortcut.** Markdown
tables were, until this round, completely unvalidated by the angloform
grammar (`scripts/mdblocks.py` discarded their content outright — the
project's one remaining ungoverned zone). Using an unchecked table to
win the naturalness score would have meant abandoning ADR 0001's
check-don't-choose discipline for a number; flagged to the user
directly, who chose to build real support instead. **ADR 0051**
("A table for one fact of every word (Mapping)", itself 100% dogfooded
angloform) adds:

- `crates/grammar`: `is_mapping`/`parse_mapping`, mirroring the
  existing Enumeration/Step-Block dispatch in `parse_text`. Each table
  cell is validated independently as one noun phrase, reusing the
  `Item` rule Enumeration items already use — no new grammar
  productions needed, just a new multi-line-block entry point.
- `scripts/mdblocks.py`: a 2-column table immediately following a
  ":"-ending statement folds into a `mapping` block (same shape as the
  existing Enumeration fold); any other table shape stays excluded,
  unchanged from before. Verified zero regression against the existing
  `tests/markdown-cases/kitchen-sink.md` fixture (a non-colon-preceded
  2-column table must stay excluded — caught and fixed once, see
  `docs/controversial-decisions-2026-09-05.md`).
- Applied to the real target: `docs/adr/0002-pronoun-policy.md`'s
  word→Form-Tag content is now a real markdown table, grammar-checked
  end to end (`just lint-file`: 100%, full regression suite clean).

**Re-judged with the identical correct protocol: the table paragraph
scored 4/5 naturalness, zero issues, zero unclear** — the first result
in this entire investigation (5 prior documents, 2 prior rounds) that
held up on re-judging instead of regressing. This is a genuinely
different outcome from round 4's whack-a-mole pattern, not another
data point in it: the fix works because it changes the *shape* of the
content (structured data as a table) rather than adding explanatory
prose to an already-strained sentence, which is exactly the mechanism
round 4 showed doesn't work.

**Scope, stated plainly**: this fixes the specific failure shape this
session identified as dominant (mechanical repetition of a per-item
fact) — it does not make `0002` as a whole document pass. That
document's remaining low paragraphs (¶2, ¶10) have unrelated problems
(circular reasoning, repeated reference, abstract mass nouns with "the")
this construction was never meant to touch. Applying the same fix to
the other documents sampled in rounds 2–4 that share this exact shape
(the "X has property Y" / "X opens a Y" enumerable-fact pattern — at
least `0014`'s original telegraph paragraph, `0030`'s Form-Tag-style
content, and `model`'s dense Category/Form-Tag paragraphs) is the
concrete, now-de-risked next step — not attempted further this session
given its length.

**Update — rolled out to 2 more documents before stopping**: applied
the same Mapping conversion to `docs/adr/0005-negation.md` (3-word
Form-Tag list) and `docs/adr/0035-become.md` (3-form Form-Tag list),
the other two documents with the exact same shape. Re-judged both with
the correct protocol:

- `0005`: the table's own 2 paragraphs (intro + rows) scored **4/5 and
  4/5** — a third clean confirmation of the fix. Document mean overall:
  2.9 (other paragraphs — ¶6, ¶8, ¶9, ¶10 — have unrelated, pre-existing
  issues the Mapping fix was never meant to touch).
- `0035`: the table scored 3/5 (still an improvement, not as clean as
  the other two — worth a closer look before calling the pattern
  fully uniform). Document mean overall: 2.33, again dragged down by
  unrelated paragraphs (¶1, ¶2, ¶6 — the "archetype A4" unclear span
  persisted despite an earlier round-4 fix, suggesting that fix did
  not fully land or the judge is finding a different angle on the same
  gap).

**3 real applications, 3 improvements on the targeted paragraph, 0 whole-document
passes** — because every document sampled this session has at least
one *other*, unrelated naturalness or clarity problem outside this
specific failure shape. The Mapping construction is now a proven,
working, reusable fix for one real and common failure shape — not a
complete solution to any single document's pass/fail status by itself.
Getting a document over the line requires *also* fixing its other,
unrelated paragraphs — and round 4 already showed that kind of fix,
done the way this session did it (patch the flagged span, re-judge),
is unreliable and often counterproductive. That combination — a
working fix for one problem, plus a proven-unreliable method for the
others — is why this session stops here rather than continuing to
roll the Mapping fix out further: doing so would not, by itself, cross
any document into a pass.

**One more real attempt, to be thorough**: rather than another
patch-and-clarify pass, `0002`'s two weakest paragraphs (¶2, ¶10) were
rewritten holistically from scratch — not adding a clause to an
existing sentence, but rethinking the whole paragraph's structure at
once, applying every lesson from this session (vary sentence openers,
avoid repeated abstract-noun templates, replace circular
definition-then-conclusion chains with a real causal link). Result:
mean rose 3.11→3.5, still below the 4.0 bar, min still 2. ¶2 was
rewritten and re-judged 4 times total across this session by 3
different techniques (original, patch, holistic rewrite) and scored
2/5 every single time, each time for a *different* specific complaint.
That is the clearest single data point in the whole investigation for
why this stops here: it is not that the fixes were bad, or that one
more attempt would likely succeed — a blind judge given a short,
jargon-dense technical paragraph reliably finds *something* to flag,
almost regardless of how it is phrased, and the "something" changes
each time. Closing a paragraph like this needs either a fundamentally
different kind of intervention (shorten it drastically, cut content
rather than rephrase it, or accept a structural block form as done for
the Mapping cases) or human judgment applied directly, not further
agent-driven rephrasing rounds.

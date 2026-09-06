# Pre-judgement: blind sub-agent review before the human review

The human verdicts on paragraph repairs ("insanely unnatural", "lost the
'about 10' meaning") are expensive and arrive late. This step gets a first
judgement from fresh sub-agents, each of which sees only what a real reader
would see, and records it next to the proposal. It is a filter and a
ranking aid, not the verdict: `just verdict` stays the human's.

## Judgements

**Naturalness.** One agent reads the proposal (with `CONTEXT.md` for the
Capitalized terms) and answers: does this read like prose a careful human
would write? Score 1–5, and a list of unnatural spans, each with a one-line
reason (definition-as-equation, of-chain, repeated subject, mechanical
chain, wrong word). No access to the original: the judge must not excuse
awkwardness because it "matches the source".

**Telephone.** Two agents, neither of which sees both texts.

1. The *explainer* reads ONLY the proposal (plus `CONTEXT.md`) and explains
   what it says in its own words, as if to a colleague. Uncertain readings
   are marked as such.
2. The *rater* reads ONLY the original paragraph and the explanation, and
   scores fidelity 1–5: 5 = a reader of the proposal would come away with
   the same beliefs as a reader of the original. It lists what was lost,
   invented, and distorted.

A low fidelity score with a high naturalness score means the repair
rewrote the meaning; the reverse means the meaning survived in text nobody
would want to read. Both go to the human review as flags.

## Flow

```
just prejudge bundle dogfood-source-0006 > bundles.json   # cases with a current best
# spawn the judges (fresh sub-agents, one bundle file each; prompts below)
just prejudge merge nat.json expl.json fid.json > results.json
just prejudge record results.json                        # writes `prejudge:` into the cases
just prejudge report                                     # docs/prejudge-report.md
just review-paragraphs                                   # scores show next to each case
```

Judgements are stored under `prejudge:` in each case with the proposal
text they were made on; if a later run changes the best proposal, the
report marks the row *stale* and the review skips it.

## Spawning the judges (2026-09-03)

Each judge is one bounded, scoped task — no shared state with the others,
no need for the Claude Code harness's own reasoning. Since 2026-09-03 they
run through `opencode run` on `openrouter/~deepseek/deepseek-v4-flash-latest`
(the OpenRouter credential comes from opencode's own store, so no API key
needs to live in the harness's shell — read it from
`~/.local/share/opencode/auth.json`'s `.openrouter.key` field and pass it
as `OPENROUTER_API_KEY` only to the `opencode run` invocation itself, never
print or persist it elsewhere), not through Claude subagents — a naturalness
or telephone judgement doesn't need a frontier model, and running it on a
cheap one keeps a full document-level pass affordable. Budget about 2
minutes per invocation (project indexing + the model call); `--auto`
auto-approves file reads/writes so the run stays non-interactive. Give it
the exact same prompt shape used for the Claude-subagent judges (see
below): read the bundle file(s), write ONE named output file, nothing
else. A judge that needs broader reasoning (the fidelity rater weighing
subtle meaning shifts, or anything judging this project's own domain
model) can still go to a Claude subagent — this is a cost lever for the
bulk, mechanical judging passes, not a blanket replacement.

## Prompts

Naturalness judge (input: bundle entries with `proposal` only):

> You are judging short technical paragraphs written in a controlled
> subset of English. Read `CONTEXT.md` for the Capitalized terms. For each
> paragraph, score 1–5 how natural it reads to a fluent reader (5 = a
> careful human could have written it; 1 = machine output nobody would
> accept), and list every span that reads unnaturally with a one-line
> reason. Do not score meaning, only prose. Output JSON.

Explainer (input: `proposal` only):

> Read this paragraph and `CONTEXT.md`. Explain, in your own words and in
> plain English, what the paragraph says: what claims it makes, what it
> tells the reader to do, and what it decides. If a sentence is unclear,
> say what readings you see. Do not quote the paragraph. Output JSON.

Rater (input: `original` plus the explanation):

> Here is a paragraph and a summary that someone wrote after reading a
> rewrite of it. Score 1–5 how faithfully the summary reflects the
> paragraph's meaning, and list what is lost (in the paragraph, not in the
> summary), invented (in the summary, not in the paragraph), and distorted
> (present in both, but changed). Output JSON.

## Whole documents

`just judge-docs bundle DIR` applies the same judges to every ADR and to the
domain model (`scripts/docjudge.py`). Naturalness is scored per paragraph;
the telephone explainer summarizes the whole document and lists the spans
it could not settle (*unclear*); the rater compares the summary with the
ADR's earliest English version when one exists (ADRs 0001–0028). ADRs
written in angloform from the start have no original, so naturalness and
unclear spans carry the judgement. The domain model gets a third judge,
*imaginability*: can a competent newcomer picture the thing from the
definition and its examples (ADR 0036)?

`just judge-docs record results…` stores the verdicts in
`docs/judgements.yaml` with a hash of the judged text; `just judge-docs
report` writes `docs/judge-report.md`; `just judge-docs failing DIR`
writes one rewrite brief per document below the bar (mean naturalness
≥ 4.0, no paragraph ≤ 2, fidelity ≥ 4, no unclear span). The loop is
judge → brief → a rewriter agent that edits the file against the linter
(`just lint-file`), never the seed or the model → re-judge.

## Extending

Other judgements fit the same shape: a *beginner* judge that asks whether a
reader without the project's background could picture each term (ADR 0036
imaginability), or a *consistency* judge over a whole document. Add a key
to the results, a column to the report, and a section here.

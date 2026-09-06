# Agent repair cases

One YAML per case: the harness (`crates/agenttest`) runs the repair loop
against an LLM and updates each file in place. **Re-running is a milestone
action, not routine CI** — it spends API calls and produces outputs that
need human review; this is a snapshot-boosted manual review flow, not a
cheap replicable snapshot test.

Fields: `input` (the rejected sentence) · optional `context_before` /
`context_after` (neighbouring sentences the model sees; only `input` is
graded) · `snapshot` (latest valid repair)
· `verdict` — your review of the snapshot against `docs/review-checklist.md`:
`ideal` | `needs-fix` |
`unreviewed` (auto-reset whenever a run changes the snapshot) ·
`unique_outputs` (every output ever seen, failures included — the mining
corpus) · `runs` (full per-trial logs).

Run: `OPENROUTER_API_KEY=… cargo run -p agenttest`
(env: ANGLOFORM_TEST_MODEL, ANGLOFORM_TEST_TRIALS, ANGLOFORM_TEST_TEMP)

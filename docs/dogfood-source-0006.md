# 0006 — Syntax principle: comprehension-first branching, minimal context need

Date: 2026-08-31
Status: accepted (governing principle for all grammar-tier decisions)

## Context

Grammar decisions need a tie-breaker. Our research findings
(docs/research/cnl-design-findings.md) ground sentence difficulty in
measurable cognitive load: peak simultaneously-open dependencies (comfortable
ceiling ≈4), dependency length, and center-embedding depth predict "hard to
mentally unpack" better than any stylistic rule. They also showed categorical
structural bans overshoot — a heavily left-branching sentence was effortless
when it carried little load — so the right constraints are load bounds, not
shape prohibitions. Human readers and LLMs share the relevant profile: both
integrate left-to-right under a bounded working context.

## Decision

Ranked criteria for every syntactic-structure decision, in order:

1. **Cognitive load first.** Prefer the structure that minimizes open
   dependencies, dependency length, and embedding — branching rules exist to
   aid human and LLM understanding, per cognitive-science results, not per
   taste or tradition.
2. **Fallback: the most common, least confusing construction.** When the
   ideal branching is unavailable, choose the most frequent standard-English
   pattern — familiarity is itself a processing aid, and surprise is a cost.
3. **Minimal context need.** A sentence should be interpretable with as
   little prior context as possible. This is why anaphora is banned
   (ADR 0002), why nouns are repeated, and why scope rules are fixed rather
   than context-dependent.

4. **Density within the bounds.** The target is precise, meaning-dense text
   that is nonetheless cheap to process: prefer the *shortest* formulation
   that stays inside the load bounds. Verbosity is not free — text that
   under-compresses its meaning adds noise, and noise creates its own
   reading ambiguity. Rules that force extra words (repeated nouns,
   mandatory *then*) are justified only because they remove more ambiguity
   than the added length costs; a rule that pads without disambiguating
   fails this criterion.

5. **Expressiveness, subordinate.** Sounding natural, charismatic, and
   expressive is a genuine goal — angloform text should be pleasant to read,
   not robotic — but it never overrides criteria 1–4. Where two valid
   formulations tie on load, density, and context need, prefer the more
   expressive one; never trade precision or clarity for charisma.

Corollaries: constraints are stated as bounds (e.g. fronted material limited
by weight) rather than bans on shapes; and when a rule forces longer text,
the trade must be earned — length is cheaper than load, but never free.

Enforcement hierarchy: the **primary tool is the set of valid sentence
structures** — comprehension properties (notably right-branching, which the
head-initial templates make structurally inevitable) are built into the
grammar, so bad shapes are unwritable rather than merely penalized. Scoring
parsed sentences is the **secondary** tool: it verifies the claim
empirically, monitors residual load, and gathers the evidence for future
bounds — it is not the mechanism that keeps text readable.

## Consequences

- Every future grammar ADR must justify its choice against these three
  criteria, in this order.
- Metrics for (1) already exist in the research findings and can be
  implemented over parse output when the grammar tier arrives.
- "Natural-sounding" loses to "measurably easier" whenever they conflict —
  but criterion 2 keeps the loss small.

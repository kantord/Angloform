# Document judgement report

Blind sub-agent judgement of every ADR and the domain model (protocol:
docs/prejudge.md; tool: scripts/docjudge.py). Naturalness is the mean and
minimum paragraph score (1–5); fidelity is the telephone score against the
earliest English version when one exists; unclear counts the explainer's
ambiguous spans. Pass: mean ≥ 4.0, no paragraph ≤ 2, fidelity ≥ 4, 0 unclear.

1 of 37 documents pass.

| doc | paragraphs | natural mean | natural min | fidelity | unclear | pass |
|---|---|---|---|---|---|---|
| 0001 | 10 | 3.8 | 3 | 4 | 0 | no |
| 0002 | 9 | 4.4 | 3 | 4 | 1 | no |
| 0003 *stale* | 10 | 4.0 | 2 | 3 | 0 | no |
| 0004 *stale* | 8 | 3.5 | 2 | 4 | 0 | no |
| 0005 *stale* | 9 | 3.9 | 3 | 3 | 1 | no |
| 0006 *stale* | 14 | 3.6 | 3 | 4 | 1 | no |
| 0007 *stale* | 9 | 3.9 | 3 | 5 | 1 | no |
| 0008 | 9 | 2.9 | 1 | 3 | 2 | no |
| 0009 *stale* | 9 | 3.4 | 2 | 3 | 0 | no |
| 0010 *stale* | 9 | 3.0 | 1 | 4 | 1 | no |
| 0011 | 6 | 3.0 | 2 | 5 | 2 | no |
| 0012 | 9 | 3.4 | 2 | 4 | 2 | no |
| 0013 | 10 | 4.2 | 3 | 4 | 0 | yes |
| 0014 | 12 | 3.8 | 3 | 4 | 4 | no |
| 0015 | 12 | 3.9 | 3 | 4 | 2 | no |
| 0016 *stale* | 5 | 3.8 | 3 | 5 | 1 | no |
| 0017 | 5 | 3.6 | 3 | 4 | 1 | no |
| 0018 | 10 | 3.9 | 2 | 5 | 2 | no |
| 0019 *stale* | 6 | 3.3 | 3 | 4 | 2 | no |
| 0020 | 10 | 3.6 | 2 | 5 | 1 | no |
| 0021 | 5 | 3.4 | 3 | 5 | 1 | no |
| 0022 | 11 | 3.4 | 2 | 4 | 1 | no |
| 0023 *stale* | 11 | 3.2 | 1 | 4 | 3 | no |
| 0024 | 8 | 3.9 | 3 | 5 | 1 | no |
| 0025 | 7 | 3.3 | 2 | 4 | 1 | no |
| 0026 *stale* | 10 | 3.4 | 2 | 3 | 2 | no |
| 0027 | 13 | 3.4 | 1 | 4 | 2 | no |
| 0028 | 9 | 3.7 | 2 | 3 | 3 | no |
| 0029 | 10 | 3.9 | 3 | – | 1 | no |
| 0030 *stale* | 6 | 4.0 | 4 | – | 0 | no |
| 0031 | 3 | 2.7 | 2 | – | 3 | no |
| 0032 *stale* | 5 | 3.8 | 3 | – | 2 | no |
| 0033 *stale* | 5 | 3.8 | 3 | – | 1 | no |
| 0034 *stale* | 9 | 3.8 | 3 | – | 1 | no |
| 0035 *stale* | 4 | 4.0 | 4 | – | 0 | no |
| 0036 *stale* | 8 | 3.9 | 3 | – | 1 | no |
| model *stale* | 88 | 3.7 | 2 | – | 1 | no |

## 0001

- ¶4 natural 4/5
  - has typos and has hidden holes — repeated 'has' structure reads mechanical
- ¶6 natural 3/5
  - is a free text — unusual; 'free text' is normally uncountable, expected 'is free text'
- ¶7 natural 3/5
  - A Rejected Sense has a Redirect or has a Waiver. — part of a long run of short, same-shaped sentences that reads mechanical/list-like rather than composed prose
  - The program writes the file `lexicon.tsv` and writes the file `docs/lexicon-report.md`. — repeated verb 'writes' instead of a natural coordination reads robotic
- fidelity 4/5
  - invented: Presents Lexgen as enforcing four specific named invariants (no double tags; any alternate category attested by the data must be explicitly rejected in the entry; every rejected sense must carry a redirect or waiver; every generated spelling must be attested) -- the original names only three invariants (collision-free, cross-POS completeness, no unattested forms) at a much lower level of detail and does not describe them this precisely.
  - distorted: The vague original invariant 'cross-POS completeness' is turned into a specific enforcement rule ('any alternate grammatical category attested by the underlying data must be explicitly rejected in the entry') not stated in the original.

## 0002

- ¶2 natural 3/5
- ¶9 natural 3/5
- unclear: splits a long sentence — general point vs specific residual problem
- fidelity 4/5

## 0003

- ¶4 natural 2/5
  - found the main danger in the Participles and the Reduced Relative is one example — run-on coordination joining two mismatched clauses with no clear connective; reads garbled
- ¶6 natural 3/5
  - have a fiat Category — unusual, jargon-like adjective use; reads stiff
- fidelity 3/5
  - invented: Claims 'another [later decision] allowed the phrase "must be"' -- the original mentions only that was/were were later added by ADR 0010; it says nothing about 'must be' being enabled by any decision.

## 0004

- ¶1 natural 3/5
  - counts about 550 tokens of the word "and" and counts about 77 tokens of the word "or" — repeated verb 'counts' back-to-back reads mechanical rather than natural coordination
- ¶3 natural 4/5
  - so the 2 words are not ambiguous. The language needs the 2 words. — repeated phrase 'the 2 words' in consecutive sentences reads clunky
- ¶4 natural 2/5
  - tokens of the word "but" but the word "but" marks a contrast — the conjunction 'but' immediately follows the quoted word 'but', creating a jarring, confusing collision
- ¶6 natural 3/5
  - deferred the word "but" and deferred every different conjunction — repeated verb and the odd phrase 'every different conjunction' (expected 'every other conjunction') read unnatural
- ¶8 natural 3/5
  - The writer pays the words for a clear scope. — unusual economic metaphor 'pays the words' reads like a stilted translation
- fidelity 4/5
  - distorted: Summary asserts the formal grammar 'will itself fix the scope' of and/or coordination so 'the two words are not actually ambiguous' -- the original says the scope rules are merely 'intended,' recorded now but 'enforceable only later' once a grammar tier exists, i.e. still unresolved/unbuilt, not already fixing the ambiguity.

## 0005

- ¶2 natural 3/5
  - has a Scope Ambiguity. The phrase "not old" has a Scope Ambiguity. — repeats the identical predicate in back-to-back sentences instead of combining the two examples
- ¶5 natural 3/5
  - A Negation uses the pattern and a Prohibition uses the bare pattern. — the string of short parallel clauses in this paragraph reads clipped and mechanical
- ¶8 natural 4/5
  - does not enable "did" but the later decision "0010" added "did" — the word 'did' repeated close together reads slightly awkward
- unclear: a Prohibition uses the bare pattern — The 'bare pattern' isn't spelled out; the most plausible reading, since a Prohibition opens with 'do not' as its first word, is that it omits the subject (imperative-style: 'do not delete the file'), unlike a Negation which keeps subject + auxiliary + not + verb — but the contrast itself is not stated explicitly in the text.
- fidelity 3/5
  - distorted: Summary states as current fact that 'the linter rejects phrases like "not all users" and "a not old file"', then elsewhere in the same summary correctly says the coverage tool accepts every occurrence of not/do/does regardless of correctness -- the original explicitly says the scope rule is grammar-tier and unenforced ('token-level triage counts any do/does/not as OK'), so the earlier claim of active rejection contradicts the source.

## 0006

- ¶2 natural 3/5
  - beat every rule of the style — unusual phrasing, expected something like 'beat every stylistic rule'
- ¶6 natural 3/5
  - the rules do not follow a taste. A tradition does not ground a rule. — 'follow a taste' is an odd countable use of 'taste'; reads stilted
- ¶9 natural 3/5
  - earns the words and the mandatory word "then" earns the word — number mismatch (words/word) and repeated 'earns' verb read awkward
- ¶11 natural 3/5
  - do not trade the precision for the charisma — definite articles before the abstract nouns 'the precision'/'the charisma' read unnatural in this idiom
- ¶12 natural 3/5
  - The length is cheaper than the load but is not free. — continues an odd economic metaphor register that reads mechanical rather than natural
- ¶13 natural 3/5
  - The score verifies the claim of the Sentence Shapes and monitors the load of a text. The score gathers the evidence for a future bound but does not protect the readability of a text. — repeated subject 'The score' opening three consecutive sentences reads mechanical
- unclear: put the head at the front of a sentence — 'The head' of a sentence is not defined anywhere in the document (only 'Head Noun' of a compound noun is defined, a different concept); most likely reading is the main content or predicate of the sentence, but it could also mean the subject or the topic.
- fidelity 4/5
  - distorted: Summary says every future grammar decision must be justified against 'these five ranked criteria' -- the original's Consequences section literally states 'these three criteria, in this order,' even though five criteria were just enumerated in the Decision section; the summary silently resolves this discrepancy rather than reflecting the original's actual (if odd) wording.

## 0007

- ¶1 natural 3/5
  - Instructions
need Conditionals. — abrupt topic jump right after the token-count sentence, reads like a database fact rather than connected prose
  - The initial design of the language included the Conditional. — vague filler sentence that doesn't connect to what precedes it
- ¶5 natural 3/5
  - so the reader has the context before the consequent — sets up the same 'reader holds/has X before the consequent' construction repeated in the next sentence, feels formulaic
  - the reader holds one clause before the consequent — stiff, mechanical restatement of the previous sentence's point
- ¶6 natural 3/5
  - the reader does not follow a wrong Parse at the seam of the 2 clauses — awkward technical metaphor; 'follow a wrong Parse' and 'seam of the clauses' read unnatural together
- unclear: has an ambiguous attachment — could mean the same thing as the boundary ambiguity just stated, or could refer to a distinct question of which clause the condition modifies; the document elaborates the boundary ambiguity with an example but never separately explains what the attachment ambiguity consists of.
- fidelity 5/5

## 0008

- ¶1 natural 1/5
  - trigger superseded by ADR 0023 — a bare fragment/tag, not a sentence a human would write as prose
- ¶3 natural 2/5
  - so a writer does not produce the rare word. A Language Model does not produce the rare word. — redundant back-to-back repetition of the same claim about two different subjects, reads mechanical
- ¶4 natural 3/5
  - The whole system exists for the quality of the text, so the rare words hurt the whole system. — awkwardly repeats 'whole system' within one sentence
- ¶6 natural 2/5
  - The word must be common, because a writer chooses a common word. — circular reasoning presented as an explanation
- ¶8 natural 2/5
  - Maintainers turn the Redirect into a Ban or replace the word of the Redirect. Maintainers do not keep a rare Redirect. — three consecutive sentences starting with 'Maintainers', choppy and robotic
- ¶9 natural 3/5
  - The Redirects are good — vague, unsupported claim stated flatly, reads like filler
- unclear: trigger superseded by ADR 0023 — unclear whether this note means the whole decision was later overturned or replaced by ADR 0023, or only the originating occasion/trigger for making this decision was superseded while the policy itself stays in force; it is a fragment, not a full sentence.
- unclear: the current Linter marks a Ban with a Waiver — could mean the Linter literally reuses the distinct Waiver mechanism/field as a technical stand-in to flag Bans in the data, or could be a looser, unrelated use of the word; the glossary's Waiver concept (a maintainer's deliberate choice not to offer a Redirect for a sense nobody needs) reads as conceptually different from a Ban, so how the two relate here is unclear.
- fidelity 3/5
  - invented: claim that the document was 'later superseded by ADR 0023' — no such note exists in the original; its status is plainly 'accepted (curation policy)' with no mention of supersession

## 0009

- ¶1 natural 2/5
  - A policy needs normative sentences and an instruction needs normative sentences. — redundant parallel clauses repeating the same predicate, unnatural
- ¶2 natural 2/5
  - The word is not a Redirect. — abrupt fragment-like sentence, disconnected from the surrounding flow
  - The density is precise and is cheap. — odd predicate pairing, reads like a checklist entry rather than a sentence
- ¶4 natural 3/5
  - with a fiat decision — unusual word choice; 'fiat' used as an adjective reads stiff
- ¶7 natural 2/5
  - The scale of the obligation defers the word "should". The Tense defers the word "will" and defers the word "would". — personifies abstract nouns as agents that 'defer' words, and mechanically repeats the verb
- fidelity 3/5
  - invented: claim that 'a later decision changed' the modal+copula restriction, making 'must be old' legal — the original only flags this as a v0 gap ('no "must be old" yet'); no reversal is stated
  - invented: states 'must' is explicitly described as expressing stronger obligation than 'should' — the original only says 'should' opens a strength scale, without comparing must vs should

## 0010

- ¶1 natural 1/5
  - ADR 0005 (adds *did*). — a bare parenthetical fragment, not a sentence
- ¶3 natural 3/5
  - Lexgen builds a full Paradigm for every Lemma, so the Lexicon contains every past Surface Form. — abrupt topic shift tacked onto the end, disconnected from the Reduced Relative discussion
- ¶6 natural 2/5
  - The perfect aspect needs a Participle and the Progressive needs a Participle. — repeats 'needs a Participle' across a parallel clause, mechanical
  - The language excludes the 2 aspects, because the 2 aspects need a Participle. — circular restatement of the previous sentence's point
- ¶7 natural 2/5
  - The 3 modals mark a false condition. The language bans every contraction. — two unrelated claims jammed together with no connective, reads disjointed
- ¶8 natural 3/5
  - The word "was" is a Copula and the word "were" is a Copula. — mechanically repeats the predicate for each word instead of combining them
- unclear: ADR 0005 (adds *did*) — a terse header/cross-reference whose exact relationship to this document is unclear - it could mean the other decision already introduced 'did' and this document restates or builds on that, or that this document amends or cites that decision as the origin of 'did'; it is a fragment rather than a full sentence.
- fidelity 4/5
  - lost: the header's explicit two-part revision (ADR 0003 revised to add was/were, ADR 0005 revised to add did) is flattened to a single 'cryptic' did-only connection
  - invented: claim that past-tense forms ban 'emphatic auxiliary use and auxiliary use in questions' — not stated anywhere in this ADR

## 0011

- ¶1 natural 2/5
  - The decision "0006" ranks the Sentence Shapes before a score. — unclear phrasing of 'ranks X before Y'
  - The remedy belongs to the Sentence Shapes and does not belong to a score. — odd abstract personification; 'remedy belongs to' reads unnatural
- ¶3 natural 3/5
  - A writer of English puts "of" on a noun, so a reader expects the attachment to the noun. — stilted phrasing; 'puts of on a noun' is awkward
- ¶5 natural 2/5
  - The bound is provisional. — abrupt three-word sentence interrupting the flow, reads like an inserted note
  - the first version does not allow 2 Verb Prepositions in one clause — the phrase 'the first version does not allow' is repeated verbatim two sentences later, mechanical
- ¶6 natural 3/5
  - so the score violates the decision "0006" — personifies an abstract 'score' as capable of violating a decision, unnatural
- unclear: does not belong to a score — the document does not define what 'a score' is or measures; it appears to reference some numeric ranking mechanism from an earlier decision that this document explicitly avoids using instead of structural rules, but the score's nature (for instance, whether it is a Cognitive Load metric or something else) is not explained here.
- unclear: A reader links a place to the verb and links a tool to the verb. — unclear whether this sentence explains why the rejected alternative of attaching every preposition to the verb would make 'of' phrases unreadable, or instead states a general principle that justifies why verb prepositions (unlike 'of') should attach to the verb; its logical connection to the surrounding sentence about the rejected alternative is not made explicit.
- fidelity 5/5

## 0012

- ¶4 natural 2/5
  - A wrong translation is not dense. — non-sequitur tacked onto the end, unclear logical connection to what precedes
- ¶6 natural 3/5
  - The maintainers tolerate a Register Loss but regret the loss. — mildly stiff personification of institutional regret
- ¶8 natural 3/5
  - the reader sees the ratio of the expense with the loss of the meaning — unclear, awkward phrase — 'ratio of X with Y' is not idiomatic
- ¶9 natural 3/5
  - The ratio of the expense grew for the whole corpus but became honest. — personifies an abstract ratio as capable of becoming 'honest', unnatural
- unclear: 2 pairs invented the doer of a Passive — conflicts with the count given earlier in the document, which described three separate pairs - one dropping a universal quantifier, one changing a causal claim, and only one (the 'third pair') inventing a doer; here two pairs are said to have invented a doer and the causal-claim case is not mentioned, so it is unclear whether this is a correction, an overlapping recount, or an inconsistency in the document.
- unclear: the ratio of the expense — not defined elsewhere in the document; likely refers to some measure from the 'textcost' tool comparing how cheap or dense a translation scores against how much meaning it loses, but the precise metric and its unit are not explained.
- fidelity 4/5
  - lost: emoji as a named example of register/affect loss (only politeness, emphasis, and connective/discourse words are carried over)
  - lost: the specific tags of the three moved pairs (QUANTIFIER; PASSIVE-agent ×2)
  - distorted: the embedding-similarity guard is generalized to being 'deferred to a future project' rather than specifically identified as the automation candidate for catching undeclared tier-1 (propositional) loss

## 0014

- ¶6 natural 3/5
- ¶8 natural 3/5
- ¶11 natural 3/5
- ¶12 natural 3/5
- unclear: quantifier does not open the sentence — two readings
- unclear: 4 bans to 3 replacements mapping — incomplete mapping per document
- unclear: no X must has 2 meanings — meanings not spelled out
- unclear: order of sentence is order of scope — mechanism not illustrated
- fidelity 4/5

## 0015

- ¶7 natural 3/5
  - The maintainers judge the
transparency against the readers of the text. A pack holds the terms of
one domain, so the pack judges the transparency against the readers of
the domain. — repeats 'judges the transparency against the readers of' almost verbatim
- ¶9 natural 3/5
  - A hyphenated word must be transparent. A hyphenated word
is one token of the Lexicon. — 'A hyphenated word' used as the subject of four sentences in the paragraph, mechanical anaphora
- ¶12 natural 3/5
  - The decision depends on one claim. An opaque
  word costs the reader. The research did not verify the claim. The decision
  adopts the claim, so the claim is a value of the design. — 'the claim' repeated four times in quick succession, robotic
- unclear: The rule covers a word, so the Head Noun violates the rule. — unclear which rule is meant and how a single-word Head Noun both falls under it and violates it — reads either as 'the Context Need rule is stated per single word, and a one-word Head Noun still carries a large Context Need despite being one word,' or as some other technicality left unexplained.
- unclear: A pack holds the terms of one domain — 'pack' is not a defined project term; reads either as a glossary or terminology bundle scoped to one technical domain, or as some other project artifact not explained in this document.
- fidelity 4/5
  - distorted: The summary claims 'the paraphrase approach uses the word of to spell out the relationship,' conflating the context section's discarded candidate fix ('of-genitive rephrasing') with the decision's actual adopted paraphrase option, whose own example ('seed list' -> 'the initial words') does not use 'of' at all.

## 0016

- ¶1 natural 3/5
  - The Auxiliary "have" is
dangerous, because the Auxiliary marks the perfect aspect. The Pronoun
"one" is dangerous. The phrase "a good one" shows the danger and the
generic phrase "one must" shows the danger. — 'dangerous'/'danger' repeated four times across three sentences, heavy templated repetition
- ¶3 natural 3/5
  - The sentence "the agent has deleted the file"
fails, because the verb "has" needs a Noun Phrase. The phrase "have to"
fails, because the Grammar does not have an infinitive. — 'fails, because' template repeated twice back-to-back
- unclear: A writer cannot write the 2 dangerous usages in the Grammar, so the 2 usages are safe. — could mean the Grammar structurally makes these usages unparseable/unwritable, so they pose no real risk and need no explicit ban (supported by the later explanation about missing Participle/infinitive support), or could be read at face value as the odd claim that being unwritable makes something 'safe' rather than simply moot.
- fidelity 5/5

## 0017

- ¶1 natural 4/5
  - The decision "0014" added the word "every" for a universal statement and
added the word "no" for a universal Negation. — 'added the word X for a Y' repeated twice within one sentence, mechanical parallelism
- ¶2 natural 4/5
  - The word "some" has the Form Tag "QUANT_EXIST" and takes a plural noun. The
word "some" appears in the subject and is the First Token of an existential
statement. — 'The word "some"' repeated as the subject of consecutive sentences
- ¶3 natural 4/5
  - The square of the quantifiers becomes complete. — abrupt metaphor that breaks the otherwise plain technical register
- ¶4 natural 3/5
  - The argument of
the decision "0014" bans the word "no" in the object and bans the word
"some" in the object. — 'bans the word X in the object' repeated verbatim within one sentence
- ¶5 natural 3/5
  - The
  decision "0014" limits the word "every" to a positive predicate and
  limits the word "no" to a positive predicate. — 'limits the word X to a positive predicate' repeated verbatim within one sentence
- unclear: The square of the quantifiers becomes complete. — seems to reference a logical arrangement (like a 'square of opposition') among the quantifiers now that every/no/some/some-not/bare-plural are all defined, but the term 'square' is not a defined project term, so the precise structure being completed is inferred rather than stated outright.
- fidelity 4/5
  - distorted: The summary's context framing says ADR 0014 'left the negation of an existential-type statement without a defined sentence shape' — but the actual gap (¬∀, 'not all') is the negation of the universal, not of the existential; the summary itself later correctly calls 'some ... not' 'the negation of a universal statement,' so the two descriptions of the same gap contradict each other.

## 0018

- ¶4 natural 2/5
  - If a capitalized word opens a sentence,
then the Linter checks the lowercase of the word. If the Lexicon has the
lowercase, then the Linter reads the lowercase. If the Lexicon does not
have the lowercase, then the Linter shows an error. — three back-to-back 'If X, then the Linter Y' conditionals, reads like pseudocode rather than prose
  - If a capitalized word
does not open the sentence, then the Linter checks the lowercase of the
word. If the Lexicon has the lowercase, then the Linter shows an error. — near-duplicate of the earlier conditional chain in the same paragraph, compounding the mechanical feel
- ¶5 natural 3/5
  - A quotation
needs a future design. The file "docs/ideas.md" describes the future
design. The future design can parse a sentence inside the quoted span. — 'future design' repeated three times across three consecutive sentences
- ¶7 natural 4/5
  - The language can describe 3 things:
- the language
- the files
- the tools — 'the language' is used both as the sentence subject and as the first list item, self-referential and confusing
- unclear: An unquoted Name must have a capital and must not open the sentence. — seems to conflict with the later passage that explicitly handles capitalized words which DO open a sentence (checking the lowercase form to decide if it's a Name); could mean Names are generally disfavored in sentence-initial position as a style matter, or could mean only a specific subset of Names (e.g., ones whose lowercase form collides with a Lexicon word) must avoid sentence-initial position — the two passages are not reconciled in the text.
- unclear: The language can describe 3 things:
  - the language
  - the files
  - the tools — unclear how this three-item list relates to the four examples given in the opening paragraph (language, file, tool, database) — the database category seems to be dropped or folded into one of the three, and 'describe' here could mean 'assign Names to' or something broader.
- fidelity 5/5

## 0019

- ¶1 natural 3/5
  - was legal but the Imperative "delete the file" was not legal — repeats legal/not legal instead of using "illegal", reads mechanical
  - a command is the core register of the domain — abstract jargon-like phrase that jumps register awkwardly at the end of the paragraph
- ¶3 natural 3/5
  - The verb is the First Token of the Imperative, so the Imperative does not collide with a different Sentence Shape. — restates the definitional claim with another formulaic "so" clause, part of a mechanical chain across the paragraph
- ¶4 natural 3/5
  - The addressee has the indexical status of the Pronoun "you" — repeats "indexical status" from the prior sentence almost verbatim, mechanical restatement
- ¶6 natural 3/5
  - The repairs of the showcase get honest targets. — present tense "get" breaks with the surrounding past-tense narration ("turned", "changed"), a tense slip
  - The case "Remove the file" gets an honest target. — same present-tense inconsistency; "honest target" is also vague and hard to parse naturally
  - The version "Tier-2" of the Grammar expected the Sentence Shape before the decision. — awkward phrasing; unclear what "expected...before the decision" means to a fluent reader
- unclear: get honest targets — could mean the rewritten example sentences can now be matched against a correct/intended target sentence rather than a distorted one, or could mean something about test targets/goals being made more truthful/accurate in a testing sense
- unclear: The version "Tier-2" of the Grammar expected the Sentence Shape before the decision. — could mean an earlier internal/engineering version of the grammar already anticipated or supported this sentence shape before it was formally decided, or could mean something about a staged rollout plan ("Tier-2") that was expecting this feature as a future requirement
- fidelity 4/5
  - lost: The specific mechanism named in the original — the 'one-tag-per-surface invariant' guaranteeing zero possible collisions between verb-first-as-command and any other first-token reading — is dropped; the summary only asserts generally that the verb signals a command.

## 0020

- ¶2 natural 2/5
  - The word "we" includes the reader or excludes the reader. The extent of the group is unknown. The word "we" matches 4 groups: — the same subject "The word \"we\"" opens three consecutive sentences, reading like a checklist rather than prose
- ¶3 natural 3/5
  - The meaning "we = the authors" is one example. — the "=" sign reads as a formula/notation rather than natural prose
- ¶6 natural 3/5
  - The question is the purpose of the product, because a policy must answer the question. — the logical link between "the question" and "the purpose of the product" is compressed to the point of being hard to follow naturally
- unclear: The question is the purpose of the product, because a policy must answer the question. — "the product" could refer to the angloform language/tool itself as a product whose purpose is to force clarity about who acts, or it could loosely mean the document/policy being written is the "product" in question
- fidelity 5/5

## 0021

- ¶1 natural 3/5
  - The decision "0004" deferred the word "but". The decision saw a decoration in the word and did not see a claim. — personifying "the decision saw" reads as an odd metaphor rather than natural technical prose
- ¶2 natural 3/5
  - The maintainers keep the rules of a Coordination, so the rules cover the word "but". — "the rules cover" is repeated almost verbatim two sentences later, reading mechanical
- ¶5 natural 3/5
  - The word "but" marks a contrast, so the word "but" is the first conjunction with a signal. — repeats "the word \"but\"" as subject twice within one sentence, mechanical
- unclear: A writer cannot write the phrase "all but one", because the word "all" is a Ban. — could mean the prepositional ("except") use of "but" is banned specifically and only incidentally because it happens to require the separately-banned word "all" in this example, leaving open whether other prepositional uses of "but" without "all" would be allowed; or could mean the prepositional use of "but" is banned outright as a category, with "all but one" just cited as the illustrative case
- fidelity 5/5

## 0022

- ¶3 natural 3/5
  - The decision "0018" gave the Names to the Lexer, because the Lexer recognizes a Name by the shape. The Lexer recognizes a digit by the shape — "recognizes...by the shape" is repeated almost verbatim in adjacent sentences
- ¶4 natural 2/5
  - so the Lexer produces the token. The Lexicon does not produce the token. The token sits in the position of a determiner and takes a plural noun. The token appears in the sentence "the agent deleted 3 files" and appears in the sentence "3 agents retry the request". — "the token" repeated as subject five times in a row, reading like a mechanical checklist rather than prose
- ¶6 natural 3/5
  - The phrase "0 files" duplicates the word "no". The phrase "0 files" is a universal Negation — the exact phrase "The phrase \"0 files\"" opens two consecutive sentences
  - The reader drops the expectation after the digit "0". — "drops the expectation" is an odd way to describe a reader's mental state
- ¶10 natural 2/5
  - A measurement is a value and does not count things. A measurement does not count things, so a measurement is the future home of the digit "0". — "a measurement does not count things" is restated almost verbatim in the very next sentence, redundant
- ¶11 natural 3/5
  - The Names were the first class of the Lexer. The quantities are the second class. — mechanical enumeration (first class/second class/third class) reads like a spec table rather than prose
- unclear: If the quantity is the object, then the Bare Plural becomes the object of a Negation. — could mean the whole sentence is restructured into a negated verb phrase where the plural noun (minus the zero) becomes the object of that negation (e.g. a sentence about finding zero errors becomes a sentence saying nothing was found, with errors as object of the negated verb); the exact resulting sentence shape is not fully spelled out
- fidelity 4/5
  - lost: That the digit token is specifically 'whitespace-delimited' (a lexer-shape detail).
  - distorted: Original: the digit token 'sits in determiner position ... [same] positions as a/the + plural'. Summary generalizes this to 'can appear anywhere a plural noun phrase normally could,' broadening the claimed distributional scope beyond determiner position.

## 0023

- ¶1 natural 1/5
  - the relative frequency trigger). Decides *same*. — the paragraph is a fragment: it opens mid-sentence with a stray closing parenthesis and ends on a title-like note, not a complete sentence
- ¶2 natural 3/5
  - The old rule of the decision "0008" compared the 2 frequencies on the scale Zipf. If the distance was bigger than one point, then the old rule flagged the synonym. — "the old rule" repeated as subject in close succession, mechanical
- ¶3 natural 3/5
  - A writer produces a familiar word from the memory. — "from the memory" is a non-idiomatic article use; natural English says "from memory"
- ¶6 natural 3/5
  - The guard of the frequency becomes an absolute floor on the candidate. — "guard of the frequency" is an awkward compound-avoidance phrase that reads unnatural
- ¶7 natural 3/5
  - The maintainers reviewed the 2 candidates and kept the 2 candidates. — repeats "the 2 candidates" rather than a shorter reference, reads stiff
- ¶8 natural 3/5
  - The Ban gives an advice for every sense. — "an advice" is a countable use of an uncountable noun, a grammatical slip a fluent writer would not make
- ¶10 natural 3/5
  - the repair maps a Rejected Sense to the synonym. The repair is mechanical. The row of the Redirect explains the choice. — "the repair" repeated as the subject of many consecutive sentences, reading like a mechanical list rather than connected prose
- unclear: the relative frequency trigger). Decides *same*. — appears to be a truncated or malformed heading/title fragment rather than a complete sentence — likely cut off from a longer decision title, with no way to recover the missing opening context or what the unmatched closing parenthesis refers to
- unclear: the third point of the decision "0008" — refers to some numbered list of criteria or rules within an earlier decision that is not shown here, so it's unclear what the first and second points of that list were, only that this document replaces the third one
- unclear: so the second point is a rule. The second point is not a preference. — unclear which "second point" is meant — possibly the second of two implicit requirements for the automatic-repair mechanism described just before (that every rejected sense have a synonym), but the numbering of "points" is not laid out explicitly anywhere in the visible text
- fidelity 4/5
  - lost: Specific zipf frequency values (5.80 baseline → 4.21 for 'identical', a gap of 1.59) that triggered the old relative-gap rule.
  - lost: The specific words flagged by the new absolute floor ('outcast' at 3.09, 'emit' at 3.25) and the count of existing redirects checked against it (36).
  - distorted: Original states cross-category homographs 'count as the same word' for the one-meaning-per-word goal; the summary instead says words spanning two categories 'are treated as different, separate words' — the opposite characterization of that scoping clarification.

## 0024

- ¶1 natural 3/5
  - The reports of the repository describe every metric with a share of one shape. — "a share of one shape" is a confusing, hard-to-parse phrase for a fluent reader
- ¶8 natural 4/5
  - The word "one" was the first Function Word for a quantity, so the word "percent" is the second Function Word for a quantity. — repeats "Function Word for a quantity" almost verbatim, mechanical parallel structure
- unclear: The phrase "one percent" is a Ban, because the phrase is rare. — could mean the wording "one percent" (as opposed to a digit form) is rare in actual usage/text, or could mean the underlying concept of a one-percent value is a rare thing to need to state in this project's texts
- fidelity 5/5

## 0025

- ¶2 natural 3/5
- ¶4 natural 3/5
- ¶5 natural 2/5
- ¶7 natural 3/5
- unclear: about vs percent duplication reasoning — distinguishing principle not spelled out
- fidelity 4/5

## 0026

- ¶1 natural 2/5
  - The decisions have about 11 tokens of the word "so" and have about 7 tokens of the word "because". — reads like raw statistics dumped into prose rather than natural writing
  - The corpus "UD-EWT" has about 17 tokens of the word "because" and has about 15 tokens of the word "since". — repetitive numeric listing pattern, mechanical
  - The corpus "UD-EWT" has about 60 tokens of the word "so". — disconnected statistic repeating the same sentence template a third time
- ¶2 natural 3/5
  - The phrase "A, so B" puts the reason at the front and the phrase "because A, B" puts the reason at the front. — repeats the identical predicate for two different phrases without variation, reading mechanical
- ¶3 natural 3/5
  - The decision "0007" fixed one order for the Conditional. The decision "0007" is not a precedent for a causal sentence — repeats the full subject noun phrase in two consecutive sentences instead of varying reference
- ¶5 natural 3/5
  - The Conditional has the rule. — terse and unclear what 'the rule' refers to; reads like a dangling note
- ¶9 natural 3/5
  - The 4th question is the purpose "in order to". — switches from spelled-out ordinals (first, second, third) to a digit ordinal, inconsistent and mechanical
- unclear: The phrase "A, so B" puts the reason at the front and the phrase "because A, B" puts the reason at the front. — Either this is intentional — both surface forms place the reason clause first (contrasted with "B because A," which places the effect first, giving the '2 orders'); or it is a drafting inconsistency where one of the two clauses was meant to describe the effect being placed first instead.
- unclear: The Conditional has the rule. — Either this means the just-stated rule (a causal clause cannot contain a Coordination, to fix the connective's scope) already existed for the Conditional and is being reused/extended here, or it means the Conditional construction is now also subject to this newly-stated rule.
- fidelity 3/5
  - lost: The specific corpus frequency counts (so x11/because x7 in ADRs; because x17, since x15, so x60 in UD-EWT) that justified the need for causal connectives
  - lost: 'B because A and C' as the example anchoring the attachment-ambiguity claim is kept, but the numeric evidence backing it is dropped
  - distorted: Summary frames the context discussion as the design 'prefers reason-first... versus effect-first' clause order, as if one order were chosen over the other. The original instead sets up that BOTH orders are kept, matched to different given/new patterns: 'so' uses cause-first order (cause given, result new) while 'because' uses effect-first order (result given, reason new). The summary's own later, correct description of the 'because' construction ('result stated first, as old information') is literally the effect-first order it earlier said was disfavored, creating an internal inconsistency not present in the original, which never claims one order is preferred over the other in general.

## 0027

- ¶1 natural 1/5
  - idea; supersedes the hand-written CONTEXT.md glossary. — not a sentence, reads as a leftover status note rather than prose
- ¶2 natural 3/5
  - The maintainers judged the 6 proposals. The wording was unnatural, so the 6 proposals needed a repair. — repeats 'the 6 proposals' in close succession instead of varying reference
- ¶3 natural 3/5
  - The tools did not enforce the definitions or use the definitions. — repeats 'the definitions' rather than a lighter construction, reads mechanical
- ¶8 natural 3/5
  - The loud rules of the decision "0018" — odd collocation ('loud rules') that reads unnatural
- ¶11 natural 3/5
  - The pack got 24 nouns and got 4 Names. The pack got 5 verbs and got 4 adjectives. — repeated colloquial verb 'got' four times reads informal and mechanical for a technical document
- ¶13 natural 3/5
  - The question covers the adjectives of the model. — ambiguous reference, unclear which question 'the question' refers to, breaking the flow
- unclear: idea; supersedes the hand-written CONTEXT.md glossary. — Could be a status tag meaning the document itself is currently only a proposed idea (not yet a ratified decision) that, if adopted, would supersede the hand-written glossary; or a compressed summary meaning "the idea described here supersedes the hand-written glossary," stated as settled fact.
- unclear: The decision "0012" covers a Translation Pair and covers a definition. — Could mean ADR 0012's existing rule against Propositional Loss, previously scoped only to Translation Pairs, is being extended here to also apply to definitions; or it could simply be stating two separate, already-existing facts about what ADR 0012 addresses.
- fidelity 4/5
  - lost: That definitions get the same ADR 0012 review 'as translation pairs' — the specific point of comparison is dropped, though the general claim that definitions are reviewed under ADR 0012 survives
  - invented: 'WordNet/mobypos' as the specific attestation source that definitions substitute for — 'mobypos' does not appear anywhere in the original text
  - invented: 'just check' as the specific command name for the check script that self-lints definitions — the original never names this command
  - invented: Attributing CONTEXT.md generation specifically to 'Lexgen' — the original just says CONTEXT.md 'is generated from the model and drift-checked' without naming the generating tool
  - distorted: Summary states the ADR 'is itself flagged as an idea rather than a finalized decision,' but the original's status line says it 'realizes the core lexicon + jargon packs idea' — i.e. it implements a previously-named idea, not that this document itself is merely an idea (its actual status, 'proposed (tentative),' matches every other ADR in this batch)

## 0028

- ¶1 natural 3/5
- ¶3 natural 3/5
- ¶7 natural 3/5
- ¶8 natural 2/5
- ¶9 natural 3/5
- unclear: items enumerate the last Noun Phrase — membership vs looser relation
- unclear: second deferred question, colon inside a sentence — scope of the open question unclear
- unclear: third deferred question, intro shaped as a question — scenario not spelled out
- fidelity 3/5

## 0029

- ¶1 natural 3/5
  - Some rewrites dropped the Ordinal. Some rewrites turned the Ordinal into a chain. — repeats 'Some rewrites' as the sentence opener twice in a row, choppy and mechanical
- unclear: The chain ranked 2 nouns in every sentence. — Could mean some rewrites replaced an Ordinal with a comparative-style construction that ranks or orders two nouns against each other instead of stating a numbered position; or "chain" could refer to some other sequential/list structure unrelated to comparison — the exact mechanism of this rewrite pattern is not spelled out.

## 0030


## 0031

- ¶1 natural 3/5
- ¶2 natural 2/5
- ¶3 natural 3/5
- unclear: metrics/load Ban mechanics — which phrase is ambiguous unclear
- unclear: Propositional Loss in 3 cases — no explanation of what was lost
- unclear: If a Copula precedes the phrase — referent of the phrase unclear

## 0032

- ¶1 natural 4/5
  - The obligation is a claim and the possibility is a claim. — template-like repetition of the same predicate for two different subjects
- ¶4 natural 3/5
  - The fences of the decision "0003" stay and the fences of the decision "0010" stay. — repeated noun-plus-verb pattern and the odd metaphor "fences" for constraints read mechanical
- ¶5 natural 4/5
  - A quantified subject takes "must be" and takes "can be". — repeating the verb "takes" instead of coordinating naturally reads stiff
- unclear: the language has 3 modals — Unclear which three modals are meant: only "must" and "can" (with negated forms) are shown in this document, so the third could be an implicit modal like "no" (used with "can be"), or a modal not otherwise named here.
- unclear: the fences of the decision "0003" stay and the fences of the decision "0010" stay — "Fences" is not a defined project term here; most likely it means the restrictions/bans set by those earlier decisions remain in force, but it could also point to some more specific named mechanism this document doesn't define.

## 0033

- ¶1 natural 4/5
  - connectives marked a sequence — quantifier-first construction ("39 connectives") reads like a data tally rather than prose
- ¶3 natural 4/5
  - is a second example — "a second example" is stiffer than the natural "another example"
- ¶4 natural 3/5
  - deferred the ordered steps and deferred the word "then" at the front of a sentence — repeating "deferred" instead of natural coordination reads mechanical
- unclear: 39 connectives marked a sequence — Unclear whether the 39 count is drawn only from the 7 sentences of archetype "A13" identified in this rewrite, or from a broader scan of connectives across all the decision documents, with the 7 A13 sentences being just a subset flagged for this fix.

## 0034

- ¶1 natural 3/5
  - The maintainer required an unambiguous structure with lines. The Enumeration is one structure with lines. Gherkin has a structure with lines — the phrase "structure with lines" is repeated verbatim across three consecutive sentences, reading templated
- ¶3 natural 4/5
  - follow the identical rule — "the identical rule" is stiffer than the natural "the same rule"
- ¶7 natural 4/5
  - 3 tools keep the lines of a Block — restates "keeps the lines of a Block" from the previous sentence almost verbatim, reading templated
- ¶9 natural 3/5
  - the Linter names the home of the word "then" — "home" is an odd metaphor for where a rejected word belongs, unnatural word choice
- unclear: the repair of the paragraphs — Unclear whether this names a specific existing tool, parallel to "lint-file" and "the extractor," that repairs or rewrites paragraph text, or is a more general description of a repair step in the pipeline; the document does not define it elsewhere.

## 0035

- ¶3 natural 4/5
  - The sentence "some guards become floors" is one example. — repeats the "is one example" pattern used two sentences earlier for a different case, reading templated

## 0036

- ¶2 natural 4/5
  - An example shows the meaning but a rule does not show the meaning. — mirrors the previous sentence's "knows/does not know the meaning" pattern too closely, reading templated
- ¶5 natural 4/5
  - ends with the character "." — an unnatural way to refer to a period; a human writer would more naturally say "ends with a period"
- ¶6 natural 3/5
  - the field "member_of" of the term names the group "Rejection" — stacked possessives ("field ... of the term") make the clause dense and hard to parse
- ¶8 natural 4/5
  - Every term shows the group of the term to a reader. — repeats "term" and stacks a possessive ("group of the term"), reading stiffly mechanical
- unclear: The prompt of a repair shows the examples to the writer. — Unclear whether "a repair" names a specific defined tool or process, such as an automated or LLM-driven step that fixes non-parsing sentences and shows a prompt to a human writer, or is used generically for any repair activity; the document does not define "repair" elsewhere.

## model

- ¶3 natural 3/5
  - the Category of a Lemma is the class of the word ... the Category decides the Surface Forms ... the Category of the Lemma "file" is "NOUN" — four sentences in a row open with 'the Category', a mechanical repetition pattern
- ¶4 natural 3/5
  - a Form Tag names the class ... every Surface Form has one Form Tag. the Category of the Lemma decides the Form Tag — repeated sentence-initial and sentence-final 'Form Tag' reads like a template
- ¶6 natural 3/5
  - the Lexicon holds every Surface Form with the Form Tag of the Surface Form. the Lexicon holds every Redirect of the Seed. — two adjacent sentences both open 'the Lexicon holds', mechanical repetition
- ¶9 natural 4/5
  - a Redirect names a replacement for a Rejected Sense of a word — slightly stilted phrasing, a human would more likely say 'a Redirect suggests a replacement word'
- ¶13 natural 3/5
  - if a Ban covers every sense of a word, then the word is a Ban — confusing category shift from 'a rejection is a Ban' to 'the word itself is a Ban'
- ¶15 natural 3/5
  - a Declared Loss is a loss of a translation — circular opening, defines the term using its own root word with no added information
- ¶16 natural 3/5
  - the Linter is a tool of the project. the Linter lints a sentence. the Linter parses the sentence or explains the Rejection. the Linter names the kind of a Rejection — four consecutive sentences open with 'the Linter', mechanical list rhythm
- ¶18 natural 3/5
  - a Sentence Shape is a pattern for a whole sentence. the Grammar defines every Sentence Shape. the First Token of a sentence announces the Sentence Shape. every sentence of the language follows one Sentence Shape. — four sentences in a row hinge on repeating 'Sentence Shape', reads like a checklist
- ¶22 natural 3/5
  - a Discourse Layer is a mechanism. the mechanism finds the noun — 'a mechanism' is a vague filler noun, the definition doesn't say what kind of mechanism
- ¶26 natural 3/5
  - Lexgen is a tool of the project. Lexgen reads the Seed and builds the Lexicon. if 2 Lemmas collide, then Lexgen does not write the Lexicon. Lexgen shows the Collision in an error. — four consecutive sentences open with 'Lexgen', mechanical repetition
- ¶27 natural 3/5
  - Triage is a tool of the project. Triage reads a corpus of English and measures the Coverage of the Lexicon. Triage counts every word of the corpus — three consecutive sentences open with 'Triage', mechanical repetition
- ¶28 natural 3/5
  - WordNet is a database of English. WordNet holds every sense of a word. the repository stores WordNet — repeated sentence-initial 'WordNet' three times in a row, mechanical
- ¶29 natural 3/5
  - a machine optimizes the rarity of the words — unclear, jargon-like justification with no explanation of what 'optimizing rarity' means
- ¶38 natural 4/5
  - a Rejected Sense is a sense of a word — mildly circular opening, restates the term's own head noun
- ¶41 natural 3/5
  - the string "- " opens the line. an item is one Noun Phrase. the last Noun Phrase of the statement names a set. the items name the members of the set. — a run of very short flat declaratives reads like a spec list rather than written prose
- ¶43 natural 3/5
  - the word "is" is a Copula and the word "are" is a Copula — parallel repetition of 'is a Copula' in one sentence reads mechanical rather than how a person would phrase a pair of examples
- ¶50 natural 3/5
  - the 2 Conjuncts have one kind, so a Noun Phrase cannot join a Noun Phrase — refers to 'the 2 Conjuncts' before the definition has established that a Coordination has 2 Conjuncts, which only appears in the next sentence
- ¶56 natural 3/5
  - a Negation uses the Auxiliary and a Prohibition uses the Auxiliary. a plain statement does not use the Auxiliary. — 'the Auxiliary' repeated as the object of three consecutive clauses, mechanical rhythm
- ¶59 natural 3/5
  - the Cognitive Load outranks the familiarity — 'the familiarity' is an unexplained referent and 'outranks' is an odd verb choice for this comparison
- ¶60 natural 3/5
  - the reader holds the first word in the memory of the reader — awkward doubled possessive, 'the reader ... of the reader', a human would just say 'in memory'
- ¶63 natural 3/5
  - measures the prior text — unclear whether it measures the amount of prior text needed or the text itself, ambiguous phrasing
- ¶64 natural 3/5
  - a Language Model is a program. a Language Model reads a text and writes a text. a Language Model reads a sentence in the order of the words. a Language Model holds a small memory — four consecutive sentences open with 'a Language Model', mechanical repetition
- ¶67 natural 3/5
  - the Noun Phrase is a plural noun — conflates the phrase with the noun itself, a phrase cannot literally be a noun
  - so the claim tolerates an exception — 'the claim' has no clear referent established earlier in the definition
- ¶77 natural 3/5
  - the language bans every Anaphoric Pronoun, so the writer repeats the Name — tangential justification about pronoun policy tacked onto the end, doesn't describe what a Name is
- ¶80 natural 3/5
  - Markdown turns the string "- " into a bullet — mechanical, code-like phrasing for describing a text format's rendering behavior
- ¶85 natural 3/5
  - the keyword "Given" marks a precondition. the keyword "When" marks an event. the keyword "Then" marks a result. the keyword "And" marks a continuation of the prior line. — four parallel short sentences in a row read like a table rather than written prose
- ¶87 natural 2/5
  - a tool keeps the lines of a Block. no tool splits a Block. — vague, unclear which tool and what 'keeping' versus 'splitting' lines concretely means
  - the language has 2 kinds of the Blocks — awkward pluralization 'the Blocks' with definite article, unnatural phrasing
- unclear: a machine optimizes the rarity of the words — could mean a machine would select or weight words by how statistically rare they are (favoring rare words over useful ones), or could mean a machine would tune/adjust the corpus's rarity distribution toward some frequency target; the direction and goal of 'optimizes' is not settled by the surrounding text.
- term coverage: imaginable 3/5 — no examples are given, so the abstract notion of 'a share of a corpus' is not anchored to a concrete instance
- term anaphoric: imaginable 3/5 — no examples given, reader must infer instances purely from the one-sentence definition
- term indexical: imaginable 3/5 — no examples given, reader must infer instances purely from the one-sentence definition
- term embedding depth: imaginable 3/5 — no examples are listed; the clause-in-a-clause counting rule stays abstract without a concrete instance to check against
- term block: imaginable 3/5 — examples only name the two subtypes rather than showing a concrete block, and the 'tool keeps the lines' language stays vague

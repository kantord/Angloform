//! Lexicon-authoring-format design (docs/lexicon-authoring-format-2026-09-07.md):
//! dedicated, tooling-only entry points that check "is this an adequate
//! *definition*," not just "is this valid Angloform." Distinct from
//! `pub Sentence` — never used by ordinary prose. `just lint`/`SentenceParser`
//! will correctly refuse these as standalone sentences (a bare NP or a
//! subjectless verb phrase isn't ordinary prose) — that's expected, not a
//! bug; they're only valid through these dedicated entry points.

use grammar::{parse_definition, DefKind, Lexicon};

fn repo(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

fn lexicon() -> Lexicon {
    Lexicon::load(&repo("lexicon.tsv")).unwrap()
}

fn ok(kind: DefKind, lemma: &str, s: &str) {
    let lex = lexicon();
    parse_definition(&lex, kind, lemma, s).unwrap_or_else(|e| panic!("expected to parse: {s} — {e}"));
}

fn rejects(kind: DefKind, lemma: &str, s: &str) {
    let lex = lexicon();
    assert!(parse_definition(&lex, kind, lemma, s).is_err(), "expected a rejection: {s}");
}

#[test]
fn noun_def_is_a_bare_np_no_copula_wrapper() {
    // REVISED (2026-09-08): originally required "Subj is a Genus, which
    // …" — a full copula clause. Since NounDef is a standalone `pub`
    // symbol (not entangled with Sentence's shared machinery, unlike
    // ADR 0059's failed embedded attempt at the same shape), dropping
    // the copula wrapper and requiring just a bare NP + mandatory
    // differentia is conflict-free and strictly shorter — real
    // dictionary style, no restated subject.
    ok(DefKind::Noun, "society", "a group, which has a history");
}

#[test]
fn noun_def_rejects_a_bare_genus_with_no_differentia() {
    // The exact real failure this whole feature exists to catch: a
    // technically-fine noun phrase that isn't a real definition.
    rejects(DefKind::Noun, "society", "a group");
}

#[test]
fn verb_trans_def_is_a_bare_verb_phrase_no_subject() {
    // Real dictionary style ("build a thing from parts"), reusing the
    // exact nonterminal Imperative already uses (VBaseP, ADR 0019) — not
    // new grammar.
    ok(DefKind::VerbTrans, "produce", "build a thing from parts");
}

#[test]
fn verb_trans_def_structurally_disallows_a_vague_filler_subject() {
    // "a tool does…" is not merely discouraged, it's ungrammatical here —
    // VBaseP starts on a bare verb; "a" can never be first. REVISION
    // HISTORY (2026-09-08): originally required a mandatory Causal
    // ("<clause>, so <clause>") purpose/result tail — found gameable and
    // too strict (see git history / design doc) before landing on the
    // subjectless form, which structurally disallows the vague-filler-
    // subject pattern by construction, matching this project's own
    // "just don't allow the construction, if there's a way to" principle
    // (docs/garden-paths-2026-09-06.md).
    rejects(DefKind::VerbTrans, "produce", "a tool builds a thing from parts");
    ok(DefKind::VerbTrans, "produce", "build a thing from parts");
}

#[test]
fn verb_trans_def_cannot_drop_the_object_entirely() {
    // Not a definition-grammar choice — a hard fact about the language:
    // transitivity is lexically fixed per category (one category per
    // word), so VERB_TRANS always mandates an object, definitions
    // included. "build from parts" (no object at all) is ungrammatical
    // for any VERB_TRANS word, full stop.
    rejects(DefKind::VerbTrans, "produce", "build from parts");
}

#[test]
fn adj_def_is_a_bare_adjective_phrase_no_subject_no_copula() {
    // REVISED (2026-09-08): originally a full predicative clause ("a rare
    // word is not common"), which forced picking an arbitrary (and often
    // wrong-domain) subject noun just to have somewhere to hang the
    // copula — the same over-narrowing problem the verbs had with "a
    // tool"/"a file". Matches the same simplification already applied to
    // NounDef and VerbTransDef: the adjectival content itself, nothing
    // else. Covers plain ("small") and the negated-contrast shape the
    // `antonym` field is meant to feed ("not common").
    ok(DefKind::Adj, "young", "small"); // grammar-shape check only, not a real definition
    ok(DefKind::Adj, "rare", "not common");
}

#[test]
fn no_trailing_period_is_allowed() {
    // REVISED (2026-09-08): previously tolerated either way. A definition
    // is a fragment, not a sentence — banned outright now, structurally,
    // not left to author discipline.
    rejects(DefKind::VerbTrans, "produce", "build a thing from parts.");
    rejects(DefKind::Noun, "society", "a group, which has a history.");
    rejects(DefKind::Adj, "rare", "not common.");
    ok(DefKind::VerbTrans, "produce", "build a thing from parts");
}

#[test]
fn a_definition_cannot_use_its_own_headword() {
    // Real mistakes caught in this design's own trial: "sit in a
    // position" (defining "sit") and "fail without a reason" (defining
    // "fail") both used the headword as their own definition's main
    // verb — directly circular, not hypothetical. Checked against the
    // word actually being defined (the `lemma` argument), not against
    // the text alone, so a different word using the same surface form
    // elsewhere isn't falsely flagged.
    rejects(DefKind::VerbIntrans, "sit", "sit in a position");
    rejects(DefKind::VerbIntrans, "fail", "fail without a reason");
    ok(DefKind::VerbIntrans, "sit", "stay in a position");
    ok(DefKind::VerbIntrans, "fail", "end without a result");
}

#[test]
fn a_definition_cannot_reference_a_domain_model_term() {
    // Real trial mistake: "a file is a Name, which holds the data" used
    // Angloform's own internal meta-vocabulary (the "Name" concept —
    // ADR 0018's unquoted-identifier term) to define an ordinary content
    // word. A category error, not a style nitpick: "Name" describes
    // Angloform's own grammar, it isn't a general word for "identifier".
    rejects(DefKind::Noun, "file", "a Name, which holds the data");
    ok(DefKind::Noun, "file", "a thing, which holds the data");
}

/// The full 14-word trial set (docs/lexicon-authoring-format-
/// 2026-09-07.md) in its final, shortest, structurally-enforced form: no
/// restated subject on the verbs, no copula wrapper on the nouns, no
/// subject/copula on the adjectives either, no trailing period anywhere,
/// no headword self-reference, no domain-model-jargon leakage (file/copy
/// both rewritten for real, not just re-punctuated).
#[test]
fn the_full_14_word_trial_set() {
    ok(DefKind::VerbTrans, "produce", "make a thing");
    ok(DefKind::VerbTrans, "hold", "keep a thing in a position");
    ok(DefKind::Noun, "copy", "a file, which matches a file");
    ok(DefKind::VerbTrans, "duplicate", "make a second copy of a thing");
    ok(DefKind::Noun, "society", "a group of people, which has a tradition");
    ok(DefKind::Noun, "file", "a thing, which holds items");
    ok(DefKind::Adj, "rare", "not common");
    ok(DefKind::Noun, "maintainer", "a person, who decides things in a project");
    ok(DefKind::Noun, "report", "a file, which names a result");
    ok(DefKind::VerbTrans, "delete", "remove a thing");
    ok(DefKind::VerbTrans, "read", "get the meaning from a document");
    ok(DefKind::VerbIntrans, "sit", "stay in a position");
    ok(DefKind::VerbIntrans, "fail", "end without a result");
    ok(DefKind::Adj, "small", "not big");
}

/// 10 more words (2026-09-08), chosen to stress specific edges the first
/// 14 didn't reach: a 2-argument verb via PPv ("compare"), a synonym
/// pair deliberately left distinct by differentia ("build" vs.
/// "produce" — contrast with delete/remove, which weren't), an ADR-
/// backed antonym substitution ("different" via ADR 0023's "same" ban),
/// and one word ("exist") kept in as an honest miss — see the design
/// doc for why it's weak, not silently improved.
#[test]
fn ten_more_words() {
    ok(DefKind::VerbTrans, "write", "make a text");
    ok(DefKind::VerbTrans, "compare", "find the difference between 2 things");
    ok(DefKind::VerbTrans, "choose", "take a thing from things");
    ok(DefKind::VerbTrans, "build", "merge parts into a thing");
    ok(DefKind::VerbTrans, "avoid", "prevent a thing");
    ok(DefKind::VerbIntrans, "exist", "occur in the world");
    ok(DefKind::VerbIntrans, "arrive", "come to a place");
    ok(DefKind::Noun, "agent", "a thing, which builds a thing or removes a thing");
    ok(DefKind::Noun, "output", "a thing, which comes from a process");
    ok(DefKind::Adj, "different", "not identical");
}

/// 10 more words (2026-09-08, round 2), reaching further edges: a
/// copular relative predicate ("error", via `CopPredn<LCopSg>` inside
/// `PredRel` — untested until now), and another near-synonym cluster
/// (create/produce/build/make) resolved by differentia instead of
/// collapsing, unlike delete/remove.
#[test]
fn ten_more_words_round_2() {
    ok(DefKind::Noun, "error", "a thing, which is wrong");
    ok(DefKind::Noun, "tool", "a thing, which helps a person");
    ok(DefKind::Noun, "word", "the smallest part of a document, which has a meaning");
    ok(DefKind::Noun, "team", "a group of people, which has a goal");
    ok(DefKind::VerbTrans, "create", "make a new thing");
    ok(DefKind::VerbTrans, "cause", "give a reason for a thing");
    ok(DefKind::VerbTrans, "describe", "give a meaning for a thing");
    ok(DefKind::VerbIntrans, "depend", "need a thing");
    ok(DefKind::VerbIntrans, "belong", "come from a community or stay in a place");
    ok(DefKind::Adj, "empty", "not full");
}

/// 25 more words (2026-09-08), a larger batch drawing entirely on
/// already-checked vocabulary and already-validated shapes (genus `of`
/// + bare plural, `Tailn`-free single predicates, antonym-via-negation
/// for adjectives). All 25 parsed on the first attempt — see the
/// design doc for the one pair flagged as a weak, mutually-referencing
/// compromise (`stay`/`wait`), the same class of issue as delete/remove.
#[test]
fn twenty_five_more_words() {
    ok(DefKind::Noun, "event", "a thing, which occurs at a time");
    ok(DefKind::Noun, "feature", "a part of a system, which gives an ability");
    ok(DefKind::Noun, "memory", "a part of a system, which holds items");
    ok(DefKind::Noun, "message", "a text, which comes from a person");
    ok(DefKind::Noun, "model", "a thing, which describes a system");
    ok(DefKind::Noun, "number", "a thing, which names a value");
    ok(DefKind::Noun, "system", "a group of parts, which helps a person");
    ok(DefKind::Noun, "sentence", "a sequence of words, which has a meaning");
    ok(DefKind::Noun, "test", "a tool, which checks a thing");
    ok(DefKind::VerbTrans, "send", "carry a thing to a person");
    ok(DefKind::VerbTrans, "receive", "take a thing from a person");
    ok(DefKind::VerbTrans, "check", "find an error in a thing");
    ok(DefKind::VerbTrans, "open", "start a thing");
    ok(DefKind::VerbTrans, "close", "stop a thing");
    ok(DefKind::VerbTrans, "accept", "allow a thing");
    ok(DefKind::VerbTrans, "reject", "ban a thing");
    ok(DefKind::VerbTrans, "fix", "remove an error from a thing");
    ok(DefKind::VerbTrans, "update", "get a new version of a thing");
    ok(DefKind::VerbIntrans, "stay", "exist in a place");
    ok(DefKind::VerbIntrans, "wait", "stay until a time");
    ok(DefKind::VerbIntrans, "win", "beat every person");
    ok(DefKind::VerbIntrans, "conflict", "come against a thing");
    ok(DefKind::Adj, "sharp", "not blunt");
    ok(DefKind::Adj, "clear", "not ambiguous");
    ok(DefKind::Adj, "false", "not correct");
}

/// Negation combined with `Tailn`'s "or" inside one NounDef differentia
/// (2026-09-08, from "noise"): `PredRelCore` already lists `NegVPn`
/// alongside the plain verb branches, so a negated second predicate
/// ("or does not have a meaning") was reachable without any grammar
/// change — just never exercised in combination with `Tailn` before.
#[test]
fn tailn_predicate_may_itself_be_negated() {
    ok(
        DefKind::Noun,
        "noise",
        "a thing, which confuses a person or does not have a meaning",
    );
}


/// "dimension" was added to the real lexicon (seed/seed.json, then
/// `cargo run -p lexgen`) on 2026-09-08 specifically so "table" could be
/// defined without a redundant/vague genus — a real, if small, addition
/// to Angloform's vocabulary, not just this trial's own scratch space.
#[test]
fn table_uses_the_newly_added_dimension_word() {
    ok(DefKind::Noun, "table", "a thing, which organizes the information in 2 dimensions");
}

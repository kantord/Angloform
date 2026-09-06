//! Property tests over *generated* angloform sentences, built from real
//! lexicon words rather than the fixed corpus. corpus.rs and the
//! `tier2_is_a_superset_of_tier1` unit test in diagnosis.rs check a curated
//! list; this generalizes both invariants — "every sentence this shape
//! produces parses" and "tier-2 never rejects what tier-1 accepts" — over a
//! much larger generated sample, with automatic shrinking to a minimal
//! counterexample on failure. This is the kind of test that would have
//! caught the ADR 0037 CoordClause/Causal LALR conflict directly, rather
//! than needing it found by hand while rewriting an ADR.

use diagnose::{diagnose, Diagnosis, Tier2};
use grammar::{parse, Lexicon};
use proptest::prelude::*;

fn repo(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

/// Single lowercase-word surface forms carrying `tag`, read from the live
/// lexicon — never hardcoded, so the generator never drifts from what the
/// grammar actually accepts.
fn words(tag: &str) -> Vec<String> {
    let text = std::fs::read_to_string(repo("lexicon.tsv")).unwrap();
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let [surface, kind, t, _] = f[..] else { return None };
            (kind == "form" && t == tag && surface.chars().all(|c| c.is_ascii_lowercase()))
                .then(|| surface.to_string())
        })
        .collect()
}

fn np() -> impl Strategy<Value = String> {
    prop::sample::select(words("NOUN_SG")).prop_map(|n| format!("the {n}"))
}

/// A predicate alone (no subject): a transitive verb with a fresh object
/// noun phrase, or a bare intransitive verb.
fn pred() -> impl Strategy<Value = String> {
    prop_oneof![
        (prop::sample::select(words("VERB_TRANS_3SG")), np())
            .prop_map(|(v, o)| format!("{v} {o}")),
        prop::sample::select(words("VERB_INTRANS_3SG")),
    ]
}

fn conj() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec!["and", "or", "but"])
}

/// A minimal, deliberately narrow slice of the grammar (ADR 0004/0021/0037's
/// Coordination, on top of a plain transitive-or-intransitive statement):
/// a plain clause, a shared-subject predicate Coordination (no comma), or a
/// full-clause Coordination (comma mandatory, ADR 0037). Not a generator
/// for the whole language — see docs/ideas.md, "Property-based testing with
/// proptest" for the wider survey and what a full generator would need.
fn statement() -> impl Strategy<Value = String> {
    prop_oneof![
        (np(), pred()).prop_map(|(s, p)| format!("{s} {p}")),
        (np(), pred(), conj(), pred())
            .prop_map(|(s, p1, c, p2)| format!("{s} {p1} {c} {p2}")),
        (np(), pred(), conj(), np(), pred())
            .prop_map(|(s1, p1, c, s2, p2)| format!("{s1} {p1}, {c} {s2} {p2}")),
    ]
}

proptest! {
    #[test]
    fn generated_statements_parse(s in statement()) {
        let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
        prop_assert!(parse(&lexicon, &s).is_ok(), "generated sentence failed to parse: {}", s);
    }

    #[test]
    fn tier2_accepts_every_generated_statement(s in statement()) {
        let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
        let d = diagnose(&lexicon, &s);
        // ADR 0048: same-verb-lemma coordination is a deliberate exception —
        // Tier-1 (grammar) still accepts it, diagnose() correctly rejects it
        // in favor of the colon-list construction. `pred()` is sampled
        // twice independently, so the generator can and does land on the
        // same verb for both coordinated predicates; that's the expected
        // shape of this one known rejection, not a regression.
        let ok = match &d {
            Diagnosis::Clean(_) => true,
            Diagnosis::Style(findings) => findings.iter().any(|f| f.contains("repeats across the coordination")),
            _ => false,
        };
        prop_assert!(ok, "diagnose() rejected: {} -> {:?}", s, d);
        let toks: Vec<_> = lexicon.tokenize(&s).unwrap().into_iter().map(|(_, t)| t).collect();
        prop_assert!(Tier2::new(&toks).count() >= 1, "tier-2 rejects a tier-1 sentence: {}", s);
    }
}

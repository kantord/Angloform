//! Property tests over arbitrary text, not just the curated corpus: every
//! public entry point here is fed real user/agent/LLM input (a diagnose()
//! call on an LLM repair proposal, a document run through units()), so
//! crash-freedom on *any* string — not just well-formed angloform — is a
//! real requirement, not a nicety. These complement corpus.rs's exact-output
//! snapshots with much broader, generated-input coverage; proptest's
//! shrinking turns any failure straight into a minimal repro.

use grammar::{is_enumeration, is_step_block, units, Lexicon};

fn repo(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

proptest::proptest! {
    #[test]
    fn tokenize_never_panics(s in "\\PC{0,200}") {
        let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
        let _ = lexicon.tokenize(&s);
    }

    #[test]
    fn units_never_panics(s in "\\PC{0,500}") {
        let _ = units(&s);
    }

    #[test]
    fn units_is_idempotent_on_its_own_output(s in "\\PC{0,500}") {
        // splitting an already-split unit apart again should not further
        // fragment it (each returned unit is a maximal one)
        let first = units(&s);
        for u in &first {
            let again = units(u);
            proptest::prop_assert_eq!(again.len(), 1, "unit re-split: {:?} -> {:?}", u, again);
        }
    }

    #[test]
    fn classifiers_never_panic(s in "\\PC{0,300}") {
        let _ = is_enumeration(&s);
        let _ = is_step_block(&s);
    }

    #[test]
    fn enumeration_and_step_block_are_mutually_exclusive(s in "\\PC{0,300}") {
        proptest::prop_assert!(!(is_enumeration(&s) && is_step_block(&s)));
    }

    #[test]
    fn parse_never_panics(s in "\\PC{0,200}") {
        let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
        let _ = grammar::parse(&lexicon, &s);
        let _ = grammar::parse_text(&lexicon, &s);
    }
}

//! Grammatical case is a pure derived view of the parse tree (position
//! already determines role, as in English): no new grammar, just a walk.

use grammar::{cases, parse_text, Case, Lexicon, Tree};

fn repo(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

fn lexicon() -> Lexicon {
    Lexicon::load(&repo("lexicon.tsv")).unwrap()
}

/// pos -> word, read straight off the tree's own leaves.
fn word_at(tree: &Tree, want: usize, out: &mut Option<String>) {
    match tree {
        Tree::Leaf { pos, word } if *pos == want => *out = Some(word.clone()),
        Tree::Node { children, .. } => {
            for c in children {
                word_at(c, want, out);
            }
        }
        _ => {}
    }
}

/// Word-by-word cases for `sentence`, as `(word, Case)` pairs — only the
/// words that got a case at all.
fn tagged(lexicon: &Lexicon, sentence: &str) -> Vec<(String, Case)> {
    let tree = parse_text(lexicon, sentence).unwrap();
    cases(&tree)
        .into_iter()
        .map(|(pos, c)| {
            let mut w = None;
            word_at(&tree, pos, &mut w);
            (w.unwrap_or_default(), c)
        })
        .collect()
}

#[test]
fn subject_and_object() {
    let lex = lexicon();
    let got = tagged(&lex, "the agent saves the report.");
    assert_eq!(
        got,
        vec![
            ("agent".to_string(), Case::Nominative),
            ("report".to_string(), Case::Accusative),
        ]
    );
}

#[test]
fn of_pp_is_genitive() {
    let lex = lexicon();
    let got = tagged(&lex, "the author of the report reads the file.");
    assert_eq!(
        got,
        vec![
            ("author".to_string(), Case::Nominative),
            ("report".to_string(), Case::Genitive),
            ("file".to_string(), Case::Accusative),
        ]
    );
}

#[test]
fn copula_complement() {
    let lex = lexicon();
    let got = tagged(&lex, "the file is old.");
    assert_eq!(
        got,
        vec![
            ("file".to_string(), Case::Nominative),
            ("old".to_string(), Case::Complement),
        ]
    );
}

#[test]
fn pp_object_is_oblique() {
    let lex = lexicon();
    let got = tagged(&lex, "the user opens the file with the tool.");
    assert_eq!(
        got,
        vec![
            ("user".to_string(), Case::Nominative),
            ("file".to_string(), Case::Accusative),
            ("tool".to_string(), Case::Oblique),
        ]
    );
}

#[test]
fn negated_verb_still_marks_object() {
    let lex = lexicon();
    let got = tagged(&lex, "the agent does not save the report.");
    assert_eq!(
        got,
        vec![
            ("agent".to_string(), Case::Nominative),
            ("report".to_string(), Case::Accusative),
        ]
    );
}

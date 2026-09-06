//! Self-linting: every example sentence the project shows a writer — in the
//! skill, in linter advice, in ban advice — must itself be valid angloform.
//! Feeding the model invalid examples and expecting valid output is the
//! failure this test exists to crash on.

use diagnose::{diagnose, Diagnosis};
use grammar::{Lexicon, Tok};

fn repo(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

/// Quoted or backticked spans that are shown as examples: ≥ 3 words, no
/// template placeholder (`<noun>`), no ellipsis. A span preceded by "not "
/// is a counter-example and is skipped; a span with no verb-like token is
/// a phrase, not a sentence, and is skipped too.
fn example_sentences(lexicon: &Lexicon, text: &str, open: char, close: char) -> Vec<String> {
    // scan honoring backslash escapes, so advice like "the pronoun \"i\"" yields
    // the example `the pronoun "i"` with its inner quotes intact
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != open {
            i += 1;
            continue;
        }
        let before: String = chars[..i].iter().collect();
        let mut j = i + 1;
        let mut span = String::new();
        while j < chars.len() {
            if chars[j] == '\\' && j + 1 < chars.len() {
                span.push(chars[j + 1]);
                j += 2;
                continue;
            }
            if chars[j] == close {
                break;
            }
            span.push(chars[j]);
            j += 1;
        }
        if j >= chars.len() {
            break;
        }
        let rest: String = chars[j + 1..].iter().collect();
        i = j + 1;
        let span = span.trim().trim_end_matches('.').trim().to_string();
        let words = span.split_whitespace().count();
        let template = span.contains('<') || span.contains('…') || span.contains("...");
        let fragment = span.starts_with(|c: char| !c.is_alphanumeric() && c != '"');
        let counter_example = before.trim_end().ends_with("not");
        // `"span" — advice` echoes the writer's own words, not an example
        let echo = rest.trim_start().starts_with('—');
        if words >= 3 && !template && !fragment && !counter_example && !echo && has_verb(lexicon, &span) {
            out.push(span);
        }
    }
    out
}

/// Any finite verb, copula, modal, or do-support token — or a sentence-
/// initial base verb (imperative). Unlexable spans count as sentences so
/// that a bad example is reported, not skipped.
fn has_verb(lexicon: &Lexicon, span: &str) -> bool {
    let Ok(toks) = lexicon.tokenize(span) else { return true };
    toks.iter().enumerate().any(|(i, (_, t))| {
        matches!(
            t,
            Tok::Vt3(_) | Tok::Vi3(_) | Tok::VtEd(_) | Tok::ViEd(_) | Tok::CopSg(_) | Tok::CopPl(_)
                | Tok::CopSgPast(_) | Tok::CopPlPast(_) | Tok::ModalMust(_) | Tok::ModalCan(_)
                | Tok::ModalCannot(_) | Tok::Do3(_) | Tok::DoPast(_)
        ) || (matches!(t, Tok::VtBase(_) | Tok::ViBase(_) | Tok::DoBase(_)) && (i == 0 || matches!(toks[i - 1].1, Tok::Pron1(_) | Tok::Pron2(_) | Tok::NounPl(_))))
    })
}

fn assert_all_parse(lexicon: &Lexicon, examples: &[String], source: &str) {
    let mut bad = Vec::new();
    for s in examples {
        // examples may hold several sentences: "A. B"
        for seg in s.split(". ").map(|x| x.trim().trim_end_matches('.').trim()).filter(|x| x.split_whitespace().count() >= 3) {
            if !matches!(diagnose(lexicon, seg), Diagnosis::Clean(_)) {
                bad.push(format!("{seg}  ← {}", match diagnose(lexicon, seg) {
                    Diagnosis::Word(m) => m,
                    Diagnosis::Style(f) | Diagnosis::Ambiguous { findings: f, .. } => f.join("; "),
                    _ => "unrecognizable".to_string(),
                }));
            }
        }
    }
    assert!(bad.is_empty(), "{source}: example sentences that do not parse:\n  {}", bad.join("\n  "));
}

/// Sentence-shape examples in the skill (backticked, in the "Sentence
/// shapes" section) are what the model is taught — they must parse.
#[test]
fn skill_examples_parse() {
    let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
    let skill = std::fs::read_to_string(repo("skills/angloform/SKILL.md")).unwrap();
    let section = skill
        .split("## Sentence shapes")
        .nth(1)
        .and_then(|s| s.split("## Hard bans").next())
        .expect("skill sections");
    let examples = example_sentences(&lexicon, section, '`', '`');
    assert!(examples.len() >= 20, "expected many skill examples, found {}", examples.len());
    assert_all_parse(&lexicon, &examples, "skills/angloform/SKILL.md");
}

/// Every definition in the domain model is angloform (ADR 0027).
#[test]
fn domain_definitions_parse() {
    let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
    let text = std::fs::read_to_string(repo("domain/model.json")).unwrap();
    let model: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut bad = Vec::new();
    for e in model.as_array().unwrap() {
        let lemma = e["lemma"].as_str().unwrap_or("?");
        // examples that are sentences or blocks must parse too (ADR 0036)
        for x in e["examples"].as_array().into_iter().flatten().filter_map(|x| x.as_str()) {
            // convention: an example with a final period is an angloform sentence;
            // one without is a member or a shape (may be a banned construction)
            let is_block = x.contains('\n');
            if is_block || x.ends_with('.') {
                let ok = if is_block { grammar::parse_text(&lexicon, x).is_ok() } else { matches!(diagnose(&lexicon, x.trim_end_matches('.')), Diagnosis::Clean(_)) };
                if !ok {
                    bad.push(format!("{lemma} (example): {}", x.replace('\n', " ⏎ ")));
                }
            }
        }
        let def = e["definition"].as_str().unwrap_or("");
        for seg in def.split(". ").map(|s| s.trim().trim_end_matches('.').trim()).filter(|s| !s.is_empty()) {
            match diagnose(&lexicon, seg) {
                Diagnosis::Clean(_) => {}
                d => bad.push(format!("{lemma}: {seg}  ← {}", match d {
                    Diagnosis::Word(m) => m,
                    Diagnosis::Style(f) | Diagnosis::Ambiguous { findings: f, .. } => f.join("; "),
                    _ => "unrecognizable".to_string(),
                })),
            }
        }
    }
    assert!(bad.is_empty(), "domain/model.json: definitions that do not parse:\n  {}", bad.join("\n  "));
}

/// Ban advice (lexicon.tsv ban rows) quotes fixes — they must parse.
#[test]
fn ban_advice_examples_parse() {
    let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
    let lex = std::fs::read_to_string(repo("lexicon.tsv")).unwrap();
    let mut examples = Vec::new();
    for line in lex.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if let [_, "ban", _, advice] = f[..] {
            examples.extend(example_sentences(&lexicon, advice, '"', '"'));
        }
    }
    assert_all_parse(&lexicon, &examples, "lexicon.tsv ban advice");
}

/// linter findings quote fixes. Drive every finding through the rejected
/// test sentences and check each quoted example.
#[test]
fn linter_advice_examples_parse() {
    let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
    let probes = [
        "if the test fails, the agent retries the request",
        "the agent retries the request if the test fails",
        "every agent does not retry the request",
        "no agent must check the input",
        "no agent cannot open a session",
        "the agent retries no request",
        "some agent retries the request",
        "a files are big",
        "one files are big",
        "the agent deleted 3 file",
        "the agent has deleted the file",
        "the agent is opening the file",
        "the file stored in the database fails",
        "the agent stopped",
        "the test fails so the agent retries the request",
        "because the test fails, the agent retries the request",
        "so the agent retries the request",
        "the project does not plan a discourse mechanism",
        "the word i is a Pronoun",
        "my is a Pronoun",
        "the Pronouns are not in the Lexicon",
        "the mechanism stores a word and a message",
        "the cost is big",
        "the agent files the report",
        "the agent stores the report in the database with the tool",
        "it fails",
        "the agent deleted 0 files",
        "the agent deleted 1 file",
        "the copies are the same",
        "the agent deleted three files",
        "Discard the file",
        "the Lexicon does not contain the banned Pronouns",
        "Pronouns for the speaker are indexical",
        "the frequencies are of Pronouns",
        "the speaker or the hearer reads the file",
        "the findings are big",
        "the design prefers clarity",
        "the report shows Pronouns are big",
        "the report shows that the test fails",
        "the i Pronoun is indexical",
        "the language does not have a discourse layer",
        "the Pronouns are \"it\", \"they\", and \"those\"",
        "instructional text uses Indexical Pronouns",
        "the team defers the mechanism to the future",
        "the Pronouns are about 2200 of the unknown tokens",
        "the document describes the finding",
        "resolving the Pronoun requires a Discourse Layer",
        "the language bans the possessive of every Anaphoric Pronoun",
        "the tool Lexgen allows my",
        "the point of the copy of the report fails",
    ];
    let mut examples = Vec::new();
    for p in probes {
        match diagnose(&lexicon, p) {
            Diagnosis::Word(m) => examples.extend(example_sentences(&lexicon, &m, '"', '"')),
            Diagnosis::Style(f) | Diagnosis::Ambiguous { findings: f, .. } => {
                for m in f {
                    examples.extend(example_sentences(&lexicon, &m, '"', '"'));
                }
            }
            Diagnosis::Clean(_) => panic!("probe unexpectedly parses: {p}"),
            Diagnosis::Unknown => panic!("probe gets no advice: {p}"),
        }
    }
    examples.sort();
    examples.dedup();
    assert!(examples.len() >= 5, "expected several advice examples, found {}", examples.len());
    assert_all_parse(&lexicon, &examples, "Linter advice");
}

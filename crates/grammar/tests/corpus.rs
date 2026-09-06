//! Every accept-corpus sentence parses (LR(1) guarantees uniqueness);
//! trees + metrics are snapshotted; sanctioned-structure violations reject.

use grammar::{metrics, parse, parse_text, Lexicon};

fn repo(path: &str) -> String {
    format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn corpus_parses_with_snapshots() {
    let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
    let corpus = std::fs::read_to_string(repo("corpus/accept.txt")).unwrap();
    let mut snap = String::new();
    for line in grammar::units(&corpus) {
        let line = line.as_str();
        let tree = parse_text(&lexicon, line)
            .unwrap_or_else(|e| panic!("corpus sentence failed to parse: {line} — {e}"));
        let m = metrics(&tree);
        snap.push_str(&format!(
            "### {line}\nopen={} maxdep={} depth={} right={:.0}% fronted={}\n",
            m.peak_open_deps,
            m.max_dep_len,
            m.embedding_depth,
            m.right_branching * 100.0,
            m.fronted
        ));
        tree.render(&mut snap, 0);
        snap.push('\n');
    }
    insta::assert_snapshot!("corpus_trees", snap);
}

/// Every .feature file is an angloform document (ADR 0034).
#[test]
fn feature_files_parse() {
    let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
    for entry in std::fs::read_dir(repo("features")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "feature") {
            let text = std::fs::read_to_string(&path).unwrap();
            for unit in grammar::units(&text) {
                parse_text(&lexicon, &unit).unwrap_or_else(|e| panic!("{}: {unit} — {e}", path.display()));
            }
        }
    }
}

#[test]
fn banned_structures_reject() {
    let lexicon = Lexicon::load(&repo("lexicon.tsv")).unwrap();
    let banned = [
        // determiner–number agreement
        "a files are big",
        // two verb-attaching PPs in one clause (ADR 0011 bound)
        "the agent stores the report in the database with the tool",
        // banned third-person pronoun (ADR 0002)
        "it fails",
        // banned modal (ADR 0009)
        "the agent may fail",
        // reduced relative shape cannot arise (ADR 0010's precondition)
        "the file stored in the database fails",
        // transitive verb without object
        "the agent stopped",
        // conditional without mandatory then (ADR 0007)
        "if the test fails, the agent retries the request",
        // consequent-first conditional is out (ADR 0007)
        "the agent retries the request if the test fails",
        // -ing forms have no sanctioned structure yet
        "the agent is opening the file",
        // quantified subjects take positive predicates only (ADR 0014)
        "every agent does not retry the request",
        "no agent must check the input",
        "no agent cannot open a session",
        // no is subject-only, first-token signature (ADR 0014)
        "the agent retries no request",
        // auxiliary have is unparseable by construction (ADR 0016)
        "the agent has deleted the file",
        // some: plural + subject-only (ADR 0017)
        "some agent retries the request",
        "the agent checks some requests",
        // digits: 0 and 1 redirect, leading zeros reject, number words
        // are banned, digit + singular / one + plural disagree (ADR 0022)
        "the agent deleted 0 files",
        "the agent deleted 1 file",
        "the agent deleted 03 files",
        "the agent deleted three files",
        "3 agent retries the request",
        "one files are big",
        // same is banned with per-sense advice (ADR 0023)
        "the same file fails",
        // percent needs a digit count and a named set (ADR 0024)
        "43 percent reduce the ambiguity",
        "one percent of the swaps reduce the ambiguity",
        "the percent of the swaps is big",
        // about / ~ only before a digit count (ADR 0025)
        "the agent reads about the file",
        "about one file fails",
        "~ 5 Surface Forms fail",
        // causal: comma mandatory, never sentence-initial, no coordination inside (ADR 0026)
        "the test fails so the agent retries the request",
        "because the test fails, the agent retries the request",
        "so the agent retries the request",
        "the test fails, so the agent retries the request and the queue is empty",
        "the agent retries the request, because the test fails hence the queue is empty",
    ];
    for s in banned {
        assert!(
            parse(&lexicon, s).is_err(),
            "should NOT parse but did: {s}"
        );
    }
    // Enumeration (ADR 0028): singular/unquantified target, count mismatch,
    // clause item, trailing PP after the enumerated NP, coordination tail
    let banned_blocks = [
        "the Linter bans the pronoun:\n- \"it\"",
        "the Linter bans 3 pronouns:\n- \"it\"\n- \"they\"",
        "the Linter bans 2 pronouns:\n- \"it\"\n- the agent reads the file",
        "the Linter bans 2 pronouns in the Lexicon:\n- \"it\"\n- \"they\"",
        "the Linter bans 2 pronouns and reads the file:\n- \"it\"\n- \"they\"",
        "the language allows 2 pronouns:\n- \"I\"\n- \"you",
    ];
    // ordinals, scales, comparatives (ADR 0029, 0030)
    let banned2 = [
        "the 1st file fails",
        "the file is heavier",
        "the file is more heavy than the report",
        "million files fail",
        "the agent deleted 3.5",
        // ADR 0031: the complement phrase is copular only; subject position stays out
        "the metrics for the load exist in the research",
        "the agent stores the report for the user in the database",
        // ADR 0032: be only after a modal, complements only
        "the file be old",
        "the file must be deleted",
        "the agent must be deleting the file",
        "the agent wants to be a tool",
        // ADR 0034: Then only inside a Step Block; a step is one clause
        "Then the queue is empty",
        "Given the test fails\nThen the queue is empty and the agent retries the request",
        "And the test fails\nThen the queue is empty",
        "Scenario:\nGiven the test fails",
        // ADR 0035: become takes the copula's complements only
        "the file becomes deleted",
        "the agent becomes running",
        "the ratio becomes",
    ];
    for s in banned2 {
        assert!(parse(&lexicon, s).is_err(), "should NOT parse but did: {s}");
    }
    let _ = [
        "",
    ];
    for s in banned_blocks {
        assert!(
            parse_text(&lexicon, s).is_err(),
            "block should NOT parse but did: {s}"
        );
    }
    let _unused = [
        "",
    ];
    for s in banned {
        assert!(
            parse(&lexicon, s).is_err(),
            "should NOT parse but did: {s}"
        );
    }
}

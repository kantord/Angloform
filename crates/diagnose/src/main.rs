//! angloform linter CLI: certificate for clean sentences, named red flags
//! for rejected ones (see CONTEXT.md "Rejection").
//!
//! Usage:
//!   cargo run -p diagnose -- "<sentence>" ["<sentence>" …]
//!   cargo run -p diagnose -- --file path/with/one/sentence/per/line

use diagnose::{diagnose, Diagnosis};
use grammar::Lexicon;

fn main() {
    let lexicon = Lexicon::load("lexicon.tsv").expect("lexicon.tsv — run lexgen first");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sentences: Vec<String> = if args.first().map(String::as_str) == Some("--file") {
        std::fs::read_to_string(args.get(1).expect("--file needs a path"))
            .expect("read file")
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect()
    } else {
        args
    };
    if sentences.is_empty() {
        eprintln!("usage: diagnose \"<sentence>\" …  |  diagnose --file <path>");
        std::process::exit(2);
    }

    let mut flagged = 0;
    for s in &sentences {
        match diagnose(&lexicon, s) {
            Diagnosis::Clean(m) => println!(
                "✓ {s}\n  parses uniquely — peak-open {}, max-dep {}, depth {}, \
                 right-branching {:.0}%",
                m.peak_open_deps,
                m.max_dep_len,
                m.embedding_depth,
                m.right_branching * 100.0
            ),
            Diagnosis::Word(msg) => {
                flagged += 1;
                println!("✗ {s}\n  WORD: {msg}");
            }
            Diagnosis::Style(findings) => {
                flagged += 1;
                println!("✗ {s}\n  STYLE:");
                for f in findings {
                    println!("    - {f}");
                }
            }
            Diagnosis::Ambiguous { readings, findings } => {
                flagged += 1;
                println!("✗ {s}\n  AMBIGUOUS — {readings} readings:");
                for f in findings {
                    println!("    - {f}");
                }
            }
            Diagnosis::Unknown => {
                flagged += 1;
                println!("✗ {s}\n  not recognizable as an angloform-like sentence");
            }
        }
    }
    if flagged > 0 {
        std::process::exit(1);
    }
}

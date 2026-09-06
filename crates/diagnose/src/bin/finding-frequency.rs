//! Real-usage frequency of diagnose() outcomes and STYLE finding kinds —
//! the answer to "which pattern_findings/slot_findings checks actually
//! fire in practice, and how often does nothing fire at all", so future
//! antiparsers get built for real gaps, not guesses (docs/ideas.md,
//! "Antiparsers").
//!
//! Two sources, run separately and reported side by side:
//!   - tests/paragraph-cases + tests/agent-cases: real near-miss angloform
//!     — proposal/output text from actual repair attempts. The higher-
//!     relevance source: these are sentences someone (or some model) was
//!     genuinely trying to write as angloform, not just any English.
//!   - data/ud/en_ewt-ud-test.conllu: real English, not curated angloform
//!     at all — kept for comparison, but most of it fails at the WORD
//!     level long before reaching structural analysis (telemetry, not a
//!     coverage target — same caveat `triage` already carries).
//!
//! Usage: cargo run -p diagnose --bin finding-frequency

use diagnose::{diagnose, Diagnosis};
use grammar::Lexicon;
use std::collections::BTreeMap;

const LEXICON_PATH: &str = "lexicon.tsv";
const EWT_CORPUS: &str = "data/ud/en_ewt-ud-test.conllu";
const REPORT_PATH: &str = "docs/finding-frequency-report.md";

/// Collapse a finding message to its stable template: every quoted span
/// becomes `X`, so "\"files\" is a noun ... use \"submit\"" and "\"opens\"
/// is a noun ... use \"begins\"" fall into the same bucket instead of each
/// being its own one-off string.
fn normalize(finding: &str) -> String {
    let mut out = String::new();
    let mut in_quotes = false;
    for c in finding.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                if !in_quotes {
                    out.push('X');
                }
            }
            _ if in_quotes => {}
            _ => out.push(c),
        }
    }
    out
}

#[derive(Default)]
struct Tally {
    outcomes: BTreeMap<&'static str, usize>,
    findings: BTreeMap<String, usize>,
    examples: BTreeMap<String, String>,
    generic_fallback: Vec<String>,
    total: usize,
}

impl Tally {
    fn record(&mut self, sentence: &str, lexicon: &Lexicon) {
        self.total += 1;
        let kind = match diagnose(lexicon, sentence) {
            Diagnosis::Clean(_) => "Clean",
            Diagnosis::Word(_) => "Word",
            Diagnosis::Style(findings) => {
                for f in &findings {
                    let key = normalize(&f);
                    *self.findings.entry(key.clone()).or_default() += 1;
                    self.examples.entry(key).or_insert_with(|| sentence.to_string());
                    if f.contains("restructure into one of the angloform templates") && self.generic_fallback.len() < 30
                    {
                        self.generic_fallback.push(sentence.to_string());
                    }
                }
                "Style"
            }
            Diagnosis::Ambiguous { .. } => "Ambiguous",
            Diagnosis::Unknown => "Unknown",
        };
        *self.outcomes.entry(kind).or_default() += 1;
    }

    fn section(&self, title: &str, source: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("## {title}\n\n{source} — {} sentences.\n\n### Outcomes\n\n", self.total));
        for (kind, n) in &self.outcomes {
            out.push_str(&format!("- {kind}: {n} ({:.1}%)\n", 100.0 * *n as f64 / self.total.max(1) as f64));
        }
        out.push_str(&format!(
            "\n### The generic fallback (\"restructure into one of the angloform templates\")\n\n\
             Fired {} times.\n\n",
            self.generic_fallback.len()
        ));
        for ex in &self.generic_fallback {
            out.push_str(&format!("- {ex}\n"));
        }
        out.push_str("\n### STYLE finding kinds, ranked (quoted words normalized to `X`)\n\n");
        let mut ranked: Vec<(&String, &usize)> = self.findings.iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (key, n) in ranked {
            let example = self.examples.get(key).map(String::as_str).unwrap_or("");
            out.push_str(&format!("- {n} — {key}\n  example: {example}\n"));
        }
        out.push('\n');
        out
    }
}

/// Lines of the shape `<indent>- field: <rest of line>` — a lightweight
/// extractor, not a YAML parser: every case file in these directories
/// keeps `text`/`output` as a single-line flow scalar (verified against
/// the real files), so this is exact for the data as it stands, not an
/// approximation that happens to work.
fn extract_field(dir: &str, field: &str) -> Vec<String> {
    let prefix = format!("- {field}: ");
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "yaml") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix(&prefix) {
                out.push(rest.trim().to_string());
            }
        }
    }
    out
}

/// Split a multi-sentence proposal into individual sentences, matching
/// how the rest of the codebase already does it (selflint.rs's
/// `assert_all_parse`).
fn sentences(text: &str) -> impl Iterator<Item = &str> {
    text.split(". ")
        .map(|s| s.trim().trim_end_matches('.').trim())
        .filter(|s| s.split_whitespace().count() >= 2)
}

fn main() {
    let lexicon = Lexicon::load(LEXICON_PATH).expect("load lexicon.tsv");

    let mut near_miss = Tally::default();
    for text in extract_field("tests/paragraph-cases", "text") {
        for s in sentences(&text) {
            near_miss.record(s, &lexicon);
        }
    }
    for output in extract_field("tests/agent-cases", "output") {
        for s in sentences(&output) {
            near_miss.record(s, &lexicon);
        }
    }

    let mut ewt = Tally::default();
    if let Ok(corpus) = std::fs::read_to_string(EWT_CORPUS) {
        for line in corpus.lines() {
            if let Some(sentence) = line.strip_prefix("# text = ") {
                ewt.record(sentence, &lexicon);
            }
        }
    }

    let mut report = String::from(
        "# Finding frequency — real-usage signal for which antiparsers to build next\n\n\
         Not a coverage target (see docs/STATUS.md, \"EWT triage numbers are telemetry, not \
         targets\"); this measures which *rejection explanations* fire, not how much of \
         English parses.\n\n",
    );
    report.push_str(&near_miss.section(
        "Near-miss angloform (tests/paragraph-cases + tests/agent-cases)",
        "Real repair-attempt proposals/outputs — the higher-relevance source",
    ));
    report.push_str(&ewt.section(
        "Real English, for comparison (data/ud/en_ewt-ud-test.conllu)",
        "Not curated angloform at all — most of it fails at the WORD level first",
    ));

    std::fs::write(REPORT_PATH, &report).expect("write report");
    println!(
        "finding-frequency: near-miss {} sentences ({} Style, generic fallback {}×); \
         EWT {} sentences ({} Style, generic fallback {}×); report in {REPORT_PATH}",
        near_miss.total,
        near_miss.outcomes.get("Style").unwrap_or(&0),
        near_miss.generic_fallback.len(),
        ewt.total,
        ewt.outcomes.get("Style").unwrap_or(&0),
        ewt.generic_fallback.len(),
    );
}

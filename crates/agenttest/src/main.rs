//! agenttest — snapshot-boosted manual review of LLM repair behavior.
//!
//! NOT a cheap replicable CI test: each run costs real API calls and its
//! outputs need human review. Re-running is a **milestone action**. The
//! flow per case (one YAML per case in tests/agent-cases/):
//!
//!   system prompt = skills/angloform/repair-prompt.md + word list from
//!   lexicon.tsv (chosen by A/B test, 2026-09-05: see docs/prompt-ab.md)
//!   round 1: "this sentence was rejected: … linter says: … reply with only
//!            the corrected angloform sentence"
//!   invalid reply → feed the new diagnosis back, up to 3 rounds
//!   N independent trials per case (temperature > 0) → first_try_rate
//!
//! Every attempt — including intermediate failures — is logged into the
//! case file for later mining. The `verdict` field (ideal | needs-fix |
//! unreviewed) is the human's judgment of the current `snapshot`; it resets
//! to `unreviewed` whenever a run changes the snapshot.
//!
//! Usage: OPENROUTER_API_KEY=… cargo run -p agenttest
//!   env: ANGLOFORM_TEST_MODEL  (default: deepseek/deepseek-v4-flash)
//!        ANGLOFORM_TEST_TRIALS (default: 3)
//!        ANGLOFORM_TEST_TEMP   (default: 0.7)

mod paragraphs;

use diagnose::{diagnose, Diagnosis};
use grammar::Lexicon;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const CASES_DIR: &str = "tests/agent-cases";
const SKILL_PATH: &str = "skills/angloform/repair-prompt.md";
pub(crate) const LEXICON_PATH: &str = "lexicon.tsv";
pub(crate) const MAX_ROUNDS: usize = 3;

#[derive(Serialize, Deserialize, Default)]
struct Case {
    kind: String,
    input: String,
    /// Optional neighbouring sentences shown to the model as context; only
    /// `input` is linted and graded (a realistic window, per review).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    context_before: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    context_after: String,
    #[serde(default = "unreviewed")]
    verdict: String,
    #[serde(default)]
    snapshot: String,
    #[serde(default)]
    unique_outputs: Vec<UniqueOutput>,
    #[serde(default)]
    runs: Vec<Run>,
}

fn unreviewed() -> String {
    "unreviewed".to_string()
}

#[derive(Serialize, Deserialize)]
struct UniqueOutput {
    output: String,
    valid: bool,
    count: usize,
}

#[derive(Serialize, Deserialize)]
struct Run {
    at_unix: u64,
    model: String,
    temperature: f64,
    trials: usize,
    first_try_valid: usize,
    trial_detail: Vec<Trial>,
}

#[derive(Serialize, Deserialize)]
struct Trial {
    rounds: usize,
    final_valid: bool,
    attempts: Vec<Attempt>,
}

#[derive(Serialize, Deserialize)]
struct Attempt {
    output: String,
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnosis: Option<String>,
}

fn main() {
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    let api_key = match std::env::var("OPENROUTER_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ if dry_run => String::new(),
        _ => {
            eprintln!(
                "agenttest: OPENROUTER_API_KEY is not set.\n\
                 This harness makes real API calls and its results need human\n\
                 review — run it deliberately, as a milestone action."
            );
            std::process::exit(2);
        }
    };
    let model = std::env::var("ANGLOFORM_TEST_MODEL")
        .unwrap_or_else(|_| "deepseek/deepseek-v4-flash".to_string());
    let trials: usize = std::env::var("ANGLOFORM_TEST_TRIALS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let temperature: f64 = std::env::var("ANGLOFORM_TEST_TEMP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.7);

    let lexicon = Lexicon::load(LEXICON_PATH).expect("lexicon.tsv — run lexgen first");
    let system_prompt = build_system_prompt();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("paragraphs") {
        let file = args.get(1).expect("usage: agenttest paragraphs <markdown> [out.md] [--dry-run]");
        let dry = args.iter().any(|a| a == "--dry-run");
        let out = args
            .get(2)
            .filter(|a| !a.starts_with("--"))
            .cloned()
            .unwrap_or_else(|| "docs/paragraph-report.md".to_string());
        paragraphs::run(&api_key, &model, temperature, trials, &system_prompt, &lexicon, file, &out, dry);
        return;
    }
    if args.first().map(String::as_str) == Some("fix") {
        let file = args.get(1).expect("usage: agenttest fix <sentences-file> [out.md]");
        let out = args.get(2).cloned().unwrap_or_else(|| "docs/autofix-report.md".to_string());
        autofix(&api_key, &model, temperature, &system_prompt, &lexicon, file, &out);
        return;
    }

    let mut paths: Vec<_> = std::fs::read_dir(CASES_DIR)
        .expect("tests/agent-cases")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    paths.sort();

    let concurrency: usize = std::env::var("ANGLOFORM_TEST_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16);
    let queue = std::sync::Mutex::new(paths.iter().cloned().collect::<std::collections::VecDeque<_>>());
    let totals = std::sync::Mutex::new((0usize, 0usize));
    std::thread::scope(|scope| {
        for _ in 0..concurrency.max(1) {
            scope.spawn(|| loop {
                let path = queue.lock().unwrap().pop_front();
                let Some(path) = path else { break };
                let (t, ok, log) = process_case(
                    &path, &api_key, &model, temperature, trials, &system_prompt, &lexicon,
                );
                print!("{log}");
                let mut tot = totals.lock().unwrap();
                tot.0 += t;
                tot.1 += ok;
            });
        }
    });
    let (total, first_try_ok) = *totals.lock().unwrap();
    println!(
        "\nagenttest: {}/{} first-try valid across {} cases ({} trials each, concurrency {})",
        first_try_ok,
        total,
        paths.len(),
        trials,
        concurrency
    );
}

/// Full repair-loop flow for one case file; returns (trials, first-try-valid,
/// human-readable log). Each case owns its file, so cases parallelize safely.
#[allow(clippy::too_many_arguments)]
fn process_case(
    path: &std::path::Path,
    api_key: &str,
    model: &str,
    temperature: f64,
    trials: usize,
    system_prompt: &str,
    lexicon: &Lexicon,
) -> (usize, usize, String) {
    let text = std::fs::read_to_string(path).expect("read case");
    let mut case: Case = serde_yaml::from_str(&text).expect("parse case yaml");
    let mut log = format!("case: {}\n", case.input);

    let error = diagnosis_text(&diagnose(lexicon, &case.input));
    let mut run = Run {
        at_unix: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        model: model.to_string(),
        temperature,
        trials,
        first_try_valid: 0,
        trial_detail: Vec::new(),
    };

    // trials are independent conversations — run them in parallel too
    let context = match (case.context_before.trim(), case.context_after.trim()) {
        ("", "") => String::new(),
        (b, a) => format!(
            "\n\nContext (do not rewrite; the sentence sits between these):\n  before: {b}\n  after: {a}"
        ),
    };
    let results: Vec<Trial> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..trials)
            .map(|_| {
                scope.spawn(|| {
                    run_trial(api_key, model, temperature, system_prompt, &case.input, &format!("{error}{context}"), lexicon)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (t, trial) in results.into_iter().enumerate() {
        if trial.attempts.first().is_some_and(|a| a.valid) {
            run.first_try_valid += 1;
        }
        log.push_str(&format!(
            "  trial {}: rounds={} final_valid={}\n",
            t + 1,
            trial.rounds,
            trial.final_valid
        ));
        run.trial_detail.push(trial);
    }

    // accumulate unique outputs (all attempts, failures included)
    let mut uniq: BTreeMap<String, (bool, usize)> = case
        .unique_outputs
        .iter()
        .map(|u| (u.output.clone(), (u.valid, u.count)))
        .collect();
    for tr in &run.trial_detail {
        for a in &tr.attempts {
            let e = uniq.entry(a.output.clone()).or_insert((a.valid, 0));
            e.1 += 1;
        }
    }
    case.unique_outputs = uniq
        .into_iter()
        .map(|(output, (valid, count))| UniqueOutput { output, valid, count })
        .collect();

    // snapshot = first valid final output of this run; verdict resets on change
    let new_snapshot = run
        .trial_detail
        .iter()
        .filter(|t| t.final_valid)
        .filter_map(|t| t.attempts.last())
        .map(|a| a.output.clone())
        .next()
        .unwrap_or_default();
    if !new_snapshot.is_empty() && new_snapshot != case.snapshot {
        case.snapshot = new_snapshot;
        case.verdict = "unreviewed".to_string();
    }

    let first_try = run.first_try_valid;
    case.runs.push(run);
    std::fs::write(path, serde_yaml::to_string(&case).unwrap()).expect("write case");
    (trials, first_try, log)
}

fn run_trial(
    api_key: &str,
    model: &str,
    temperature: f64,
    system_prompt: &str,
    input: &str,
    error: &str,
    lexicon: &Lexicon,
) -> Trial {
    let mut messages = vec![
        serde_json::json!({"role": "system", "content": system_prompt}),
        serde_json::json!({"role": "user", "content": format!(
            "This sentence was rejected by the angloform linter:\n\n  {}\n\n\
             Linter output:\n  {}\n\n\
             Reply with ONLY the corrected angloform sentence (or sentences, \
             separated by periods), preserving the original meaning. No \
             explanation, no quotes around your answer.",
            input, error
        )}),
    ];
    let mut attempts = Vec::new();
    for round in 0..MAX_ROUNDS {
        let output = match complete(api_key, model, temperature, &messages) {
            Ok(o) => clean_reply(&o),
            Err(e) => {
                eprintln!("  API error: {e}");
                break;
            }
        };
        let (valid, diag_text) = validate(lexicon, &output);
        attempts.push(Attempt {
            output: output.clone(),
            valid,
            diagnosis: (!valid).then(|| diag_text.clone()),
        });
        if valid {
            break;
        }
        if round + 1 < MAX_ROUNDS {
            messages.push(serde_json::json!({"role": "assistant", "content": output}));
            messages.push(serde_json::json!({"role": "user", "content": format!(
                "Still rejected. Linter output:\n  {diag_text}\n\n\
                 Reply with ONLY the corrected angloform sentence."
            )}));
        }
    }
    Trial {
        rounds: attempts.len(),
        final_valid: attempts.last().is_some_and(|a| a.valid),
        attempts,
    }
}

/// Multi-sentence replies are valid iff every sentence parses.
fn validate(lexicon: &Lexicon, reply: &str) -> (bool, String) {
    let mut problems = Vec::new();
    let mut any = false;
    for seg in reply.split(". ").map(|s| s.trim().trim_end_matches('.').trim()).filter(|s| !s.is_empty()) {
        any = true;
        match diagnose(lexicon, seg) {
            Diagnosis::Clean(_) => {}
            d => problems.push(format!("\"{seg}\": {}", diagnosis_text(&d))),
        }
    }
    if !any {
        return (false, "empty reply".to_string());
    }
    (problems.is_empty(), problems.join(" | "))
}

pub(crate) fn diagnosis_text(d: &Diagnosis) -> String {
    match d {
        Diagnosis::Clean(_) => "accepted".to_string(),
        Diagnosis::Word(m) => format!("WORD: {m}"),
        Diagnosis::Style(f) => format!("STYLE: {}", f.join("; ")),
        Diagnosis::Ambiguous { readings, findings } => {
            format!("AMBIGUOUS ({readings} readings): {}", findings.join("; "))
        }
        Diagnosis::Unknown => "not recognizable as an angloform-like sentence".to_string(),
    }
}

fn clean_reply(raw: &str) -> String {
    let line = raw.trim().lines().next().unwrap_or("").trim();
    line.trim_matches('`').trim().trim_matches('"').trim().to_string()
}

fn complete(
    api_key: &str,
    model: &str,
    temperature: f64,
    messages: &[serde_json::Value],
) -> Result<String, String> {
    complete_with(api_key, model, temperature, messages, 120)
}

pub(crate) fn complete_with(
    api_key: &str,
    model: &str,
    temperature: f64,
    messages: &[serde_json::Value],
    max_tokens: u32,
) -> Result<String, String> {
    let resp: serde_json::Value = ureq::post("https://openrouter.ai/api/v1/chat/completions")
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
        }))
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    resp["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("unexpected response: {resp}"))
}

/// Skill prose + the word list derived from the live lexicon (never drifts).
fn build_system_prompt() -> String {
    let skill = std::fs::read_to_string(SKILL_PATH).expect("skill file");
    let body = skill.splitn(3, "---").nth(2).unwrap_or(&skill).trim();
    let lex = std::fs::read_to_string(LEXICON_PATH).expect("lexicon");
    let mut by_group: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for line in lex.lines().filter(|l| !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        let [surface, kind, tag, _] = f[..] else { continue };
        if kind != "form" {
            continue;
        }
        let group = match tag {
            t if t.starts_with("NOUN") => "nouns (sg/pl)",
            t if t.starts_with("VERB_TRANS") => "transitive verbs (all forms)",
            t if t.starts_with("VERB_INTRANS") => "intransitive verbs (all forms)",
            "ADJ" => "adjectives",
            t if t.starts_with("PREP") => "prepositions",
            _ => "function words",
        };
        by_group.entry(group).or_default().push(surface);
    }
    let mut out = format!("{body}\n\n");
    for (group, words) in by_group {
        out.push_str(&format!("{group}: {}\n", words.join(", ")));
    }
    // ADR 0027: defined terms, one unit each, Capitalized in text
    if let Ok(text) = std::fs::read_to_string("domain/model.json") {
        if let Ok(model) = serde_json::from_str::<serde_json::Value>(&text) {
            out.push_str("\nDefined terms (write noun terms Capitalized, as one unit; they mean exactly this):\n");
            for e in model.as_array().into_iter().flatten() {
                let lemma = e["lemma"].as_str().unwrap_or("");
                let cat = e["category"].as_str().unwrap_or("");
                let def = e["definition"].as_str().unwrap_or("");
                let shown = if let Some(n) = e["forms"]["name"].as_str() {
                    n.to_string()
                } else if cat == "NOUN" || cat == "NAME" {
                    lemma.split(' ').map(|w| {
                        let mut c = w.chars();
                        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
                    }).collect::<Vec<_>>().join(" ")
                } else {
                    lemma.to_string()
                };
                let ex: Vec<&str> = e["examples"].as_array().into_iter().flatten().filter_map(|x| x.as_str()).filter(|x| !x.contains('\n')).collect();
                let ex = if ex.is_empty() { String::new() } else { format!(" Examples: {}.", ex.join("; ")) };
                out.push_str(&format!("- {shown} ({}): {def}{ex}\n", cat.to_lowercase()));
            }
        }
    }
    out
}

/// Autofix experiment: lint every sentence of a file; for rejections, run
/// the repair loop and write a PROPOSALS document for human review. Never
/// edits the source — repair output drifts meaning (see the findings doc),
/// so ADR 0012 review stands between proposal and application. The model
/// may reply "GAP: <reason>" when no meaning-preserving fix exists.
fn autofix(
    api_key: &str,
    model: &str,
    temperature: f64,
    system_prompt: &str,
    lexicon: &Lexicon,
    file: &str,
    out_path: &str,
) {
    let text = std::fs::read_to_string(file).expect("sentences file");
    let sentences: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let system_prompt = format!(
        "{system_prompt}\nIMPORTANT: if a meaning-preserving fix is impossible \
         with the available words and structures, reply exactly \
         `GAP: <one-line reason>` — NEVER reply with prose about your \
         limitations, apologies, or commentary; any non-GAP reply must be \
         nothing but the corrected angloform sentence(s)."
    );
    let mut out = format!(
        "# Autofix proposals: {file}\n\n*Generated by `agenttest fix` — \
         PROPOSALS ONLY, apply by hand after ADR 0012 meaning review.*\n\n"
    );
    let (mut clean, mut fixed, mut gaps, mut failed) = (0, 0, 0, 0);
    // parallel over sentences; results reassembled in order
    let concurrency: usize = std::env::var("ANGLOFORM_TEST_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16);
    let queue = std::sync::Mutex::new((0..sentences.len()).collect::<std::collections::VecDeque<_>>());
    let results = std::sync::Mutex::new(vec![None; sentences.len()]);
    std::thread::scope(|scope| {
        for _ in 0..concurrency.max(1) {
            scope.spawn(|| loop {
                let i = queue.lock().unwrap().pop_front();
                let Some(i) = i else { break };
                let sent = sentences[i];
                let entry = match diagnose(lexicon, sent) {
                    Diagnosis::Clean(_) => {
                        println!("  ✓ clean: {sent}");
                        (0u8, format!("- ✓ already angloform: {sent}\n"))
                    }
                    d => {
                        let error = diagnosis_text(&d);
                        let trial = run_trial(
                            api_key, model, temperature, &system_prompt, sent, &error, lexicon,
                        );
                        let last = trial.attempts.last();
                        if let Some(a) = last.filter(|a| a.output.starts_with("GAP:")) {
                            println!("  GAP: {sent}");
                            (2, format!("\n**GAP** — {sent}\n  - flag: {error}\n  - model: {}\n", a.output))
                        } else {
                            match last {
                                Some(a) if a.valid => {
                                    println!("  fixed ({}r): {sent}", trial.rounds);
                                    (1, format!("\n**PROPOSAL** ({} round(s)) — {sent}\n  - flag: {error}\n  → {}\n", trial.rounds, a.output))
                                }
                                _ => {
                                    println!("  UNRESOLVED: {sent}");
                                    (3, format!("\n**UNRESOLVED** — {sent}\n  - flag: {error}\n"))
                                }
                            }
                        }
                    }
                };
                results.lock().unwrap()[i] = Some(entry);
            });
        }
    });
    for r in results.into_inner().unwrap().into_iter().flatten() {
        match r.0 {
            0 => clean += 1,
            1 => fixed += 1,
            2 => gaps += 1,
            _ => failed += 1,
        }
        out.push_str(&r.1);
    }
    out.push_str(&format!(
        "\n**Summary:** {} sentences — {clean} already angloform, {fixed} proposals, \
         {gaps} declared gaps, {failed} unresolved.\n",
        sentences.len()
    ));
    std::fs::write(out_path, &out).expect("write report");
    println!(
        "autofix: {clean} clean, {fixed} proposals, {gaps} gaps, {failed} unresolved → {out_path}"
    );
}

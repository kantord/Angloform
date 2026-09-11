//! `seed/definitions/<lemma>.yaml` — the per-word definition format (ADR
//! 0061). Two checks, both required, neither a substitute for the other:
//! `seed/definitions.schema.json` (shape only — required fields, enum
//! values, the redirects/advice mutual exclusivity) and `parse_definition`
//! (grammar — is the text actually a valid Angloform definition of the
//! shape its Category implies).

use serde::Deserialize;
use std::collections::BTreeMap;

pub const DEFINITIONS_DIR: &str = "seed/definitions";
pub const SCHEMA_PATH: &str = "seed/definitions.schema.json";

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SurfaceFormOverride {
    pub tag: String,
    pub form: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RejectedSense {
    pub category: String,
    #[serde(default)]
    pub redirects: Option<Vec<String>>,
    #[serde(default)]
    pub advice: Option<String>,
    #[serde(default)]
    pub gap: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DefinitionEntry {
    pub category: String,
    #[serde(default)]
    pub forms: Vec<SurfaceFormOverride>,
    pub definition: String,
    #[serde(default)]
    pub antonym: Option<String>,
    #[serde(default)]
    pub rejected: Vec<RejectedSense>,
}

/// `DefinitionEntry.category` (and `RejectedSense.category`, when it names
/// one of these 4) → the grammar entry point that checks its text.
pub fn def_kind_of(category: &str) -> Option<grammar::DefKind> {
    match category {
        "NOUN" => Some(grammar::DefKind::Noun),
        "VERB_TRANS" => Some(grammar::DefKind::VerbTrans),
        "VERB_INTRANS" => Some(grammar::DefKind::VerbIntrans),
        "ADJ" => Some(grammar::DefKind::Adj),
        // ADR 0062: a color adjective's own definition is adjective-shaped
        // too (grammatically it composes like ADJ everywhere) — only its
        // Form Tag differs, not the definition text's grammar.
        "COLOR_ADJ" => Some(grammar::DefKind::Adj),
        _ => None,
    }
}

/// `SurfaceFormOverride.tag` → the `seed.json`-style slot name
/// `crate::seed`'s `expand()` already knows how to fill (`Category::slots`).
/// The one lossy spot: `VERB_..._ED` covers both "past" and "past
/// participle" (`crate::seed::SeedEntry.forms` keeps them as separate
/// slots, "past"/"ppart", so two different explicit `..._ED` spellings —
/// e.g. "wrote" and "written" — need to land in different slots). Order
/// matters here: the first `..._ED` override seen is "past", a second,
/// *different*, one is "ppart" (irregular verbs where they coincide, e.g.
/// "held"/"held", only ever need one entry anyway).
fn tag_to_slot(tag: &str, seen_ed: &mut Option<String>) -> Option<&'static str> {
    match tag {
        "NOUN_PL" => Some("plural"),
        "ADJ_CMP" => Some("comparative"),
        "ADJ_SUP" => Some("superlative"),
        "VERB_TRANS_3SG" | "VERB_INTRANS_3SG" => Some("third"),
        "VERB_TRANS_ING" | "VERB_INTRANS_ING" => Some("ing"),
        "VERB_TRANS_ED" | "VERB_INTRANS_ED" => match seen_ed {
            None => {
                *seen_ed = Some(tag.to_string());
                Some("past")
            }
            Some(_) => Some("ppart"),
        },
        _ => None,
    }
}

/// Every `seed/definitions/*.yaml` entry, as a `SeedEntry` — the single
/// shape the rest of `lexgen` already knows how to expand, cross-check,
/// and render, regardless of whether a word came from `seed.json` or from
/// here (ADR 0061 + the follow-up: one loader, one pipeline, for both
/// sources). Doesn't set `explicit` per surface form the way `seed.json`'s
/// own loader implicitly does via `expand()` — an override here is always
/// explicit, same as a `seed.json` `forms` entry always is.
pub fn to_seed_entries(defs: &BTreeMap<String, DefinitionEntry>) -> Vec<crate::seed::SeedEntry> {
    defs.iter()
        .map(|(lemma, entry)| {
            let mut forms = BTreeMap::new();
            let mut seen_ed = None;
            for f in &entry.forms {
                if let Some(slot) = tag_to_slot(&f.tag, &mut seen_ed) {
                    forms.insert(slot.to_string(), f.form.clone());
                }
            }
            let mut reject = BTreeMap::new();
            for r in &entry.rejected {
                let target = match (&r.redirects, &r.advice, r.gap) {
                    (Some(words), _, _) => crate::seed::RejectTarget::Word(words[0].clone()),
                    (None, Some(advice), _) => crate::seed::RejectTarget::Advice { advice: advice.clone() },
                    (None, None, Some(true)) => crate::seed::RejectTarget::Gap,
                    (None, None, _) => continue, // schema/check already flagged this
                };
                reject.insert(r.category.clone(), target);
            }
            crate::seed::SeedEntry {
                lemma: lemma.clone(),
                category: entry.category.clone(),
                forms,
                reject,
                advice: String::new(),
                definition: String::new(),
                kind: String::new(),
                examples: Vec::new(),
                member_of: String::new(),
                domain: false,
                pack: None,
                note: String::new(),
            }
        })
        .collect()
}

/// Every `seed/definitions/*.yaml` file, keyed by lemma (the filename minus
/// `.yaml`) — loaded and schema-checked; parse errors and schema
/// violations are pushed onto `errors` rather than failing fast, so one
/// bad file doesn't hide the rest (matches every other lint block in
/// `main.rs`). Files that fail to parse as YAML at all are skipped (their
/// error is still recorded); everything downstream only sees entries that
/// at least parsed.
pub fn load(dir: &str, schema_path: &str, errors: &mut Vec<String>) -> BTreeMap<String, DefinitionEntry> {
    let mut out = BTreeMap::new();

    let schema_json: serde_json::Value = match std::fs::read_to_string(schema_path) {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{schema_path}: invalid JSON: {e}"));
                return out;
            }
        },
        Err(e) => {
            errors.push(format!("{schema_path}: {e}"));
            return out;
        }
    };
    let validator = match jsonschema::validator_for(&schema_json) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("{schema_path}: not a valid JSON Schema: {e}"));
            return out;
        }
    };

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out, // no seed/definitions/ yet — nothing to load
    };
    let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();

    for path in paths {
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let lemma = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("{}: {e}", path.display()));
                continue;
            }
        };

        let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{}: invalid YAML: {e}", path.display()));
                continue;
            }
        };
        let json_value: serde_json::Value = match serde_json::to_value(&yaml_value) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{}: cannot convert to JSON for schema validation: {e}", path.display()));
                continue;
            }
        };
        for err in validator.iter_errors(&json_value) {
            errors.push(format!("{}: schema: {err} (at {})", path.display(), err.instance_path()));
        }

        match serde_yaml::from_str::<DefinitionEntry>(&text) {
            Ok(entry) => {
                out.insert(lemma, entry);
            }
            Err(e) => {
                errors.push(format!("{}: {e}", path.display()));
            }
        }
    }

    out
}

/// The grammar checks `load`'s schema pass can't do — `parse_definition`
/// on `definition` and on every `rejected[].advice`, plus `antonym` and
/// `rejected[].redirects` resolving to real words. Run only after the
/// full `Lexicon` is built (self-reference/domain-term checks inside
/// `parse_definition` need to see every word, not just this file's own).
pub fn check(defs: &BTreeMap<String, DefinitionEntry>, lexicon: &grammar::Lexicon, errors: &mut Vec<String>) {
    for (lemma, entry) in defs {
        let path = format!("{DEFINITIONS_DIR}/{lemma}.yaml");

        let Some(kind) = def_kind_of(&entry.category) else {
            errors.push(format!("{path}: category \"{}\" has no definition shape", entry.category));
            continue;
        };
        if let Err(e) = grammar::parse_definition(lexicon, kind, lemma, &entry.definition) {
            errors.push(format!("{path}: definition: {e}"));
        }

        if let Some(antonym) = &entry.antonym {
            if lexicon.lemma_of(antonym).is_none() && !defs.contains_key(antonym) {
                errors.push(format!("{path}: antonym \"{antonym}\" is not a word of the Seed"));
            }
        }

        for (i, r) in entry.rejected.iter().enumerate() {
            match (&r.redirects, &r.advice, r.gap) {
                (Some(redirects), None, None) => {
                    for word in redirects {
                        if lexicon.lemma_of(word).is_none() && !defs.contains_key(word) {
                            errors.push(format!(
                                "{path}: rejected[{i}] redirect \"{word}\" is not a word of the Seed"
                            ));
                        }
                    }
                }
                (None, Some(advice), None) => {
                    if let Some(rkind) = def_kind_of(&r.category) {
                        if let Err(e) = grammar::parse_definition(lexicon, rkind, lemma, advice) {
                            errors.push(format!("{path}: rejected[{i}].advice: {e}"));
                        }
                    }
                    // a rejected category with no definition shape (e.g. ADV)
                    // still gets free advice text — nothing further to check.
                }
                (None, None, Some(true)) => {
                    // Gap: a fixed, non-authored template — nothing to
                    // check, that's the point (docs/lexicon-authoring-
                    // format-2026-09-07.md's "Gap" concept).
                }
                _ => errors.push(format!(
                    "{path}: rejected[{i}] must name exactly one of `redirects`, `advice`, or \
                     `gap` (the schema should already have caught this)"
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &std::path::Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).unwrap();
    }

    fn real_lexicon() -> grammar::Lexicon {
        let path = format!("{}/../../lexicon.tsv", env!("CARGO_MANIFEST_DIR"));
        grammar::Lexicon::load(&path).unwrap()
    }

    /// The pipeline this whole module exists for (ADR 0061): a schema
    /// violation and a grammar violation are BOTH caught, by DIFFERENT
    /// checks — losing either one silently would be worse than losing
    /// neither, since a shape-valid-but-ungrammatical definition would
    /// look checked when it isn't.
    #[test]
    fn catches_a_schema_violation_and_a_grammar_violation_separately() {
        let tmp = tempdir();
        write(tmp.path(), "good.yaml", "category: NOUN\ndefinition: a thing, which holds items\n");
        write(tmp.path(), "bad_category.yaml", "category: BOGUS\ndefinition: a thing, which holds items\n");
        write(tmp.path(), "bad_grammar.yaml", "category: VERB_TRANS\ndefinition: build from parts\n");

        let mut errors = Vec::new();
        let defs = load(tmp.path().to_str().unwrap(), schema_fixture_path(), &mut errors);
        assert!(
            errors.iter().any(|e| e.contains("bad_category") && e.contains("BOGUS")),
            "expected a schema-enum error for bad_category.yaml, got: {errors:?}"
        );
        assert_eq!(defs.len(), 3, "a schema violation is recorded as an error but doesn't stop the entry from loading — the deeper `check()` pass still needs to see it: {defs:?}");

        check(&defs, &real_lexicon(), &mut errors);
        assert!(
            errors.iter().any(|e| e.contains("bad_grammar") && e.contains("definition:")),
            "expected a parse_definition error for bad_grammar.yaml (\"build from parts\" — VERB_TRANS with no object), got: {errors:?}"
        );
        assert!(
            !errors.iter().any(|e| e.contains("good.yaml")),
            "good.yaml should not have produced any error: {errors:?}"
        );
    }

    #[test]
    fn accepts_multiple_redirects_looked_up_from_their_own_definitions() {
        let tmp = tempdir();
        write(
            tmp.path(),
            "good.yaml",
            "category: VERB_TRANS\ndefinition: make a copy of a thing\nrejected:\n  - category: NOUN\n    redirects: [copy]\n",
        );
        let mut errors = Vec::new();
        let defs = load(tmp.path().to_str().unwrap(), schema_fixture_path(), &mut errors);
        assert!(errors.is_empty(), "unexpected schema errors: {errors:?}");
        check(&defs, &real_lexicon(), &mut errors);
        assert!(errors.is_empty(), "unexpected check errors: {errors:?}");
    }

    #[test]
    fn rejects_redirects_and_advice_together_and_neither() {
        let tmp = tempdir();
        write(
            tmp.path(),
            "both.yaml",
            "category: NOUN\ndefinition: a thing, which holds items\nrejected:\n  - category: VERB\n    redirects: [file]\n    advice: not real advice\n",
        );
        let mut errors = Vec::new();
        load(tmp.path().to_str().unwrap(), schema_fixture_path(), &mut errors);
        assert!(
            errors.iter().any(|e| e.contains("both.yaml") && e.contains("oneOf")),
            "expected a schema oneOf violation, got: {errors:?}"
        );
    }

    fn schema_fixture_path() -> &'static str {
        // The real, committed schema — not a test fixture copy, so this
        // test also catches the schema file itself going invalid.
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../seed/definitions.schema.json")
    }

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }
}

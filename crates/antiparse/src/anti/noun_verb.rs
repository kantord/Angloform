use super::Repair;
use grammar::Redirect;

#[derive(Debug, Clone)]
pub struct NounVerbMatch {
    pub intro: Option<String>,
    pub word: String,
    pub next_det: Option<String>,
}

impl NounVerbMatch {
    /// A single deterministic repair only when the lemma has a known VERB
    /// *word* redirect (the substitution data already lives in the seed —
    /// ADR 0008's per-sense-synonym policy is exactly what makes this case
    /// fully mechanical, unlike bare_coord's ellipsis case). An advice-only
    /// redirect (ADR 0060: no word substitutes) can't be substituted into
    /// the sentence text, so it falls back to `Repair::None` with the
    /// advice surfaced instead of the generic message.
    pub fn repair(&self, verb_redirect: Option<&Redirect>) -> Repair {
        match verb_redirect {
            Some(Redirect::Word(v)) => Repair::Single(format!(
                "{}{}",
                self.intro.as_ref().map(|i| format!("{i} ")).unwrap_or_default(),
                v
            )),
            Some(Redirect::Advice(a)) => Repair::None(format!("\"{}\" — {a}", self.word)),
            None => Repair::None(format!(
                "\"{}\" has no known verb-sense redirect — restructure the sentence",
                self.word
            )),
        }
    }
}

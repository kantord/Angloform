//! Regular English morphology. Deliberately small: irregulars belong in the
//! seed entry's `forms` override, and a wrong guess here is caught loudly by
//! the unattested-form lint (e.g. generated "runned" is attested nowhere).

pub fn pluralize(noun: &str) -> String {
    sibilant_s(noun)
}

pub fn third_singular(verb: &str) -> String {
    sibilant_s(verb)
}

pub fn past(verb: &str) -> String {
    if verb.ends_with('e') {
        return format!("{verb}d");
    }
    if let Some(stem) = strip_consonant_y(verb) {
        return format!("{stem}ied");
    }
    format!("{}ed", doubled(verb))
}

pub fn gerund(verb: &str) -> String {
    if let Some(stem) = verb.strip_suffix("ie") {
        return format!("{stem}ying");
    }
    if verb.ends_with('e') && !verb.ends_with("ee") {
        return format!("{}ing", &verb[..verb.len() - 1]);
    }
    format!("{}ing", doubled(verb))
}

/// One syllable, or two ending in -y — the same split ADR 0030's
/// comparative uses and ADR 0056's superlative reuses (both inflect only
/// on this class; longer adjectives take the periphrastic "more"/"most").
fn is_short(adj: &str) -> bool {
    let syllables = adj
        .chars()
        .fold((0usize, false), |(n, prev_vowel), c| {
            let v = is_vowel(c) || c == 'y';
            (if v && !prev_vowel { n + 1 } else { n }, v)
        })
        .0
        .max(1);
    syllables == 1 || (syllables == 2 && adj.ends_with('y'))
}

/// Inflected comparative for short adjectives (ADR 0030): one syllable, or
/// two ending in -y. Longer adjectives return None and use "more <adj>".
pub fn comparative(adj: &str) -> Option<String> {
    if !is_short(adj) {
        return None;
    }
    Some(if adj.ends_with('e') {
        format!("{adj}r")
    } else if let Some(stem) = strip_consonant_y(adj) {
        format!("{stem}ier")
    } else {
        format!("{}er", doubled(adj))
    })
}

/// Inflected superlative for short adjectives (ADR 0056's grammar; same
/// short/long split as `comparative`). Longer adjectives return None and
/// use "most <adj>".
pub fn superlative(adj: &str) -> Option<String> {
    if !is_short(adj) {
        return None;
    }
    Some(if adj.ends_with('e') {
        format!("{adj}st")
    } else if let Some(stem) = strip_consonant_y(adj) {
        format!("{stem}iest")
    } else {
        format!("{}est", doubled(adj))
    })
}

/// -s / -es / -ies, shared by noun plurals and 3rd-person singular.
fn sibilant_s(word: &str) -> String {
    if let Some(stem) = strip_consonant_y(word) {
        return format!("{stem}ies");
    }
    let es = ["s", "x", "z", "ch", "sh", "o"];
    if es.iter().any(|suf| word.ends_with(suf)) {
        return format!("{word}es");
    }
    format!("{word}s")
}

/// For "retry" → Some("retr"); None when the y follows a vowel ("play").
fn strip_consonant_y(word: &str) -> Option<&str> {
    let stem = word.strip_suffix('y')?;
    let last = stem.chars().last()?;
    (!is_vowel(last)).then_some(stem)
}

/// Final-consonant doubling for short CVC words ("stop" → "stopp").
/// Stress can't be detected, so this only fires for words ≤ 4 letters;
/// longer CVC words that need doubling take a `forms` override.
fn doubled(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    if word.len() <= 4 && chars.len() >= 3 {
        let [a, b, c] = [
            chars[chars.len() - 3],
            chars[chars.len() - 2],
            chars[chars.len() - 1],
        ];
        if !is_vowel(a) && is_vowel(b) && !is_vowel(c) && !"wxy".contains(c) {
            return format!("{word}{c}");
        }
    }
    word.to_string()
}

fn is_vowel(c: char) -> bool {
    "aeiou".contains(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_morphology() {
        assert_eq!(pluralize("agent"), "agents");
        assert_eq!(pluralize("process"), "processes");
        assert_eq!(pluralize("query"), "queries");
        assert_eq!(third_singular("watch"), "watches");
        assert_eq!(third_singular("go"), "goes");
        assert_eq!(third_singular("retry"), "retries");
        assert_eq!(past("store"), "stored");
        assert_eq!(past("retry"), "retried");
        assert_eq!(past("stop"), "stopped");
        assert_eq!(past("start"), "started");
        assert_eq!(past("reject"), "rejected");
        assert_eq!(gerund("write"), "writing");
        assert_eq!(gerund("stop"), "stopping");
        assert_eq!(gerund("die"), "dying");
        assert_eq!(gerund("see"), "seeing");
        assert_eq!(gerund("fail"), "failing");
        assert_eq!(comparative("big").as_deref(), Some("bigger"));
        assert_eq!(superlative("big").as_deref(), Some("biggest"));
        assert_eq!(superlative("easy").as_deref(), Some("easiest"));
        assert_eq!(superlative("short").as_deref(), Some("shortest"));
        assert_eq!(superlative("transparent"), None);
    }

    // Property tests: these functions are pure string transforms fed
    // arbitrary lemma text (attested-form checking happens elsewhere, in
    // seedcheck.py against real corpora — these properties are about
    // *shape* and *crash-freedom*, not correctness of any one output).
    // Regular English morphology always lands on a fixed suffix regardless
    // of the input; any counterexample proptest finds here is either a bug
    // or a lemma that belongs in a `forms` override, not the regular rules.
    proptest::proptest! {
        #[test]
        fn no_panic_on_arbitrary_input(s in "\\PC{0,40}") {
            // any Unicode text, including empty — every function here must
            // return, never panic (byte-length vs. char-count mismatches
            // in `doubled` are exactly the kind of bug this would catch)
            let _ = pluralize(&s);
            let _ = third_singular(&s);
            let _ = past(&s);
            let _ = gerund(&s);
            let _ = comparative(&s);
            let _ = superlative(&s);
        }

        #[test]
        fn pluralize_and_third_singular_end_in_s(s in "[a-z]{1,20}") {
            proptest::prop_assert!(pluralize(&s).ends_with('s'));
            proptest::prop_assert!(third_singular(&s).ends_with('s'));
        }

        #[test]
        fn past_ends_in_d(s in "[a-z]{0,20}") {
            proptest::prop_assert!(past(&s).ends_with('d'));
        }

        #[test]
        fn gerund_ends_in_ing(s in "[a-z]{0,20}") {
            proptest::prop_assert!(gerund(&s).ends_with("ing"));
        }

        #[test]
        fn comparative_when_some_ends_in_er(s in "[a-z]{1,20}") {
            if let Some(c) = comparative(&s) {
                proptest::prop_assert!(c.ends_with("er"));
            }
        }

        #[test]
        fn superlative_when_some_ends_in_est(s in "[a-z]{1,20}") {
            if let Some(s) = superlative(&s) {
                proptest::prop_assert!(s.ends_with("est"));
            }
        }

        #[test]
        fn outputs_never_shrink_below_input(s in "[a-z]{1,20}") {
            // every rule only appends or substitutes a trailing letter for
            // a longer suffix; the result is never shorter than the input
            proptest::prop_assert!(pluralize(&s).len() >= s.len());
            proptest::prop_assert!(third_singular(&s).len() >= s.len());
            proptest::prop_assert!(past(&s).len() >= s.len());
            proptest::prop_assert!(gerund(&s).len() >= s.len());
        }
    }
}

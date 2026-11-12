//! Game-plan suggestions (M150): short claims about a plan's matchup drawn
//! from the Draft Graph's trait rules, which a coach adds to the plan as a
//! line of their own or dismisses for that plan.
//!
//! The server builds the full ordered list (`server::graph::suggest`); the
//! page shows the first [`MAX_SHOWN`] that are neither dismissed on the plan
//! nor already written in it, each rule once before any repeats
//! ([`visible`], [`vary_rules`]). Everything here is pure and
//! compiles for both targets.

use serde::{Deserialize, Serialize};

/// The most suggestions the page shows at once.
pub const MAX_SHOWN: usize = 3;
/// The most dismissed keys a plan keeps.
pub const MAX_DISMISSED: usize = 64;
/// The longest dismissed key kept.
pub const MAX_DISMISSED_KEY: usize = 96;
/// The longest claim, in characters.
pub const MAX_CLAIM_CHARS: usize = 60;

/// Where an accepted claim is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionTarget {
    /// A line in the plan's Win conditions.
    WinConditions,
    /// A line in the plan's Notes.
    Notes,
}

/// One suggestion for a plan's matchup.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    /// Dismiss key: `rule_id:idA:idB` for a pair (champion ids; sorted for a
    /// synergy, "a beats b" order for a counter) or `rule_id:ours|theirs`
    /// for a composition claim.
    pub key: String,
    /// The trait rule behind it ("syn.engage_burst", "fit.needs_engage").
    pub rule_id: String,
    /// The short claim (at most [`MAX_CLAIM_CHARS`] characters), written
    /// into the plan as is when accepted.
    pub claim: String,
    /// The full one-sentence reason.
    pub reason: String,
    /// Where it comes from ("Trait rules").
    pub source: String,
    /// The pair it is about (display names, in the claim's order); empty for
    /// a composition claim.
    pub pair: Vec<String>,
    /// 1 slight, 2 clear, 3 strong.
    pub strength: u8,
    pub target: SuggestionTarget,
}

/// `rule_id:key` or `rule_id:key:key`: the rule id in `a-z`, `_` and `.`,
/// the parts in ASCII letters and digits.
fn key_has_shape(k: &str) -> bool {
    let mut parts = k.split(':');
    let Some(rule) = parts.next() else {
        return false;
    };
    if rule.is_empty()
        || !rule
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '_' || c == '.')
    {
        return false;
    }
    let rest: Vec<&str> = parts.collect();
    (1..=2).contains(&rest.len())
        && rest
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// Bound a client-supplied list of dismissed keys: trimmed, empty and
/// over-long entries and unknown shapes dropped, duplicates removed (the
/// first kept), at most [`MAX_DISMISSED`] entries.
pub fn sanitize_dismissed(keys: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for k in keys {
        if out.len() >= MAX_DISMISSED {
            break;
        }
        let k = k.trim();
        if k.is_empty() || k.chars().count() > MAX_DISMISSED_KEY || !key_has_shape(k) {
            continue;
        }
        if out.iter().any(|o| o == k) {
            continue;
        }
        out.push(k.to_string());
    }
    out
}

fn fold_line(s: &str) -> String {
    s.trim().to_lowercase()
}

/// The claim is already a line of the plan's Win conditions or Notes
/// (trimmed, case-folded, whole-line match). Accepting writes the claim as
/// a plain line, so this is how an accepted chip retires; deleting the line
/// brings it back.
pub fn claim_written(claim: &str, win_text: &str, notes_text: &str) -> bool {
    let c = fold_line(claim);
    if c.is_empty() {
        return false;
    }
    win_text
        .lines()
        .chain(notes_text.lines())
        .any(|l| fold_line(l) == c)
}

/// The field's text with `claim` added as its last line ("Add to plan"):
/// a newline first unless the field is empty or already ends with one.
/// Plain text, no marker.
pub fn append_claim(field: &str, claim: &str) -> String {
    let claim = claim.trim();
    if field.trim().is_empty() {
        claim.to_string()
    } else if field.ends_with('\n') {
        format!("{field}{claim}")
    } else {
        format!("{field}\n{claim}")
    }
}

/// Reorder a ranked list for variety: the first suggestion of each rule id
/// (in the given order), then the repeats (in the given order). Stable and
/// deterministic, so a lineup where one rule fires on many pairs does not
/// fill the shown chips with the same fact.
pub fn vary_rules(ranked: Vec<Suggestion>) -> Vec<Suggestion> {
    let mut seen: Vec<String> = Vec::new();
    let mut firsts = Vec::with_capacity(ranked.len());
    let mut repeats = Vec::new();
    for s in ranked {
        if seen.iter().any(|r| r == &s.rule_id) {
            repeats.push(s);
        } else {
            seen.push(s.rule_id.clone());
            firsts.push(s);
        }
    }
    firsts.extend(repeats);
    firsts
}

/// The suggestions to show: of `all` (in the server's order), those neither
/// dismissed nor written in the plan, reordered by [`vary_rules`] (so a
/// dismissal does not bring a repeated rule forward while another rule is
/// left), the first [`MAX_SHOWN`].
pub fn visible(
    all: &[Suggestion],
    dismissed: &[String],
    win: &str,
    notes: &str,
) -> Vec<Suggestion> {
    let left: Vec<Suggestion> = all
        .iter()
        .filter(|s| !dismissed.iter().any(|d| d == &s.key))
        .filter(|s| !claim_written(&s.claim, win, notes))
        .cloned()
        .collect();
    let mut out = vary_rules(left);
    out.truncate(MAX_SHOWN);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(key: &str, claim: &str) -> Suggestion {
        Suggestion {
            key: key.into(),
            rule_id: key.split(':').next().unwrap_or_default().into(),
            claim: claim.into(),
            reason: format!("{claim}."),
            source: "Trait rules".into(),
            pair: vec![],
            strength: 2,
            target: SuggestionTarget::WinConditions,
        }
    }

    #[test]
    fn append_claim_adds_one_line() {
        assert_eq!(append_claim("", "A claim"), "A claim");
        assert_eq!(append_claim("  \n", "A claim"), "A claim");
        assert_eq!(append_claim("Win lane", "A claim"), "Win lane\nA claim");
        assert_eq!(append_claim("Win lane\n", " A claim "), "Win lane\nA claim");
        assert!(claim_written("A claim", &append_claim("x", "A claim"), ""));
    }

    #[test]
    fn sanitize_keeps_good_keys_in_order() {
        let got = sanitize_dismissed(vec![
            " syn.engage_burst:Malphite:Syndra ".into(),
            "fit.needs_engage:ours".into(),
        ]);
        assert_eq!(
            got,
            vec![
                "syn.engage_burst:Malphite:Syndra".to_string(),
                "fit.needs_engage:ours".to_string()
            ]
        );
    }

    #[test]
    fn sanitize_drops_bad_shapes() {
        let bad = [
            "",
            "   ",
            "syn.engage_burst",
            "syn.engage_burst:",
            "syn.engage_burst::Syndra",
            "Syn.engage:A:B",
            "syn.x:A:B:C",
            "syn.x:Kai'Sa:B",
            "syn.x:A B:C",
            ":A:B",
            "syn-x:A:B",
            "syn.x:A:<script>",
            "syn1:A:B",
        ];
        let got = sanitize_dismissed(bad.iter().map(|b| b.to_string()).collect());
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn sanitize_drops_long_keys_and_dedupes() {
        let long = format!("syn.x:{}:B", "A".repeat(100));
        let got = sanitize_dismissed(vec![
            long,
            "syn.x:A:B".into(),
            "syn.x:A:B".into(),
            " syn.x:A:B".into(),
        ]);
        assert_eq!(got, vec!["syn.x:A:B".to_string()]);
    }

    #[test]
    fn sanitize_caps_the_list() {
        let keys: Vec<String> = (0..200).map(|i| format!("syn.x:A{i}:B")).collect();
        let got = sanitize_dismissed(keys);
        assert_eq!(got.len(), MAX_DISMISSED);
        assert_eq!(got[0], "syn.x:A0:B");
    }

    #[test]
    fn claim_written_matches_whole_lines_case_folded() {
        assert!(claim_written(
            "Malphite engages, Syndra bursts",
            "Win lane\n  malphite ENGAGES, syndra bursts  ",
            ""
        ));
        assert!(claim_written(
            "Our comp has no hard engage",
            "",
            "our comp has no hard engage"
        ));
        assert!(!claim_written(
            "Malphite engages, Syndra bursts",
            "Malphite engages, Syndra bursts early",
            ""
        ));
        assert!(!claim_written("", "", ""));
        assert!(!claim_written("A claim", "", ""));
    }

    #[test]
    fn visible_skips_dismissed_and_written_and_caps() {
        let all = vec![
            s("syn.a:A:B", "One"),
            s("syn.b:A:B", "Two"),
            s("syn.c:A:B", "Three"),
            s("syn.d:A:B", "Four"),
            s("syn.e:A:B", "Five"),
        ];
        let v = visible(&all, &["syn.a:A:B".into()], "two", "");
        let claims: Vec<&str> = v.iter().map(|s| s.claim.as_str()).collect();
        assert_eq!(claims, vec!["Three", "Four", "Five"]);
        assert_eq!(visible(&all, &[], "", "").len(), MAX_SHOWN);
        assert!(visible(&[], &[], "", "").is_empty());
    }

    #[test]
    fn vary_rules_puts_each_rule_first_once() {
        let all = vec![
            s("syn.wombo:A:B", "W1"),
            s("syn.wombo:A:C", "W2"),
            s("syn.wombo:B:C", "W3"),
            s("syn.cc_chain:A:D", "C1"),
            s("ctr.peel_dive:E:F", "P1"),
            s("syn.cc_chain:B:D", "C2"),
        ];
        let order: Vec<String> = vary_rules(all.clone())
            .into_iter()
            .map(|s| s.claim)
            .collect();
        assert_eq!(order, vec!["W1", "C1", "P1", "W2", "W3", "C2"]);
        let shown: Vec<String> = visible(&all, &[], "", "")
            .into_iter()
            .map(|s| s.claim)
            .collect();
        assert_eq!(shown, vec!["W1", "C1", "P1"]);
        // Dismissing the first wombo: the next wombo stays behind the other
        // rules, which still fill the chips with distinct rules.
        let shown: Vec<String> = visible(&all, &["syn.wombo:A:B".into()], "", "")
            .into_iter()
            .map(|s| s.claim)
            .collect();
        assert_eq!(shown, vec!["W2", "C1", "P1"]);
        // Dismissing the only peel: a repeat fills the third chip.
        let shown: Vec<String> = visible(&all, &["ctr.peel_dive:E:F".into()], "", "")
            .into_iter()
            .map(|s| s.claim)
            .collect();
        assert_eq!(shown, vec!["W1", "C1", "W2"]);
    }
}

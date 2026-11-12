//! Game-plan suggestions (M150): what the trait rules say about a finished
//! matchup, as short claims a coach can add to the plan.
//!
//! [`suggest`] reads only the precomputed pair table
//! ([`PairRelations::between`](super::rules::PairRelations::between)) and
//! the picks' [`CompProfile`]; it never calls the slot ranking
//! (`fit::rank`), the team's games or the Riot API. It is pure: the same
//! picks (in any order, by id or display name) give the same list.
//!
//! Candidates, once both sides have [`FULL_SIDE`] resolved picks:
//! - our counters on their picks (we beat them): Win conditions;
//! - their counters on our picks (they beat us): Notes;
//! - synergies between our picks: Win conditions;
//! - composition claims over our five (what we lack): Notes;
//! - composition claims over their five (what they lack): Win conditions.
//!
//! Composition claims reuse the ids of [`FIT_RULE_IDS`] and the Draft
//! Graph's thresholds ([`CompProfile::little_frontline`] and the rest); no
//! new predicate. Their strength: 2 for little front line and one-sided
//! damage (they decide how fights go), 1 for no hard engage and slow wave
//! clear. Their sentences, `{side}` being "our" or "their" ("Our"/"Their"
//! at the start):
//!
//! | id | claim | reason |
//! |----|-------|--------|
//! | `fit.needs_engage` | "{Side} comp has no hard engage" | "None of {side} five champions brings hard engage" |
//! | `fit.needs_frontline` | "{Side} comp has little front line" | "{Side} five champions add up to little front line" |
//! | `fit.needs_wave_clear` | "{Side} comp clears waves slowly" | "{Side} champions clear waves slowly on average" |
//! | `fit.one_sided_damage` | "{Side} damage is mostly {physical\|magic}" | "At least three quarters of {side} damage is {physical\|magic}" |
//!
//! Pair claims use [`claim_template`]; their reason is the rule's own
//! sentence ([`Rule::sentence`]).
//!
//! Order: strength (highest first), then Win conditions before Notes, then
//! rule id, key and claim. One suggestion per key, and at most one per
//! unordered champion pair (the strongest). At most [`MAX_SUGGESTIONS`].

use std::collections::HashSet;

use super::rules::{Rule, FIT_RULE_IDS, RULES};
use super::traits::TraitTable;
use crate::models::graph::{CompProfile, Engage, ReasonSource, RelationKind};
use crate::models::suggestion::{Suggestion, SuggestionTarget, MAX_CLAIM_CHARS};

/// Picks a side needs before anything is suggested ("both drafts are in").
pub const FULL_SIDE: usize = 5;
/// The most suggestions returned (the page shows three; the rest stand in
/// when some are dismissed or written).
pub const MAX_SUGGESTIONS: usize = 12;

/// The short claim for a pair rule, `{a}` and `{b}` being the two names in
/// the rule's order (for a counter, `{a}` beats `{b}`). Every id in
/// [`RULES`] has one (a test checks it); with the longest display names
/// (14 characters) every claim fits in [`MAX_CLAIM_CHARS`].
pub fn claim_template(rule_id: &str) -> Option<&'static str> {
    Some(match rule_id {
        // Synergy.
        "syn.engage_burst" => "{a} engages, {b} bursts",
        "syn.engage_wombo" => "{a}'s engage sets up {b}",
        "syn.wombo" => "{a} and {b} combine ultimates",
        "syn.protect_scale" => "{a} keeps {b} alive to scale",
        "syn.protect_style" => "{a} plays around {b}",
        "syn.cc_chain" => "{a} and {b} chain crowd control",
        "syn.dive_together" => "{a} and {b} dive together",
        "syn.dive_followup" => "{b} follows {a}'s engage",
        "syn.siege_range" => "{a} and {b} siege from range",
        "syn.frontline_carry" => "{a} fronts for {b}",
        "syn.catch_followup" => "{a} catches, {b} follows up",
        "syn.disengage_poke" => "{a} peels while {b} pokes",
        "syn.split_hold" => "{b} holds waves, {a} splits",
        "syn.early_pressure" => "{a} and {b} press early",
        "syn.skirmish" => "{a} and {b} win skirmishes",
        // Counter: a beats b.
        "ctr.poke_outranges" => "{a} pokes {b} down from range",
        "ctr.engage_denied" => "{a} undoes {b}'s engage",
        "ctr.peel_dive" => "{a} peels {b}'s dive",
        "ctr.side_lane" => "{a} splits where {b} can't clear",
        "ctr.burst_immobile" => "{a} reaches and bursts {b}",
        "ctr.dive_backline" => "{a} dives {b}, who can't escape",
        "ctr.cc_mobile_melee" => "{a}'s crowd control pins {b}",
        "ctr.lane_range" => "{a} outranges {b} in lane",
        "ctr.true_vs_tank" => "{a}'s true damage cuts {b}",
        "ctr.sustained_vs_tank" => "{a} wears {b} down",
        "ctr.engage_on_poke" => "{a} engages before {b} pokes",
        "ctr.mobile_vs_poke" => "{a} dodges {b}'s poke",
        "ctr.catch_immobile" => "{a} catches {b}",
        "ctr.clear_siege" => "{a} clears {b}'s siege waves",
        _ => return None,
    })
}

/// Every rule id a suggestion can carry (pair rules and composition ids).
pub fn is_known_rule_id(id: &str) -> bool {
    RULES.iter().any(|r| r.id == id) || FIT_RULE_IDS.contains(&id)
}

/// Keep a claim within [`MAX_CLAIM_CHARS`]: cut at the last word boundary
/// that leaves room for "…".
pub fn fit_claim(s: String) -> String {
    if s.chars().count() <= MAX_CLAIM_CHARS {
        return s;
    }
    let head: String = s.chars().take(MAX_CLAIM_CHARS - 1).collect();
    let cut = match head.rfind(' ') {
        Some(i) if i > 0 => &head[..i],
        _ => head.as_str(),
    };
    format!("{}…", cut.trim_end_matches([',', ' ']))
}

/// Row indices of a side's picks: unknown and blank picks ignored, each
/// champion once, in row (champion id) order.
fn resolve(table: &TraitTable, picks: &[String]) -> Vec<usize> {
    let mut out: Vec<usize> = picks
        .iter()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .filter_map(|p| table.index(p))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

struct Cand {
    s: Suggestion,
    /// The unordered pair (row indices), for pair rules.
    pair: Option<(usize, usize)>,
}

fn pair_cand(
    table: &TraitTable,
    rule: &Rule,
    a: usize,
    b: usize,
    target: SuggestionTarget,
) -> Option<Cand> {
    let template = claim_template(rule.id)?;
    let (ra, rb) = (&table.rows[a], &table.rows[b]);
    let (ida, idb) = (ra.traits.champion.as_str(), rb.traits.champion.as_str());
    let key = match rule.kind {
        RelationKind::Synergy => {
            let (x, y) = if ida <= idb { (ida, idb) } else { (idb, ida) };
            format!("{}:{x}:{y}", rule.id)
        }
        RelationKind::Counter => format!("{}:{ida}:{idb}", rule.id),
    };
    let claim = fit_claim(template.replace("{a}", &ra.name).replace("{b}", &rb.name));
    Some(Cand {
        s: Suggestion {
            key,
            rule_id: rule.id.to_string(),
            claim,
            reason: rule.sentence(&ra.name, &rb.name),
            source: ReasonSource::Rule.label().to_string(),
            pair: vec![ra.name.clone(), rb.name.clone()],
            strength: rule.weight.clamp(1, 3),
            target,
        },
        pair: Some((a.min(b), a.max(b))),
    })
}

fn comp_cand(rule_id: &str, side: &str, claim: String, reason: String, strength: u8) -> Cand {
    let target = if side == "ours" {
        SuggestionTarget::Notes
    } else {
        SuggestionTarget::WinConditions
    };
    Cand {
        s: Suggestion {
            key: format!("{rule_id}:{side}"),
            rule_id: rule_id.to_string(),
            claim: fit_claim(claim),
            reason,
            source: ReasonSource::Rule.label().to_string(),
            pair: Vec::new(),
            strength,
            target,
        },
        pair: None,
    }
}

/// Composition claims over one side's five.
fn comp_cands(table: &TraitTable, side_rows: &[usize], side: &str, out: &mut Vec<Cand>) {
    let traits: Vec<_> = side_rows.iter().map(|i| &table.rows[*i].traits).collect();
    let profile = CompProfile::sum(traits.iter().copied());
    let (cap, low) = if side == "ours" {
        ("Our", "our")
    } else {
        ("Their", "their")
    };
    if !traits.iter().any(|t| t.engage == Engage::Hard) {
        out.push(comp_cand(
            "fit.needs_engage",
            side,
            format!("{cap} comp has no hard engage"),
            format!("None of {low} five champions brings hard engage"),
            1,
        ));
    }
    if profile.little_frontline() {
        out.push(comp_cand(
            "fit.needs_frontline",
            side,
            format!("{cap} comp has little front line"),
            format!("{cap} five champions add up to little front line"),
            2,
        ));
    }
    if profile.slow_wave_clear() {
        out.push(comp_cand(
            "fit.needs_wave_clear",
            side,
            format!("{cap} comp clears waves slowly"),
            format!("{cap} champions clear waves slowly on average"),
            1,
        ));
    }
    let kind = if profile.physical_heavy() {
        Some("physical")
    } else if profile.magic_heavy() {
        Some("magic")
    } else {
        None
    };
    if let Some(kind) = kind {
        out.push(comp_cand(
            "fit.one_sided_damage",
            side,
            format!("{cap} damage is mostly {kind}"),
            format!("At least three quarters of {low} damage is {kind}"),
            2,
        ));
    }
}

fn target_rank(t: SuggestionTarget) -> u8 {
    match t {
        SuggestionTarget::WinConditions => 0,
        SuggestionTarget::Notes => 1,
    }
}

/// The suggestions for a matchup, strongest first (see the module doc).
/// Empty unless both sides resolve to [`FULL_SIDE`] champions.
pub fn suggest(table: &TraitTable, ours: &[String], theirs: &[String]) -> Vec<Suggestion> {
    let o = resolve(table, ours);
    let t = resolve(table, theirs);
    if o.len() < FULL_SIDE || t.len() < FULL_SIDE {
        return Vec::new();
    }
    let rel = &table.relations;
    let mut cands: Vec<Cand> = Vec::new();

    // Synergies between our picks, both directions; a rule that holds both
    // ways is taken once, in row order.
    for (n, &x) in o.iter().enumerate() {
        for &y in &o[n + 1..] {
            let mut seen: Vec<&str> = Vec::new();
            for r in rel
                .between(x, y)
                .filter(|r| r.kind == RelationKind::Synergy)
            {
                seen.push(r.id);
                cands.extend(pair_cand(table, r, x, y, SuggestionTarget::WinConditions));
            }
            for r in rel
                .between(y, x)
                .filter(|r| r.kind == RelationKind::Synergy && !seen.contains(&r.id))
            {
                cands.extend(pair_cand(table, r, y, x, SuggestionTarget::WinConditions));
            }
        }
    }
    // Counters across the sides.
    for &a in &o {
        for &b in &t {
            for r in rel
                .between(a, b)
                .filter(|r| r.kind == RelationKind::Counter)
            {
                cands.extend(pair_cand(table, r, a, b, SuggestionTarget::WinConditions));
            }
            for r in rel
                .between(b, a)
                .filter(|r| r.kind == RelationKind::Counter)
            {
                cands.extend(pair_cand(table, r, b, a, SuggestionTarget::Notes));
            }
        }
    }
    comp_cands(table, &o, "ours", &mut cands);
    comp_cands(table, &t, "theirs", &mut cands);

    cands.sort_by(|x, y| {
        y.s.strength
            .cmp(&x.s.strength)
            .then(target_rank(x.s.target).cmp(&target_rank(y.s.target)))
            .then_with(|| x.s.rule_id.cmp(&y.s.rule_id))
            .then_with(|| x.s.key.cmp(&y.s.key))
            .then_with(|| x.s.claim.cmp(&y.s.claim))
    });
    let mut keys: HashSet<String> = HashSet::new();
    let mut pairs: HashSet<(usize, usize)> = HashSet::new();
    let mut out = Vec::new();
    for c in cands {
        if out.len() >= MAX_SUGGESTIONS {
            break;
        }
        if keys.contains(&c.s.key) {
            continue;
        }
        if let Some(p) = c.pair {
            if !pairs.insert(p) {
                continue;
            }
        }
        keys.insert(c.s.key.clone());
        out.push(c.s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::{
        ChampionTraits, DamageProfile, DamageType, Role, Strength, TraitRecord, TraitSource,
    };
    use crate::models::suggestion::sanitize_dismissed;
    use crate::server::graph::traits::{parse_defaults, DEFAULTS_CSV};

    fn record(t: ChampionTraits, name: &str) -> TraitRecord {
        TraitRecord {
            name: name.to_string(),
            traits: t,
            source: TraitSource::Default,
            edited_by: None,
            edited_at: None,
            reviewed_for_patch: None,
        }
    }

    /// The shipped defaults, with a few display names that differ from ids.
    fn csv_table() -> TraitTable {
        let rows = parse_defaults(DEFAULTS_CSV)
            .unwrap()
            .into_iter()
            .map(|t| {
                let name = match t.champion.as_str() {
                    "MonkeyKing" => "Wukong".to_string(),
                    "JarvanIV" => "Jarvan IV".to_string(),
                    "Nunu" => "Nunu & Willump".to_string(),
                    other => other.to_string(),
                };
                record(t, &name)
            })
            .collect();
        TraitTable::new(rows)
    }

    fn v(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| x.to_string()).collect()
    }

    const OURS: &[&str] = &["Ornn", "LeeSin", "Orianna", "Jinx", "Leona"];
    const THEIRS: &[&str] = &["Gnar", "Nidalee", "Xerath", "Ezreal", "Lulu"];

    #[test]
    fn every_pair_rule_has_a_claim_template() {
        for r in RULES {
            let t = claim_template(r.id).unwrap_or_else(|| panic!("{} has no claim", r.id));
            assert!(t.contains("{a}") && t.contains("{b}"), "{}: {t}", r.id);
        }
        for id in FIT_RULE_IDS {
            assert!(is_known_rule_id(id));
        }
        assert!(!is_known_rule_id("syn.made_up"));
    }

    #[test]
    fn claims_fit_with_the_longest_names() {
        let table = csv_table();
        let longest = table
            .rows
            .iter()
            .map(|r| {
                r.name
                    .chars()
                    .count()
                    .max(r.traits.champion.chars().count())
            })
            .max()
            .unwrap();
        assert!(longest <= 14, "a display name is longer than 14: {longest}");
        let n = "Nunu & Willump";
        for r in RULES {
            let c = claim_template(r.id)
                .unwrap()
                .replace("{a}", n)
                .replace("{b}", n);
            assert!(
                c.chars().count() <= MAX_CLAIM_CHARS,
                "{}: {c} ({})",
                r.id,
                c.chars().count()
            );
        }
    }

    #[test]
    fn fit_claim_cuts_at_a_word() {
        let long = "word ".repeat(20);
        let c = fit_claim(long);
        assert!(c.chars().count() <= MAX_CLAIM_CHARS, "{c}");
        assert!(c.ends_with("word…"), "{c}");
        assert_eq!(fit_claim("short".into()), "short");
    }

    #[test]
    fn full_sides_give_bounded_valid_suggestions() {
        let table = csv_table();
        let got = suggest(&table, &v(OURS), &v(THEIRS));
        assert!(!got.is_empty());
        assert!(got.len() <= MAX_SUGGESTIONS);
        let keys: Vec<String> = got.iter().map(|s| s.key.clone()).collect();
        assert_eq!(
            sanitize_dismissed(keys.clone()),
            keys,
            "keys pass the bound"
        );
        let mut pairs = HashSet::new();
        for s in &got {
            assert!(s.key.chars().count() <= 96);
            assert!(s.claim.chars().count() <= MAX_CLAIM_CHARS, "{}", s.claim);
            assert!((1..=3).contains(&s.strength));
            assert!(is_known_rule_id(&s.rule_id));
            assert!(s.key.starts_with(&format!("{}:", s.rule_id)));
            assert_eq!(s.source, "Trait rules");
            if s.pair.len() == 2 {
                for name in &s.pair {
                    assert!(s.reason.contains(name.as_str()), "{} / {}", s.reason, name);
                }
                let mut p = s.pair.clone();
                p.sort();
                assert!(pairs.insert(p), "one suggestion per pair: {:?}", s.pair);
            } else {
                assert!(s.pair.is_empty());
                assert!(s.key.ends_with(":ours") || s.key.ends_with(":theirs"));
            }
        }
        // Sorted by strength.
        assert!(got.windows(2).all(|w| w[0].strength >= w[1].strength));
    }

    #[test]
    fn same_picks_same_suggestions_in_any_order_or_spelling() {
        let table = csv_table();
        let a = suggest(&table, &v(OURS), &v(THEIRS));
        let b = suggest(&table, &v(OURS), &v(THEIRS));
        assert_eq!(a, b);
        let shuffled_ours = v(&["leona", "Jinx", " Orianna ", "Lee Sin", "ornn"]);
        let shuffled_theirs = v(&["Lulu", "Xerath", "Gnar", "Ezreal", "Nidalee"]);
        let c = suggest(&table, &shuffled_ours, &shuffled_theirs);
        assert_eq!(a, c);
    }

    #[test]
    fn an_incomplete_side_gives_nothing() {
        let table = csv_table();
        assert!(suggest(&table, &[], &v(THEIRS)).is_empty());
        assert!(suggest(&table, &v(OURS), &[]).is_empty());
        assert!(suggest(&table, &v(&OURS[..4]), &v(THEIRS)).is_empty());
        // An unknown, a blank and a repeated pick do not count.
        let padded = v(&["Ornn", "LeeSin", "Orianna", "Jinx", "Notachampion"]);
        assert!(suggest(&table, &padded, &v(THEIRS)).is_empty());
        let repeated = v(&["Ornn", "LeeSin", "Orianna", "Jinx", "ornn"]);
        assert!(suggest(&table, &repeated, &v(THEIRS)).is_empty());
        let blank = v(&["Ornn", "LeeSin", "Orianna", "Jinx", "  "]);
        assert!(suggest(&table, &blank, &v(THEIRS)).is_empty());
    }

    /// Ten neutral rows (no engage, no front line): the composition claims
    /// fire on both sides, with their fixed keys and targets.
    #[test]
    fn composition_claims_name_the_side() {
        let mut rows = Vec::new();
        for i in 0..10 {
            let mut t = ChampionTraits::neutral(format!("Champ{i}"), vec![Role::Mid]);
            t.damage_type = DamageType::Physical;
            rows.push(record(t, &format!("Champ {i}")));
        }
        let table = TraitTable::new(rows);
        let ours = v(&["Champ0", "Champ1", "Champ2", "Champ3", "Champ4"]);
        let theirs = v(&["Champ5", "Champ6", "Champ7", "Champ8", "Champ9"]);
        let got = suggest(&table, &ours, &theirs);
        let find = |k: &str| got.iter().find(|s| s.key == k).cloned();
        let e = find("fit.needs_engage:ours").expect("no engage, ours");
        assert_eq!(e.target, SuggestionTarget::Notes);
        assert_eq!(e.claim, "Our comp has no hard engage");
        let e = find("fit.needs_engage:theirs").expect("no engage, theirs");
        assert_eq!(e.target, SuggestionTarget::WinConditions);
        assert_eq!(e.claim, "Their comp has no hard engage");
        let d = find("fit.one_sided_damage:ours").expect("physical");
        assert_eq!(d.claim, "Our damage is mostly physical");
        assert_eq!(d.strength, 2);
        assert!(find("fit.needs_frontline:theirs").is_some());
        // Wave clear "fair" on every pick is not slow.
        assert!(find("fit.needs_wave_clear:ours").is_none());
    }

    /// A counter both ways on one pair keeps only the stronger; a synergy is
    /// keyed by the sorted ids.
    #[test]
    fn pair_keys_and_one_per_pair() {
        let mut rows = Vec::new();
        for i in 0..10 {
            let t = ChampionTraits::neutral(format!("Champ{i}"), vec![Role::Mid]);
            rows.push(record(t, &format!("Champ {i}")));
        }
        // Champ3 engages hard, Champ1 bursts: syn.engage_burst (3) on ours.
        rows[3].traits.engage = Engage::Hard;
        rows[1].traits.damage_profile = DamageProfile::Burst;
        // Champ7 (theirs) has strong disengage: ctr.engage_denied on Champ3.
        rows[7].traits.disengage = Strength::Strong;
        let table = TraitTable::new(rows);
        let ours = v(&["Champ0", "Champ1", "Champ2", "Champ3", "Champ4"]);
        let theirs = v(&["Champ5", "Champ6", "Champ7", "Champ8", "Champ9"]);
        let got = suggest(&table, &ours, &theirs);
        let syn = got
            .iter()
            .find(|s| s.rule_id == "syn.engage_burst")
            .expect("synergy");
        assert_eq!(syn.key, "syn.engage_burst:Champ1:Champ3");
        assert_eq!(syn.pair, v(&["Champ 3", "Champ 1"]));
        assert_eq!(syn.claim, "Champ 3 engages, Champ 1 bursts");
        assert_eq!(syn.target, SuggestionTarget::WinConditions);
        assert_eq!(got[0].key, syn.key, "strongest first");
        let ctr = got
            .iter()
            .find(|s| s.rule_id == "ctr.engage_denied")
            .expect("counter");
        assert_eq!(ctr.key, "ctr.engage_denied:Champ7:Champ3");
        assert_eq!(ctr.target, SuggestionTarget::Notes);
        assert_eq!(ctr.reason, "Champ 7 can undo Champ 3's engage");
    }
}

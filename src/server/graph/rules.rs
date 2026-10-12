//! Trait rules: the readable rule set that turns two trait rows into a
//! relation a coach can argue with.
//!
//! A [`Rule`] has an id, a kind (synergy or counter), a weight (1 slight,
//! 2 clear, 3 strong), a sentence template and a predicate over two trait
//! rows. Rules are directional: `applies(a, b)` means "a works with b" for a
//! synergy (the sentence names who does what) and "a beats b" for a
//! counter. The graph asks both orders for synergies.
//!
//! [`PairRelations::compute`] evaluates every rule for every ordered pair of
//! champions once per table load (about 30,000 pairs, a few milliseconds),
//! so ranking only looks them up.
//!
//! Composition rules (what the comp still lacks) and power-spike rules live
//! in `fit.rs`; their ids are listed in [`FIT_RULE_IDS`] so the docs and the
//! tests see the whole set.

use crate::models::graph::{
    ChampionTraits as T, CrowdControl, DamageProfile, DamageType, Engage, Mobility, Playstyle,
    RangeClass, RelationKind, Role, Scaling, Strength, WaveClear,
};

/// One trait rule.
pub struct Rule {
    /// Stable id ("syn.engage_burst"), shown with the reason.
    pub id: &'static str,
    pub kind: RelationKind,
    /// 1 slight, 2 clear, 3 strong.
    pub weight: u8,
    /// Sentence with `{a}` and `{b}` for the two champions' names.
    pub template: &'static str,
    pub applies: fn(&T, &T) -> bool,
}

impl Rule {
    /// The sentence for two names.
    pub fn sentence(&self, a: &str, b: &str) -> String {
        self.template.replace("{a}", a).replace("{b}", b)
    }
}

fn cc_at_least(t: &T, level: CrowdControl) -> bool {
    t.cc.level() >= level.level()
}

fn shares_role(a: &T, b: &T) -> bool {
    a.roles.iter().any(|r| b.roles.contains(r))
}

fn mobile(t: &T) -> bool {
    t.mobility != Mobility::Low
}

fn late_carry(t: &T) -> bool {
    t.scaling == Scaling::Late
        && (t.plays(Role::Bot) || t.plays(Role::Mid))
        && matches!(
            t.damage_profile,
            DamageProfile::Sustained | DamageProfile::Hybrid
        )
        && t.frontline == Strength::Absent
}

use RelationKind::{Counter, Synergy};

/// The pair rules. Ids are stable: tests, docs and saved explanations name
/// them.
pub static RULES: &[Rule] = &[
    // ---- Synergy -------------------------------------------------------
    Rule {
        id: "syn.engage_burst",
        kind: Synergy,
        weight: 3,
        template: "{a} starts fights that {b}'s burst finishes",
        applies: |a, b| a.engage == Engage::Hard && b.damage_profile == DamageProfile::Burst,
    },
    Rule {
        id: "syn.engage_wombo",
        kind: Synergy,
        weight: 3,
        template: "{a}'s engage sets up {b}'s area damage",
        applies: |a, b| {
            a.engage == Engage::Hard && b.has_style(Playstyle::Wombo) && b.engage != Engage::Hard
        },
    },
    Rule {
        id: "syn.wombo",
        kind: Synergy,
        weight: 3,
        template: "{a} and {b} land their teamfight ultimates together",
        applies: |a, b| a.has_style(Playstyle::Wombo) && b.has_style(Playstyle::Wombo),
    },
    Rule {
        id: "syn.protect_scale",
        kind: Synergy,
        weight: 3,
        template: "{a} keeps {b} alive while {b} scales",
        applies: |a, b| a.peel == Strength::Strong && late_carry(b),
    },
    Rule {
        id: "syn.protect_style",
        kind: Synergy,
        weight: 2,
        template: "{a} plays around {b}, a late carry",
        applies: |a, b| {
            a.has_style(Playstyle::ProtectTheCarry)
                && a.peel != Strength::Strong
                && b.scaling == Scaling::Late
                && b.range == RangeClass::Long
        },
    },
    Rule {
        id: "syn.cc_chain",
        kind: Synergy,
        weight: 2,
        template: "{a} and {b} chain their crowd control",
        applies: |a, b| {
            cc_at_least(a, CrowdControl::Medium) && cc_at_least(b, CrowdControl::Medium)
        },
    },
    Rule {
        id: "syn.dive_together",
        kind: Synergy,
        weight: 2,
        template: "{a} and {b} dive the backline together",
        applies: |a, b| {
            a.has_style(Playstyle::Dive) && b.has_style(Playstyle::Dive) && mobile(a) && mobile(b)
        },
    },
    Rule {
        id: "syn.dive_followup",
        kind: Synergy,
        weight: 2,
        template: "{b} follows {a}'s engage into the backline",
        applies: |a, b| {
            a.engage == Engage::Hard && b.has_style(Playstyle::Dive) && b.mobility == Mobility::High
        },
    },
    Rule {
        id: "syn.siege_range",
        kind: Synergy,
        weight: 2,
        template: "{a} and {b} siege from long range",
        applies: |a, b| {
            let long_poke = |t: &T| {
                t.range == RangeClass::Long
                    && (t.has_style(Playstyle::Poke) || t.has_style(Playstyle::Siege))
            };
            long_poke(a) && long_poke(b)
        },
    },
    Rule {
        id: "syn.frontline_carry",
        kind: Synergy,
        weight: 2,
        template: "{a} holds the front line while {b} deals damage",
        applies: |a, b| {
            a.frontline == Strength::Strong
                && b.range == RangeClass::Long
                && matches!(
                    b.damage_profile,
                    DamageProfile::Sustained | DamageProfile::Hybrid
                )
                && (b.plays(Role::Bot) || b.plays(Role::Mid))
                && b.peel != Strength::Strong
        },
    },
    Rule {
        id: "syn.catch_followup",
        kind: Synergy,
        weight: 2,
        template: "{a} catches a target and {b} jumps on it",
        applies: |a, b| {
            a.has_style(Playstyle::Pick)
                && cc_at_least(a, CrowdControl::Medium)
                && b.damage_profile == DamageProfile::Burst
                && b.mobility == Mobility::High
        },
    },
    Rule {
        id: "syn.disengage_poke",
        kind: Synergy,
        weight: 2,
        template: "{a} keeps divers off {b} while {b} pokes",
        applies: |a, b| a.disengage == Strength::Strong && b.damage_profile == DamageProfile::Poke,
    },
    Rule {
        id: "syn.split_hold",
        kind: Synergy,
        weight: 1,
        template: "{b} holds the waves while {a} splits",
        applies: |a, b| {
            a.has_style(Playstyle::SplitPush)
                && b.wave_clear == WaveClear::Strong
                && b.disengage != Strength::Absent
        },
    },
    Rule {
        id: "syn.early_pressure",
        kind: Synergy,
        weight: 1,
        template: "{a} and {b} press an early lead together",
        applies: |a, b| {
            a.scaling == Scaling::Early && b.scaling == Scaling::Early && mobile(a) && mobile(b)
        },
    },
    Rule {
        id: "syn.skirmish",
        kind: Synergy,
        weight: 1,
        template: "{a} and {b} win small fights together",
        applies: |a, b| {
            a.has_style(Playstyle::Skirmish)
                && b.has_style(Playstyle::Skirmish)
                && a.scaling != Scaling::Late
                && b.scaling != Scaling::Late
        },
    },
    // ---- Counter (a beats b) -------------------------------------------
    Rule {
        id: "ctr.poke_outranges",
        kind: Counter,
        weight: 2,
        template: "{a} pokes {b} down before {b} can reach the fight",
        applies: |a, b| {
            a.damage_profile == DamageProfile::Poke
                && a.range == RangeClass::Long
                && b.range == RangeClass::Melee
                && b.engage == Engage::Absent
                && b.mobility != Mobility::High
        },
    },
    Rule {
        id: "ctr.engage_denied",
        kind: Counter,
        weight: 2,
        template: "{a} can undo {b}'s engage",
        applies: |a, b| a.disengage == Strength::Strong && b.engage == Engage::Hard,
    },
    Rule {
        id: "ctr.peel_dive",
        kind: Counter,
        weight: 2,
        template: "{a} peels {b}'s dive off the carries",
        applies: |a, b| a.peel == Strength::Strong && b.has_style(Playstyle::Dive),
    },
    Rule {
        id: "ctr.side_lane",
        kind: Counter,
        weight: 1,
        template: "{a} pressures a side lane that {b} cannot clear",
        applies: |a, b| {
            a.has_style(Playstyle::SplitPush) && mobile(a) && b.wave_clear == WaveClear::Weak
        },
    },
    Rule {
        id: "ctr.burst_immobile",
        kind: Counter,
        weight: 2,
        template: "{a} reaches and bursts the immobile {b}",
        applies: |a, b| {
            a.damage_profile == DamageProfile::Burst
                && a.mobility == Mobility::High
                && b.mobility == Mobility::Low
                && b.frontline == Strength::Absent
                && b.range == RangeClass::Long
        },
    },
    Rule {
        id: "ctr.dive_backline",
        kind: Counter,
        weight: 2,
        template: "{a} dives onto {b}, who has little to escape with",
        applies: |a, b| {
            a.has_style(Playstyle::Dive)
                && a.mobility == Mobility::High
                && a.damage_profile != DamageProfile::Burst
                && b.range == RangeClass::Long
                && b.mobility == Mobility::Low
                && b.peel == Strength::Absent
                && b.frontline == Strength::Absent
        },
    },
    Rule {
        id: "ctr.cc_mobile_melee",
        kind: Counter,
        weight: 1,
        template: "{a}'s crowd control pins down {b}",
        applies: |a, b| {
            a.cc == CrowdControl::Heavy
                && b.mobility == Mobility::High
                && b.range == RangeClass::Melee
                && b.frontline == Strength::Absent
        },
    },
    Rule {
        id: "ctr.lane_range",
        kind: Counter,
        weight: 1,
        template: "{a} outranges {b} in a shared lane",
        applies: |a, b| {
            shares_role(a, b) && a.range == RangeClass::Long && b.range == RangeClass::Melee
        },
    },
    Rule {
        id: "ctr.true_vs_tank",
        kind: Counter,
        weight: 2,
        template: "{a}'s true damage cuts through {b}'s tankiness",
        applies: |a, b| a.damage_type == DamageType::TrueHeavy && b.frontline == Strength::Strong,
    },
    Rule {
        id: "ctr.sustained_vs_tank",
        kind: Counter,
        weight: 1,
        template: "{a}'s sustained damage wears {b} down",
        applies: |a, b| {
            a.damage_profile == DamageProfile::Sustained
                && a.range == RangeClass::Long
                && a.plays(Role::Bot)
                && b.frontline == Strength::Strong
        },
    },
    Rule {
        id: "ctr.engage_on_poke",
        kind: Counter,
        weight: 2,
        template: "{a} engages onto {b} before the poke adds up",
        applies: |a, b| {
            a.engage == Engage::Hard
                && b.damage_profile == DamageProfile::Poke
                && b.mobility == Mobility::Low
        },
    },
    Rule {
        id: "ctr.mobile_vs_poke",
        kind: Counter,
        weight: 1,
        template: "{a} moves out of {b}'s skillshot poke",
        applies: |a, b| a.mobility == Mobility::High && b.damage_profile == DamageProfile::Poke,
    },
    Rule {
        id: "ctr.catch_immobile",
        kind: Counter,
        weight: 1,
        template: "{a} catches the immobile {b}",
        applies: |a, b| {
            a.has_style(Playstyle::Pick)
                && cc_at_least(a, CrowdControl::Medium)
                && b.mobility == Mobility::Low
                && b.disengage == Strength::Absent
        },
    },
    Rule {
        id: "ctr.clear_siege",
        kind: Counter,
        weight: 1,
        template: "{a} clears the waves {b} sieges with",
        applies: |a, b| {
            a.wave_clear == WaveClear::Strong
                && a.disengage != Strength::Absent
                && b.has_style(Playstyle::Siege)
        },
    },
];

/// Ids of the composition and power-spike rules in `fit.rs`.
pub const FIT_RULE_IDS: &[&str] = &[
    "fit.tag_match",
    "fit.needs_engage",
    "fit.needs_frontline",
    "fit.needs_wave_clear",
    "fit.needs_magic",
    "fit.needs_physical",
    "fit.one_sided_damage",
    "spike.with_plan",
    "spike.against_plan",
];

/// Every rule that held for every ordered pair of a table, computed once.
pub struct PairRelations {
    n: usize,
    /// `hits[a * n + b]`: indices into [`RULES`] that hold for (a, b).
    hits: Vec<Vec<u8>>,
}

impl PairRelations {
    pub fn compute(rows: &[&T]) -> Self {
        let n = rows.len();
        let mut hits = vec![Vec::new(); n * n];
        for (i, a) in rows.iter().enumerate() {
            for (j, b) in rows.iter().enumerate() {
                if i == j {
                    continue;
                }
                let cell = &mut hits[i * n + j];
                for (k, r) in RULES.iter().enumerate() {
                    if (r.applies)(a, b) {
                        cell.push(k as u8);
                    }
                }
            }
        }
        Self { n, hits }
    }

    /// Rules that hold for (a, b), by row index.
    pub fn between(&self, a: usize, b: usize) -> impl Iterator<Item = &'static Rule> + '_ {
        let cell: &[u8] = if a < self.n && b < self.n {
            &self.hits[a * self.n + b]
        } else {
            &[]
        };
        cell.iter().map(|k| &RULES[*k as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::{ChampionTraits, Confidence};

    /// A neutral row to override: melee, no engage, nothing special.
    fn t(id: &str) -> ChampionTraits {
        let mut c = ChampionTraits::neutral(id, vec![Role::Mid]);
        c.confidence = Confidence::Normal;
        c.damage_type = DamageType::Physical;
        c.damage_profile = DamageProfile::Sustained;
        c
    }

    fn rule(id: &str) -> &'static Rule {
        RULES.iter().find(|r| r.id == id).expect(id)
    }

    fn holds(id: &str, a: &ChampionTraits, b: &ChampionTraits) -> bool {
        (rule(id).applies)(a, b)
    }

    /// Every rule: one pair where it holds and the neutral pair where it
    /// does not.
    #[test]
    fn each_rule_has_a_pair_it_explains() {
        let n = t("Neutral");
        let cases: Vec<(&str, ChampionTraits, ChampionTraits)> = vec![
            (
                "syn.engage_burst",
                {
                    let mut a = t("Malphite");
                    a.engage = Engage::Hard;
                    a
                },
                {
                    let mut b = t("Syndra");
                    b.damage_profile = DamageProfile::Burst;
                    b
                },
            ),
            (
                "syn.engage_wombo",
                {
                    let mut a = t("Leona");
                    a.engage = Engage::Hard;
                    a
                },
                {
                    let mut b = t("MissFortune");
                    b.styles = vec![Playstyle::Wombo];
                    b
                },
            ),
            (
                "syn.wombo",
                {
                    let mut a = t("Orianna");
                    a.styles = vec![Playstyle::Wombo];
                    a
                },
                {
                    let mut b = t("Yasuo");
                    b.styles = vec![Playstyle::Wombo];
                    b
                },
            ),
            (
                "syn.protect_scale",
                {
                    let mut a = t("Lulu");
                    a.peel = Strength::Strong;
                    a
                },
                {
                    let mut b = t("Jinx");
                    b.roles = vec![Role::Bot];
                    b.scaling = Scaling::Late;
                    b
                },
            ),
            (
                "syn.protect_style",
                {
                    let mut a = t("Yuumi");
                    a.styles = vec![Playstyle::ProtectTheCarry];
                    a
                },
                {
                    let mut b = t("KogMaw");
                    b.scaling = Scaling::Late;
                    b.range = RangeClass::Long;
                    b
                },
            ),
            (
                "syn.cc_chain",
                {
                    let mut a = t("Nautilus");
                    a.cc = CrowdControl::Heavy;
                    a
                },
                {
                    let mut b = t("Sejuani");
                    b.cc = CrowdControl::Medium;
                    b
                },
            ),
            (
                "syn.dive_together",
                {
                    let mut a = t("JarvanIV");
                    a.styles = vec![Playstyle::Dive];
                    a
                },
                {
                    let mut b = t("Diana");
                    b.styles = vec![Playstyle::Dive];
                    b.mobility = Mobility::High;
                    b
                },
            ),
            (
                "syn.dive_followup",
                {
                    let mut a = t("Rell");
                    a.engage = Engage::Hard;
                    a
                },
                {
                    let mut b = t("Kaisa");
                    b.styles = vec![Playstyle::Dive];
                    b.mobility = Mobility::High;
                    b
                },
            ),
            (
                "syn.siege_range",
                {
                    let mut a = t("Xerath");
                    a.range = RangeClass::Long;
                    a.styles = vec![Playstyle::Poke];
                    a
                },
                {
                    let mut b = t("Caitlyn");
                    b.range = RangeClass::Long;
                    b.styles = vec![Playstyle::Siege];
                    b
                },
            ),
            (
                "syn.frontline_carry",
                {
                    let mut a = t("Ornn");
                    a.frontline = Strength::Strong;
                    a
                },
                {
                    let mut b = t("Jinx");
                    b.range = RangeClass::Long;
                    b.roles = vec![Role::Bot];
                    b
                },
            ),
            (
                "syn.catch_followup",
                {
                    let mut a = t("Thresh");
                    a.styles = vec![Playstyle::Pick];
                    a.cc = CrowdControl::Heavy;
                    a
                },
                {
                    let mut b = t("Zed");
                    b.damage_profile = DamageProfile::Burst;
                    b.mobility = Mobility::High;
                    b
                },
            ),
            (
                "syn.disengage_poke",
                {
                    let mut a = t("Janna");
                    a.disengage = Strength::Strong;
                    a
                },
                {
                    let mut b = t("Ezreal");
                    b.damage_profile = DamageProfile::Poke;
                    b
                },
            ),
            (
                "syn.split_hold",
                {
                    let mut a = t("Fiora");
                    a.styles = vec![Playstyle::SplitPush];
                    a
                },
                {
                    let mut b = t("Anivia");
                    b.wave_clear = WaveClear::Strong;
                    b.disengage = Strength::Strong;
                    b
                },
            ),
            (
                "syn.early_pressure",
                {
                    let mut a = t("LeeSin");
                    a.scaling = Scaling::Early;
                    a
                },
                {
                    let mut b = t("Renekton");
                    b.scaling = Scaling::Early;
                    b
                },
            ),
            (
                "syn.skirmish",
                {
                    let mut a = t("Graves");
                    a.styles = vec![Playstyle::Skirmish];
                    a
                },
                {
                    let mut b = t("Lucian");
                    b.styles = vec![Playstyle::Skirmish];
                    b
                },
            ),
            (
                "ctr.poke_outranges",
                {
                    let mut a = t("Xerath");
                    a.damage_profile = DamageProfile::Poke;
                    a.range = RangeClass::Long;
                    a
                },
                {
                    let mut b = t("Garen");
                    b.mobility = Mobility::Low;
                    b
                },
            ),
            (
                "ctr.engage_denied",
                {
                    let mut a = t("Janna");
                    a.disengage = Strength::Strong;
                    a
                },
                {
                    let mut b = t("Malphite");
                    b.engage = Engage::Hard;
                    b
                },
            ),
            (
                "ctr.peel_dive",
                {
                    let mut a = t("Lulu");
                    a.peel = Strength::Strong;
                    a
                },
                {
                    let mut b = t("Diana");
                    b.styles = vec![Playstyle::Dive];
                    b
                },
            ),
            (
                "ctr.side_lane",
                {
                    let mut a = t("Fiora");
                    a.styles = vec![Playstyle::SplitPush];
                    a
                },
                {
                    let mut b = t("Vayne");
                    b.wave_clear = WaveClear::Weak;
                    b
                },
            ),
            (
                "ctr.burst_immobile",
                {
                    let mut a = t("Zed");
                    a.damage_profile = DamageProfile::Burst;
                    a.mobility = Mobility::High;
                    a
                },
                {
                    let mut b = t("Xerath");
                    b.mobility = Mobility::Low;
                    b.range = RangeClass::Long;
                    b
                },
            ),
            (
                "ctr.dive_backline",
                {
                    let mut a = t("Hecarim");
                    a.styles = vec![Playstyle::Dive];
                    a.mobility = Mobility::High;
                    a
                },
                {
                    let mut b = t("Jinx");
                    b.mobility = Mobility::Low;
                    b.range = RangeClass::Long;
                    b
                },
            ),
            (
                "ctr.cc_mobile_melee",
                {
                    let mut a = t("Leona");
                    a.cc = CrowdControl::Heavy;
                    a
                },
                {
                    let mut b = t("MasterYi");
                    b.mobility = Mobility::High;
                    b
                },
            ),
            (
                "ctr.lane_range",
                {
                    let mut a = t("Syndra");
                    a.range = RangeClass::Long;
                    a
                },
                t("Akali"),
            ),
            (
                "ctr.true_vs_tank",
                {
                    let mut a = t("Vayne");
                    a.damage_type = DamageType::TrueHeavy;
                    a
                },
                {
                    let mut b = t("Ornn");
                    b.frontline = Strength::Strong;
                    b
                },
            ),
            (
                "ctr.sustained_vs_tank",
                {
                    let mut a = t("KogMaw");
                    a.range = RangeClass::Long;
                    a.roles = vec![Role::Bot];
                    a
                },
                {
                    let mut b = t("Sion");
                    b.frontline = Strength::Strong;
                    b
                },
            ),
            (
                "ctr.engage_on_poke",
                {
                    let mut a = t("Malphite");
                    a.engage = Engage::Hard;
                    a
                },
                {
                    let mut b = t("Xerath");
                    b.damage_profile = DamageProfile::Poke;
                    b.mobility = Mobility::Low;
                    b
                },
            ),
            (
                "ctr.mobile_vs_poke",
                {
                    let mut a = t("Kassadin");
                    a.mobility = Mobility::High;
                    a
                },
                {
                    let mut b = t("Xerath");
                    b.damage_profile = DamageProfile::Poke;
                    b
                },
            ),
            (
                "ctr.catch_immobile",
                {
                    let mut a = t("Blitzcrank");
                    a.styles = vec![Playstyle::Pick];
                    a.cc = CrowdControl::Heavy;
                    a
                },
                {
                    let mut b = t("Jinx");
                    b.mobility = Mobility::Low;
                    b
                },
            ),
            (
                "ctr.clear_siege",
                {
                    let mut a = t("Anivia");
                    a.wave_clear = WaveClear::Strong;
                    a.disengage = Strength::Strong;
                    a
                },
                {
                    let mut b = t("Caitlyn");
                    b.styles = vec![Playstyle::Siege];
                    b
                },
            ),
        ];
        let tested: Vec<&str> = cases.iter().map(|c| c.0).collect();
        for r in RULES {
            assert!(tested.contains(&r.id), "rule {} has no test pair", r.id);
        }
        for (id, a, b) in &cases {
            assert!(
                holds(id, a, b),
                "{id} should hold for {} and {}",
                a.champion,
                b.champion
            );
            assert!(
                !holds(id, &n, &n),
                "{id} should not hold for two neutral rows"
            );
        }
    }

    #[test]
    fn rule_set_is_sized_and_unique() {
        assert!((20..=40).contains(&(RULES.len() + FIT_RULE_IDS.len())));
        let mut ids: Vec<&str> = RULES.iter().map(|r| r.id).collect();
        ids.extend(FIT_RULE_IDS);
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "rule ids are unique");
        for r in RULES {
            assert!((1..=3).contains(&r.weight), "{}", r.id);
            assert!(
                r.template.contains("{a}") && r.template.contains("{b}"),
                "{}",
                r.id
            );
            let prefix = match r.kind {
                Synergy => "syn.",
                Counter => "ctr.",
            };
            assert!(r.id.starts_with(prefix), "{}", r.id);
        }
        assert_eq!(
            rule("syn.engage_burst").sentence("Malphite", "Syndra"),
            "Malphite starts fights that Syndra's burst finishes"
        );
    }

    #[test]
    fn counters_are_directional() {
        let mut janna = t("Janna");
        janna.disengage = Strength::Strong;
        let mut malph = t("Malphite");
        malph.engage = Engage::Hard;
        assert!(holds("ctr.engage_denied", &janna, &malph));
        assert!(!holds("ctr.engage_denied", &malph, &janna));
    }

    #[test]
    fn pair_relations_cover_both_orders_from_the_shipped_defaults() {
        let rows = crate::server::graph::traits::parse_defaults(
            crate::server::graph::traits::DEFAULTS_CSV,
        )
        .unwrap();
        let refs: Vec<&ChampionTraits> = rows.iter().collect();
        let started = std::time::Instant::now();
        let rel = PairRelations::compute(&refs);
        let took = started.elapsed();
        eprintln!("pair precompute: {took:?} for {} rows", rows.len());
        let idx = |id: &str| rows.iter().position(|r| r.champion == id).unwrap();
        // Kit sanity on the shipped data: Malphite's engage with Orianna's
        // area damage; Janna undoes Malphite's engage.
        let ids: Vec<&str> = rel
            .between(idx("Malphite"), idx("Orianna"))
            .map(|r| r.id)
            .collect();
        assert!(ids.contains(&"syn.engage_wombo"), "{ids:?}");
        let ids: Vec<&str> = rel
            .between(idx("Janna"), idx("Malphite"))
            .map(|r| r.id)
            .collect();
        assert!(ids.contains(&"ctr.engage_denied"), "{ids:?}");
        assert_eq!(rel.between(idx("Ahri"), idx("Ahri")).count(), 0);
        assert!(took.as_millis() < 2000, "precompute took {took:?}");
    }
}

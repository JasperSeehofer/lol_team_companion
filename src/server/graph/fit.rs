//! Fit: ranks every candidate champion for the current pick slot.
//!
//! Fit starts at a neutral 50. Four parts, in the contract's priority order,
//! each move it by at most half their weight (see
//! [`Component::weight`](crate::models::graph::Component::weight)):
//!
//! | Part | Weight | Points | From |
//! |---|---|---|---|
//! | Synergy | 40 | ±20 | relations with the side's locked picks |
//! | Counters | 30 | ±15 | relations with the other side's locked picks (and, for our picks, the opponent's scouted champions) |
//! | Playstyle fit | 20 | ±10 | the draft's composition tags and what the comp still lacks |
//! | Power spikes | 10 | ±5 | the candidate's curve against the comp's plan |
//!
//! Then the slot player's pool adds −8 to +6 (our side only), and the result
//! is clamped to 0–100.
//!
//! A part's value is a sum of relation strengths (1 slight to 3 strong per
//! relation) divided by its full scale and clamped to −1..1:
//!
//! - a pair's trait rules add their weights (at most 3 per pair and kind);
//! - a team note adds its weight (at most 3 per pair and kind);
//! - match evidence (n ≥ 5) adds ±1 only when a rule or note already relates
//!   the pair: it corroborates, it never stands alone, and it is shown with
//!   its sample either way.
//!
//! Full scale: synergy 3 per locked pick of the side (a strong relation with
//! every pick), counters 3 per locked pick of the other side (both at least
//! 3), playstyle 3, power spikes 1. Every point has a [`Reason`].
//!
//! The ranking is a pure function of the trait table, the draft state and
//! the team's knowledge, so it is deterministic and unit-tested.

use std::collections::{HashMap, HashSet};

use crate::components::ui::{pick_slots, DraftAction, DraftSide, TOURNAMENT_ORDER};
use crate::models::graph::tier_label;
use crate::models::graph::{
    Candidate, CompProfile, Component, DamageType, DraftState, Engage, FitPart, GraphEdge,
    GraphNode, GraphNotes, GraphResult, GraphTarget, Playstyle, PoolStatus, Reason, ReasonSource,
    RelationKind, Role, Scaling, Strength, TeamRelation, TraitSource, WaveClear, MAX_CANDIDATES,
    MIN_SAMPLE,
};
use crate::server::graph::evidence::{MatchEvidence, Tally};
use crate::server::graph::traits::TraitTable;

/// A roster player with their pool, for the pool modifier.
#[derive(Clone, Debug)]
pub struct PoolPlayer {
    pub username: String,
    pub role: Option<Role>,
    /// (champion id, tier).
    pub entries: Vec<(String, String)>,
}

/// A champion the opponent dossier says one of their players plays.
#[derive(Clone, Debug)]
pub struct Scouted {
    /// Their player's name.
    pub player: String,
    pub role: Option<Role>,
    /// Champion id.
    pub champion: String,
}

/// What the team knows, gathered by the server function.
pub struct TeamKnowledge<'a> {
    pub team: bool,
    pub relations: &'a [TeamRelation],
    pub pools: &'a [PoolPlayer],
    pub evidence: &'a MatchEvidence,
    pub scouted: &'a [Scouted],
}

/// Full scale per locked pick for synergy and counters, in relation
/// strength (one strong relation with each pick).
const PER_PICK_SCALE: f32 = 3.0;
/// Full scale of the other parts.
const PLAYSTYLE_SCALE: f32 = 3.0;
const TIMING_SCALE: f32 = 1.0;
/// Cap per pair and kind for rules, and again for team notes.
const PAIR_CAP: f32 = 3.0;
/// Scouted champions count a third, at most this much in total.
const SCOUTING_CAP: f32 = 1.5;

/// Points the pool adds for a tier; `None` (not in the pool) takes 8.
pub fn tier_points(tier: Option<&str>) -> i32 {
    match tier {
        Some("comfort") => 6,
        Some("match_ready") => 4,
        Some("scrim_ready") => 2,
        Some("practicing") => 0,
        Some("to_practice") => -2,
        Some(_) => 0,
        None => -8,
    }
}

/// The draft's composition tags and the playstyles that serve them.
fn tag_styles(tag: &str) -> &'static [Playstyle] {
    match tag {
        "teamfight" => &[
            Playstyle::Teamfight,
            Playstyle::Wombo,
            Playstyle::FrontToBack,
        ],
        "split-push" => &[Playstyle::SplitPush],
        "poke" => &[Playstyle::Poke, Playstyle::Siege],
        "pick" => &[Playstyle::Pick],
        "protect-the-carry" => &[Playstyle::ProtectTheCarry, Playstyle::FrontToBack],
        _ => &[],
    }
}

fn clamp1(v: f32) -> f32 {
    v.clamp(-1.0, 1.0)
}

fn points(c: Component, value: f32) -> i32 {
    (c.band() as f32 * clamp1(value)).round() as i32
}

/// Team notes indexed by row pairs.
struct NoteIndex<'a> {
    synergy: HashMap<(usize, usize), Vec<&'a TeamRelation>>,
    /// (winner, loser).
    counter: HashMap<(usize, usize), Vec<&'a TeamRelation>>,
}

impl<'a> NoteIndex<'a> {
    fn new(table: &TraitTable, rels: &'a [TeamRelation]) -> Self {
        let mut synergy: HashMap<(usize, usize), Vec<&TeamRelation>> = HashMap::new();
        let mut counter: HashMap<(usize, usize), Vec<&TeamRelation>> = HashMap::new();
        for r in rels {
            let (Some(a), Some(b)) = (table.index(&r.a), table.index(&r.b)) else {
                continue;
            };
            match r.kind {
                RelationKind::Synergy => synergy.entry((a.min(b), a.max(b))).or_default().push(r),
                RelationKind::Counter => counter.entry((a, b)).or_default().push(r),
            }
        }
        Self { synergy, counter }
    }
}

fn note_reason(component: Component, r: &TeamRelation, with: &str, strength: f32) -> Reason {
    Reason {
        component,
        source: ReasonSource::TeamNote,
        rule_id: None,
        text: r.reason.clone(),
        with: Some(with.to_string()),
        author: Some(r.author_name.clone()),
        note_id: r.id.clone(),
        games: None,
        wins: None,
        strength,
    }
}

fn rule_reason(component: Component, id: &str, text: String, with: &str, strength: f32) -> Reason {
    Reason {
        component,
        source: ReasonSource::Rule,
        rule_id: Some(id.to_string()),
        text,
        with: Some(with.to_string()),
        author: None,
        note_id: None,
        games: None,
        wins: None,
        strength,
    }
}

fn fit_reason(component: Component, id: &str, text: String, strength: f32) -> Reason {
    Reason {
        component,
        source: ReasonSource::Rule,
        rule_id: Some(id.to_string()),
        text,
        with: None,
        author: None,
        note_id: None,
        games: None,
        wins: None,
        strength,
    }
}

fn match_reason(component: Component, text: String, with: &str, t: Tally, strength: f32) -> Reason {
    Reason {
        component,
        source: ReasonSource::MatchData,
        rule_id: None,
        text,
        with: Some(with.to_string()),
        author: None,
        note_id: None,
        games: Some(t.games),
        wins: Some(t.wins),
        strength,
    }
}

/// Evidence corroborates an existing relation: +1 at 60 % or more, −1 at
/// 40 % or less, 0 otherwise or without a relation.
fn corroboration(t: Tally, related: bool) -> f32 {
    if !related || t.games < MIN_SAMPLE {
        0.0
    } else if t.rate() >= 0.6 {
        1.0
    } else if t.rate() <= 0.4 {
        -1.0
    } else {
        0.0
    }
}

struct Ctx<'a> {
    table: &'a TraitTable,
    notes: NoteIndex<'a>,
    k: &'a TeamKnowledge<'a>,
    ours: bool,
}

impl Ctx<'_> {
    fn name(&self, i: usize) -> &str {
        &self.table.rows[i].name
    }
    fn id(&self, i: usize) -> &str {
        &self.table.rows[i].traits.champion
    }

    /// Synergy between candidate `c` and own pick `p`: strength and reasons.
    fn synergy(&self, c: usize, p: usize) -> (f32, Vec<Reason>, Vec<ReasonSource>) {
        let mut reasons = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();
        let mut rule_sum = 0.0;
        for (x, y) in [(c, p), (p, c)] {
            for r in self.table.relations.between(x, y) {
                if r.kind != RelationKind::Synergy || !seen.insert(r.id) {
                    continue;
                }
                rule_sum += r.weight as f32;
                reasons.push(rule_reason(
                    Component::Synergy,
                    r.id,
                    r.sentence(self.name(x), self.name(y)),
                    self.id(p),
                    r.weight as f32,
                ));
            }
        }
        let mut note_sum = 0.0;
        if let Some(list) = self.notes.synergy.get(&(c.min(p), c.max(p))) {
            for n in list {
                note_sum += n.weight as f32;
                reasons.push(note_reason(
                    Component::Synergy,
                    n,
                    self.id(p),
                    n.weight as f32,
                ));
            }
        }
        let base = rule_sum.min(PAIR_CAP) + note_sum.min(PAIR_CAP);
        let mut total = base;
        if let Some(t) = self.k.evidence.pair(self.id(c), self.id(p)) {
            if t.games >= MIN_SAMPLE && self.ours {
                let s = corroboration(t, base > 0.0);
                total += s;
                reasons.push(match_reason(
                    Component::Synergy,
                    format!(
                        "{} and {} together in {} of your games, {} won",
                        self.name(c),
                        self.name(p),
                        t.games,
                        t.wins
                    ),
                    self.id(p),
                    t,
                    s,
                ));
            }
        }
        let mut sources = Vec::new();
        if rule_sum > 0.0 {
            sources.push(ReasonSource::Rule);
        }
        if note_sum > 0.0 {
            sources.push(ReasonSource::TeamNote);
        }
        if total != base {
            sources.push(ReasonSource::MatchData);
        }
        (total, reasons, sources)
    }

    /// Rule and note strength of `x` beating `y`, with reasons (signed by
    /// `sign`, from the candidate's point of view).
    fn beats(&self, x: usize, y: usize, with: usize, sign: f32) -> (f32, f32, Vec<Reason>) {
        let mut reasons = Vec::new();
        let mut rule_sum = 0.0;
        for r in self.table.relations.between(x, y) {
            if r.kind != RelationKind::Counter {
                continue;
            }
            rule_sum += r.weight as f32;
            reasons.push(rule_reason(
                Component::Counter,
                r.id,
                r.sentence(self.name(x), self.name(y)),
                self.id(with),
                sign * r.weight as f32,
            ));
        }
        let mut note_sum = 0.0;
        if let Some(list) = self.notes.counter.get(&(x, y)) {
            for n in list {
                note_sum += n.weight as f32;
                reasons.push(note_reason(
                    Component::Counter,
                    n,
                    self.id(with),
                    sign * n.weight as f32,
                ));
            }
        }
        (rule_sum.min(PAIR_CAP), note_sum.min(PAIR_CAP), reasons)
    }

    /// Counter balance of candidate `c` against enemy pick `e`.
    fn counter(&self, c: usize, e: usize) -> (f32, Vec<Reason>, Vec<ReasonSource>) {
        let (wr, wn, mut reasons) = self.beats(c, e, e, 1.0);
        let (lr, ln, lost) = self.beats(e, c, e, -1.0);
        reasons.extend(lost);
        let base = (wr + wn) - (lr + ln);
        let related = wr + wn + lr + ln > 0.0;
        let mut total = base;
        // Matchup evidence is from our side: our champion against theirs.
        let tally = if self.ours {
            self.k.evidence.matchup(self.id(c), self.id(e))
        } else {
            self.k
                .evidence
                .matchup(self.id(e), self.id(c))
                .map(|t| Tally {
                    games: t.games,
                    wins: t.games - t.wins,
                })
        };
        if let Some(t) = tally.filter(|t| t.games >= MIN_SAMPLE) {
            let s = corroboration(t, related);
            total += s;
            let text = if self.ours {
                format!(
                    "Your {} against {}: {} of {} games won",
                    self.name(c),
                    self.name(e),
                    t.wins,
                    t.games
                )
            } else {
                format!(
                    "Your {} against their {}: {} of {} games lost",
                    self.name(e),
                    self.name(c),
                    t.wins,
                    t.games
                )
            };
            reasons.push(match_reason(Component::Counter, text, self.id(e), t, s));
        }
        let mut sources = Vec::new();
        if wr + lr > 0.0 {
            sources.push(ReasonSource::Rule);
        }
        if wn + ln > 0.0 {
            sources.push(ReasonSource::TeamNote);
        }
        if total != base {
            sources.push(ReasonSource::MatchData);
        }
        (total, reasons, sources)
    }
}

fn empty(reason: &str, notes: GraphNotes) -> GraphResult {
    GraphResult {
        empty_reason: Some(reason.to_string()),
        notes,
        ..GraphResult::default()
    }
}

/// Rank the candidates for the draft's current step.
pub fn rank(table: &TraitTable, state: &DraftState, k: &TeamKnowledge) -> GraphResult {
    let notes = GraphNotes {
        team: k.team,
        relations: k.relations.len() as u32,
        games: k.evidence.games,
        min_sample: MIN_SAMPLE,
        default_rows: table
            .rows
            .iter()
            .filter(|r| r.source == TraitSource::Default)
            .count() as u32,
        low_confidence_rows: table
            .rows
            .iter()
            .filter(|r| r.traits.confidence == crate::models::graph::Confidence::Low)
            .count() as u32,
        scouting: false,
    };
    if table.is_empty() {
        return empty(
            "The trait table is not loaded yet, so there is nothing to rank. Reload the page in a moment.",
            notes,
        );
    }
    let Some(cur) = state.current.filter(|c| *c < 20) else {
        return empty(
            "The draft is complete. Choose a pick on the board to see what else fits there.",
            notes,
        );
    };
    let step = TOURNAMENT_ORDER[cur];
    if step.action == DraftAction::Ban {
        return empty(
            "This step is a ban. The graph ranks picks: choose a pick slot on the board to see candidates.",
            notes,
        );
    }
    let side = step.side;
    let our_side = DraftSide::parse(&state.our_side).unwrap_or(DraftSide::Blue);
    let ours = side == our_side;
    let slot_champ = |i: usize| -> Option<usize> {
        state
            .slots
            .get(i)
            .cloned()
            .flatten()
            .and_then(|n| table.index(&n))
    };
    let slot_role = |i: usize| -> Option<Role> {
        state
            .roles
            .get(i)
            .cloned()
            .flatten()
            .and_then(|r| Role::parse(&r))
    };

    // Locked picks of both sides (the current slot is being decided).
    let mut locked = Vec::new();
    let mut own: Vec<usize> = Vec::new();
    let mut enemy: Vec<usize> = Vec::new();
    let mut taken: Vec<Role> = Vec::new();
    for s in [side, side.other()] {
        for slot in pick_slots(s) {
            if slot == cur {
                continue;
            }
            let Some(i) = slot_champ(slot) else { continue };
            let role = slot_role(slot);
            if s == side {
                own.push(i);
                let r = role.or_else(|| {
                    table.rows[i]
                        .traits
                        .roles
                        .iter()
                        .copied()
                        .find(|r| !taken.contains(r))
                });
                if let Some(r) = r {
                    taken.push(r);
                }
            } else {
                enemy.push(i);
            }
            locked.push(GraphNode {
                champion: table.rows[i].traits.champion.clone(),
                name: table.rows[i].name.clone(),
                side: s.as_str().to_string(),
                slot,
                role,
                own: s == side,
            });
        }
    }

    let (roles, role_chosen) = match state.role {
        Some(r) => (vec![r], true),
        None => {
            let open: Vec<Role> = Role::ALL
                .iter()
                .copied()
                .filter(|r| !taken.contains(r))
                .collect();
            (
                if open.is_empty() {
                    Role::ALL.to_vec()
                } else {
                    open
                },
                false,
            )
        }
    };

    let mut unavailable: HashSet<usize> = (0..20)
        .filter(|i| *i != cur)
        .filter_map(slot_champ)
        .collect();
    unavailable.extend(state.unavailable.iter().filter_map(|n| table.index(n)));

    let own_traits: Vec<_> = own.iter().map(|i| &table.rows[*i].traits).collect();
    let own_profile = CompProfile::sum(own_traits.iter().copied());
    let enemy_profile = CompProfile::sum(enemy.iter().map(|i| &table.rows[*i].traits));
    let n_own = own.len();
    let synergy_scale = PER_PICK_SCALE * n_own.max(1) as f32;
    let counter_scale = PER_PICK_SCALE * enemy.len().max(1) as f32;

    // The comp's plan: the draft's tags first, else the picks' curve.
    let tags: Vec<&str> = {
        let mut t: Vec<&str> = state.tags.iter().map(|s| s.trim()).collect();
        t.sort_unstable();
        t.dedup();
        t
    };
    let plan = if tags.contains(&"scaling") {
        Some(Scaling::Late)
    } else if tags.contains(&"early-game") {
        Some(Scaling::Early)
    } else if n_own >= 2 {
        own_profile.plan()
    } else {
        None
    };
    let has_hard_engage = own_traits.iter().any(|t| t.engage == Engage::Hard);

    // Scouted champions still in play (ours only).
    let scouted: Vec<(&Scouted, usize)> = if ours {
        k.scouted
            .iter()
            .filter_map(|s| table.index(&s.champion).map(|i| (s, i)))
            .filter(|(_, i)| !unavailable.contains(i) && !enemy.contains(i))
            .collect()
    } else {
        Vec::new()
    };

    let cx = Ctx {
        table,
        notes: NoteIndex::new(table, k.relations),
        k,
        ours,
    };

    let mut candidates: Vec<(Candidate, i32, i32)> = Vec::new();
    let mut all_edges: Vec<GraphEdge> = Vec::new();
    for (ci, rec) in table.rows.iter().enumerate() {
        if unavailable.contains(&ci) {
            continue;
        }
        let t = &rec.traits;
        let Some(role) = t.roles.iter().copied().find(|r| roles.contains(r)) else {
            continue;
        };
        let mut reasons: Vec<Reason> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();

        // Synergy with own picks.
        let mut syn = 0.0;
        for &p in &own {
            let (s, r, sources) = cx.synergy(ci, p);
            syn += s;
            reasons.extend(r);
            if s != 0.0 {
                edges.push(GraphEdge {
                    from: t.champion.clone(),
                    to: cx.id(p).to_string(),
                    kind: RelationKind::Synergy,
                    strength: s,
                    sources,
                });
            }
        }

        // Counters against enemy picks, then scouted champions.
        let mut ctr = 0.0;
        for &e in &enemy {
            let (s, r, sources) = cx.counter(ci, e);
            ctr += s;
            reasons.extend(r);
            if s != 0.0 || !sources.is_empty() {
                edges.push(GraphEdge {
                    from: t.champion.clone(),
                    to: cx.id(e).to_string(),
                    kind: RelationKind::Counter,
                    strength: s,
                    sources,
                });
            }
        }
        let mut scout_sum = 0.0;
        for (s, si) in &scouted {
            let (wr, wn, _) = cx.beats(ci, *si, *si, 1.0);
            let strength = (wr + wn) / 3.0;
            if strength <= 0.0 || scout_sum >= SCOUTING_CAP {
                continue;
            }
            let add = strength.min(SCOUTING_CAP - scout_sum);
            scout_sum += add;
            let who = match s.role {
                Some(r) => format!("their {} {}", r.label().to_lowercase(), s.player),
                None => format!("their player {}", s.player),
            };
            reasons.push(Reason {
                component: Component::Counter,
                source: ReasonSource::Scouting,
                rule_id: None,
                text: format!(
                    "Answers {}, which {} has played (opponent dossier)",
                    cx.name(*si),
                    who
                ),
                with: Some(cx.id(*si).to_string()),
                author: None,
                note_id: None,
                games: None,
                wins: None,
                strength: add,
            });
        }
        ctr += scout_sum;

        // Playstyle fit.
        let mut play = 0.0;
        let mut tag_hits = 0;
        for tag in &tags {
            if tag_hits >= 2 {
                break;
            }
            if tag_styles(tag).iter().any(|s| t.has_style(*s)) {
                tag_hits += 1;
                play += 1.0;
                reasons.push(fit_reason(
                    Component::Playstyle,
                    "fit.tag_match",
                    format!("Fits the draft's {} plan", tag.replace('-', " ")),
                    1.0,
                ));
            }
        }
        if n_own >= 2 {
            if !has_hard_engage && t.engage != Engage::Absent {
                let v = if t.engage == Engage::Hard { 1.0 } else { 0.5 };
                play += v;
                reasons.push(fit_reason(
                    Component::Playstyle,
                    "fit.needs_engage",
                    format!(
                        "The comp has no hard engage yet; {} brings {} engage",
                        rec.name,
                        if v >= 1.0 { "hard" } else { "some" }
                    ),
                    v,
                ));
            }
            if own_profile.frontline < 2 && t.frontline != Strength::Absent {
                let v = if t.frontline == Strength::Strong {
                    1.0
                } else {
                    0.5
                };
                play += v;
                reasons.push(fit_reason(
                    Component::Playstyle,
                    "fit.needs_frontline",
                    format!("The comp has little front line; {} adds some", rec.name),
                    v,
                ));
            }
            if (own_profile.wave_clear as usize) < n_own && t.wave_clear == WaveClear::Strong {
                play += 0.5;
                reasons.push(fit_reason(
                    Component::Playstyle,
                    "fit.needs_wave_clear",
                    format!(
                        "The comp clears waves slowly; {} clears them well",
                        rec.name
                    ),
                    0.5,
                ));
            }
            let physical_heavy = own_profile.physical >= 0.75 * n_own as f32;
            let magic_heavy = own_profile.magic >= 0.75 * n_own as f32;
            let is_physical = matches!(t.damage_type, DamageType::Physical | DamageType::TrueHeavy);
            let is_magic = t.damage_type == DamageType::Magic;
            if physical_heavy && is_magic {
                play += 1.0;
                reasons.push(fit_reason(
                    Component::Playstyle,
                    "fit.needs_magic",
                    format!(
                        "Most of the comp's damage is physical; {} adds magic damage",
                        rec.name
                    ),
                    1.0,
                ));
            } else if magic_heavy && is_physical {
                play += 1.0;
                reasons.push(fit_reason(
                    Component::Playstyle,
                    "fit.needs_physical",
                    format!(
                        "Most of the comp's damage is magic; {} adds physical damage",
                        rec.name
                    ),
                    1.0,
                ));
            } else if (physical_heavy && is_physical) || (magic_heavy && is_magic) {
                play -= 1.0;
                reasons.push(fit_reason(
                    Component::Playstyle,
                    "fit.one_sided_damage",
                    format!(
                        "The comp's damage is already mostly {}; {} adds more of it",
                        if physical_heavy { "physical" } else { "magic" },
                        rec.name
                    ),
                    -1.0,
                ));
            }
        }

        // Power spikes.
        let mut timing = 0.0;
        if let Some(p) = plan {
            let word = p.label().to_lowercase();
            if t.scaling == p {
                timing = 1.0;
                reasons.push(fit_reason(
                    Component::Timing,
                    "spike.with_plan",
                    format!(
                        "{} is strongest {} game, like the comp's plan",
                        rec.name, word
                    ),
                    1.0,
                ));
            } else if matches!(
                (t.scaling, p),
                (Scaling::Early, Scaling::Late) | (Scaling::Late, Scaling::Early)
            ) {
                timing = -1.0;
                reasons.push(fit_reason(
                    Component::Timing,
                    "spike.against_plan",
                    format!(
                        "{} is strongest {} game; the comp plans for the {} game",
                        rec.name,
                        t.scaling.label().to_lowercase(),
                        word
                    ),
                    -1.0,
                ));
            }
        }

        // Pool modifier (our side).
        let pool = if ours && k.team {
            let player = k.pools.iter().find(|p| p.role == Some(role));
            Some(match player {
                None => PoolStatus {
                    player: None,
                    tier: None,
                    points: 0,
                },
                Some(p) => {
                    let tier = p
                        .entries
                        .iter()
                        .find(|(id, _)| id == &t.champion)
                        .map(|(_, tier)| tier.clone());
                    let pts = tier_points(tier.as_deref());
                    let text = match &tier {
                        Some(tr) if tr == "comfort" => format!("Comfort pick for {}", p.username),
                        Some(tr) => format!("In {}'s pool: {}", p.username, tier_label(tr)),
                        None => format!("Not in {}'s pool", p.username),
                    };
                    reasons.push(Reason {
                        component: Component::Pool,
                        source: ReasonSource::Pool,
                        rule_id: None,
                        text,
                        with: None,
                        author: Some(p.username.clone()),
                        note_id: None,
                        games: None,
                        wins: None,
                        strength: pts as f32 / 3.0,
                    });
                    PoolStatus {
                        player: Some(p.username.clone()),
                        tier,
                        points: pts,
                    }
                }
            })
        } else {
            None
        };

        let parts = vec![
            FitPart {
                component: Component::Synergy,
                points: points(Component::Synergy, syn / synergy_scale),
                band: Component::Synergy.band(),
            },
            FitPart {
                component: Component::Counter,
                points: points(Component::Counter, ctr / counter_scale),
                band: Component::Counter.band(),
            },
            FitPart {
                component: Component::Playstyle,
                points: points(Component::Playstyle, play / PLAYSTYLE_SCALE),
                band: Component::Playstyle.band(),
            },
            FitPart {
                component: Component::Timing,
                points: points(Component::Timing, timing / TIMING_SCALE),
                band: Component::Timing.band(),
            },
            FitPart {
                component: Component::Pool,
                points: pool.as_ref().map(|p| p.points).unwrap_or(0),
                band: Component::Pool.band(),
            },
        ];
        let fit = (50 + parts.iter().map(|p| p.points).sum::<i32>()).clamp(0, 100) as u8;

        // Strongest first; ties keep the priority order.
        let order = |c: Component| {
            Component::WEIGHTED
                .iter()
                .position(|w| *w == c)
                .unwrap_or(4)
        };
        reasons.sort_by(|a, b| {
            b.strength
                .abs()
                .partial_cmp(&a.strength.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(order(a.component).cmp(&order(b.component)))
        });

        let syn_pts = parts[0].points;
        let ctr_pts = parts[1].points;
        candidates.push((
            Candidate {
                champion: t.champion.clone(),
                name: rec.name.clone(),
                fit,
                role: Some(role),
                role_guessed: !role_chosen,
                parts,
                reasons,
                pool,
                adds: CompProfile::of(t),
                trait_default: rec.source == TraitSource::Default,
                trait_low_confidence: t.confidence == crate::models::graph::Confidence::Low,
            },
            syn_pts,
            ctr_pts,
        ));
        all_edges.extend(edges);
    }

    candidates.sort_by(|(a, asyn, actr), (b, bsyn, bctr)| {
        b.fit
            .cmp(&a.fit)
            .then(bsyn.cmp(asyn))
            .then(bctr.cmp(actr))
            .then(a.name.cmp(&b.name))
    });
    candidates.truncate(MAX_CANDIDATES);
    let kept: HashSet<&str> = candidates.iter().map(|c| c.0.champion.as_str()).collect();
    let edges: Vec<GraphEdge> = all_edges
        .into_iter()
        .filter(|e| kept.contains(e.from.as_str()))
        .collect();

    GraphResult {
        target: Some(GraphTarget {
            slot: cur,
            side: side.as_str().to_string(),
            ours,
            roles,
            role_chosen,
        }),
        empty_reason: None,
        candidates: candidates.into_iter().map(|c| c.0).collect(),
        locked,
        edges,
        own: own_profile,
        enemy: enemy_profile,
        notes: GraphNotes {
            scouting: !scouted.is_empty(),
            ..notes
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::{TraitRecord, TraitSource};
    use crate::server::graph::traits::{parse_defaults, DEFAULTS_CSV};

    fn table() -> TraitTable {
        let rows = parse_defaults(DEFAULTS_CSV)
            .unwrap()
            .into_iter()
            .map(|t| TraitRecord {
                name: match t.champion.as_str() {
                    "MonkeyKing" => "Wukong".into(),
                    "JarvanIV" => "Jarvan IV".into(),
                    other => other.to_string(),
                },
                traits: t,
                source: TraitSource::Default,
                edited_by: None,
                edited_at: None,
                reviewed_for_patch: None,
            })
            .collect();
        TraitTable::new(rows)
    }

    /// Fixture draft: blue (us) has Ornn top and Jinx bot locked, red has
    /// Syndra and Lee Sin; Azir and Zed are banned; blue pick 3 (slot 10)
    /// is being decided.
    fn fixture() -> DraftState {
        let mut slots = vec![None; 20];
        slots[0] = Some("Azir".to_string());
        slots[1] = Some("Zed".to_string());
        slots[6] = Some("Ornn".to_string());
        slots[7] = Some("Syndra".to_string());
        slots[8] = Some("LeeSin".to_string());
        slots[9] = Some("Jinx".to_string());
        let mut roles = vec![None; 20];
        roles[6] = Some("top".to_string());
        roles[9] = Some("bot".to_string());
        DraftState {
            slots,
            roles,
            roles_guessed: vec![false; 20],
            current: Some(10),
            our_side: "blue".into(),
            tags: vec!["teamfight".into()],
            opponent_id: None,
            unavailable: vec![],
            role: None,
        }
    }

    fn nobody() -> MatchEvidence {
        MatchEvidence::default()
    }

    fn knowledge<'a>(
        rels: &'a [TeamRelation],
        pools: &'a [PoolPlayer],
        ev: &'a MatchEvidence,
        scouted: &'a [Scouted],
    ) -> TeamKnowledge<'a> {
        TeamKnowledge {
            team: true,
            relations: rels,
            pools,
            evidence: ev,
            scouted,
        }
    }

    #[test]
    fn ranks_open_roles_and_excludes_taken_champions() {
        let t = table();
        let ev = nobody();
        let r = rank(&t, &fixture(), &knowledge(&[], &[], &ev, &[]));
        let target = r.target.clone().unwrap();
        assert_eq!(target.slot, 10);
        assert!(target.ours);
        assert_eq!(target.roles, vec![Role::Jungle, Role::Mid, Role::Support]);
        assert!(!target.role_chosen);
        assert!(!r.candidates.is_empty());
        for c in &r.candidates {
            assert!(
                !["Azir", "Zed", "Ornn", "Syndra", "LeeSin", "Jinx"].contains(&c.champion.as_str()),
                "{} is on the board",
                c.champion
            );
            let role = c.role.unwrap();
            assert!(target.roles.contains(&role));
            assert!(c.role_guessed);
            assert!(c.fit <= 100);
            let sum: i32 = c.parts.iter().map(|p| p.points).sum();
            assert_eq!(
                c.fit as i32,
                (50 + sum).clamp(0, 100),
                "fit is the sum of its parts"
            );
            for p in &c.parts {
                assert!(
                    p.points.abs() <= p.band,
                    "{:?} within its band",
                    p.component
                );
            }
        }
        // Sorted best first.
        assert!(r.candidates.windows(2).all(|w| w[0].fit >= w[1].fit));
        assert_eq!(r.locked.len(), 4);
        assert_eq!(r.own.picks, 2);
        assert_eq!(r.enemy.picks, 2);
    }

    #[test]
    fn ranking_is_deterministic() {
        let t = table();
        let ev = nobody();
        let a = rank(&t, &fixture(), &knowledge(&[], &[], &ev, &[]));
        let b = rank(&t, &fixture(), &knowledge(&[], &[], &ev, &[]));
        assert_eq!(a, b);
    }

    #[test]
    fn every_point_has_a_reason() {
        let t = table();
        let ev = nobody();
        let r = rank(&t, &fixture(), &knowledge(&[], &[], &ev, &[]));
        for c in &r.candidates {
            for p in &c.parts {
                if p.points != 0 {
                    assert!(
                        c.reasons.iter().any(|x| x.component == p.component),
                        "{}: {:?} has points but no reason",
                        c.champion,
                        p.component
                    );
                }
            }
        }
    }

    #[test]
    fn a_team_note_raises_the_pair_and_shows_its_author() {
        let t = table();
        let ev = nobody();
        let base = rank(&t, &fixture(), &knowledge(&[], &[], &ev, &[]));
        let fit_of = |r: &GraphResult, id: &str| {
            r.candidates
                .iter()
                .find(|c| c.champion == id)
                .map(|c| c.fit)
        };
        let before = fit_of(&base, "Karma").unwrap_or(0);
        let note = TeamRelation {
            id: Some("team_relation:1".into()),
            team_id: "team:t".into(),
            a: "Jinx".into(),
            b: "Karma".into(),
            kind: RelationKind::Synergy,
            weight: 3,
            reason: "Shields keep Jinx shooting".into(),
            author_id: "user:u".into(),
            author_name: "Pell".into(),
            created_at: None,
        };
        let rels = [note];
        let after = rank(&t, &fixture(), &knowledge(&rels, &[], &ev, &[]));
        let karma = after
            .candidates
            .iter()
            .find(|c| c.champion == "Karma")
            .unwrap();
        assert!(karma.fit > before, "{} -> {}", before, karma.fit);
        let r = karma
            .reasons
            .iter()
            .find(|r| r.source == ReasonSource::TeamNote)
            .unwrap();
        assert_eq!(r.author.as_deref(), Some("Pell"));
        assert_eq!(r.text, "Shields keep Jinx shooting");
        assert_eq!(r.with.as_deref(), Some("Jinx"));
        assert!(after.edges.iter().any(|e| e.from == "Karma"
            && e.to == "Jinx"
            && e.sources.contains(&ReasonSource::TeamNote)));
    }

    #[test]
    fn counter_notes_point_the_right_way() {
        let t = table();
        let ev = nobody();
        let note = TeamRelation {
            id: Some("team_relation:2".into()),
            team_id: "team:t".into(),
            a: "Syndra".into(),
            b: "Ahri".into(),
            kind: RelationKind::Counter,
            weight: 3,
            reason: "Syndra stuns Ahri out of every all-in".into(),
            author_id: "user:u".into(),
            author_name: "Kim".into(),
            created_at: None,
        };
        let rels = [note];
        let r = rank(&t, &fixture(), &knowledge(&rels, &[], &ev, &[]));
        let ahri = r.candidates.iter().find(|c| c.champion == "Ahri");
        if let Some(ahri) = ahri {
            let n = ahri
                .reasons
                .iter()
                .find(|x| x.source == ReasonSource::TeamNote)
                .unwrap();
            assert!(n.strength < 0.0, "Syndra beats Ahri: against Ahri");
        } else {
            // Pushed out of the top list entirely, which is also correct.
            assert!(r.candidates.len() == MAX_CANDIDATES);
        }
    }

    #[test]
    fn pool_comfort_and_absence_are_reasons() {
        let t = table();
        let ev = nobody();
        let pools = vec![PoolPlayer {
            username: "Pell".into(),
            role: Some(Role::Support),
            entries: vec![("Thresh".into(), "comfort".into())],
        }];
        let mut s = fixture();
        s.role = Some(Role::Support);
        let r = rank(&t, &s, &knowledge(&[], &pools, &ev, &[]));
        assert!(r.target.as_ref().unwrap().role_chosen);
        let thresh = r
            .candidates
            .iter()
            .find(|c| c.champion == "Thresh")
            .unwrap();
        assert!(!thresh.role_guessed);
        assert_eq!(thresh.pool.as_ref().unwrap().points, 6);
        assert!(thresh
            .reasons
            .iter()
            .any(|x| x.text == "Comfort pick for Pell"));
        let other = r
            .candidates
            .iter()
            .find(|c| c.champion != "Thresh")
            .unwrap();
        assert_eq!(other.pool.as_ref().unwrap().points, -8);
        assert!(other.reasons.iter().any(|x| x.text == "Not in Pell's pool"));
        for c in &r.candidates {
            assert_eq!(c.role, Some(Role::Support));
        }
    }

    #[test]
    fn match_evidence_needs_a_sample_and_a_relation() {
        let t = table();
        let mut ev = MatchEvidence::default();
        // Four games: below the minimum, not shown.
        ev.pairs.insert(
            ("Jinx".into(), "Thresh".into()),
            Tally { games: 4, wins: 4 },
        );
        let mut s = fixture();
        s.role = Some(Role::Support);
        let r = rank(&t, &s, &knowledge(&[], &[], &ev, &[]));
        let thresh = r
            .candidates
            .iter()
            .find(|c| c.champion == "Thresh")
            .unwrap();
        assert!(!thresh
            .reasons
            .iter()
            .any(|x| x.source == ReasonSource::MatchData));
        // Seven games: shown with n, and corroborates the rule relation.
        ev.pairs.insert(
            ("Jinx".into(), "Thresh".into()),
            Tally { games: 7, wins: 5 },
        );
        let r2 = rank(&t, &s, &knowledge(&[], &[], &ev, &[]));
        let thresh2 = r2
            .candidates
            .iter()
            .find(|c| c.champion == "Thresh")
            .unwrap();
        let m = thresh2
            .reasons
            .iter()
            .find(|x| x.source == ReasonSource::MatchData)
            .unwrap();
        assert_eq!((m.games, m.wins), (Some(7), Some(5)));
        assert!(m.text.contains("7 of your games"));
        assert!(thresh2.fit >= thresh.fit);
    }

    #[test]
    fn bans_and_complete_drafts_have_no_target() {
        let t = table();
        let ev = nobody();
        let mut s = fixture();
        s.current = Some(13);
        let r = rank(&t, &s, &knowledge(&[], &[], &ev, &[]));
        assert!(r.target.is_none());
        assert!(r.empty_reason.unwrap().contains("ban"));
        s.current = None;
        let r = rank(&t, &s, &knowledge(&[], &[], &ev, &[]));
        assert!(r.target.is_none());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn enemy_steps_rank_from_their_side_without_pools() {
        let t = table();
        let ev = nobody();
        let pools = vec![PoolPlayer {
            username: "Pell".into(),
            role: Some(Role::Support),
            entries: vec![],
        }];
        let mut s = fixture();
        s.current = Some(11); // red pick 3
        let r = rank(&t, &s, &knowledge(&[], &pools, &ev, &[]));
        let target = r.target.unwrap();
        assert!(!target.ours);
        assert_eq!(target.side, "red");
        assert!(r.candidates.iter().all(|c| c.pool.is_none()));
        assert_eq!(r.own.picks, 2, "red's Syndra and Lee Sin");
    }

    #[test]
    fn scouted_champions_count_as_counter_evidence() {
        let t = table();
        let ev = nobody();
        let scouted = vec![Scouted {
            player: "Nova".into(),
            role: Some(Role::Support),
            champion: "Malphite".into(),
        }];
        let r = rank(&t, &fixture(), &knowledge(&[], &[], &ev, &scouted));
        assert!(r.notes.scouting);
        let janna = r.candidates.iter().find(|c| c.champion == "Janna").unwrap();
        assert!(janna
            .reasons
            .iter()
            .any(|x| x.source == ReasonSource::Scouting && x.text.contains("Malphite")));
    }

    /// Budget: under 50 ms server-side for the whole table. The compute part
    /// is measured here (debug build); release is several times faster.
    #[test]
    fn ranking_the_whole_table_is_fast() {
        let t = table();
        let ev = nobody();
        let rels: Vec<TeamRelation> = (0..40)
            .map(|i| TeamRelation {
                id: Some(format!("team_relation:{i}")),
                team_id: "team:t".into(),
                a: t.rows[i].traits.champion.clone(),
                b: t.rows[i + 1].traits.champion.clone(),
                kind: if i % 2 == 0 {
                    RelationKind::Synergy
                } else {
                    RelationKind::Counter
                },
                weight: 2,
                reason: "fixture".into(),
                author_id: "user:u".into(),
                author_name: "u".into(),
                created_at: None,
            })
            .collect();
        let mut s = fixture();
        s.current = Some(17);
        s.slots[10] = Some("Thresh".into());
        s.slots[11] = Some("Ahri".into());
        s.slots[16] = Some("Rell".into());
        let started = std::time::Instant::now();
        let mut n = 0;
        for _ in 0..10 {
            n += rank(&t, &s, &knowledge(&rels, &[], &ev, &[]))
                .candidates
                .len();
        }
        let per = started.elapsed() / 10;
        eprintln!("rank: {per:?} per ranking ({} rows)", t.len());
        assert!(n > 0);
        assert!(per.as_millis() < 50, "one ranking took {per:?}");
    }
}

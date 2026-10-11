//! Draft Graph model (M4), shared by the server and the browser.
//!
//! - **Trait vocabulary**: small enums for what a champion's kit does
//!   structurally (damage type, range, engage, crowd control ...). Stored as
//!   their `key()` strings in `champion_trait` and in the checked-in
//!   defaults file `src/server/graph/champion_traits.csv`.
//! - **[`ChampionTraits`]** / **[`TraitRecord`]**: one row per champion, with
//!   where it came from (shipped default or the maintainer's edit).
//! - **[`TeamRelation`]**: what a team knows about a pair ("X + Y works
//!   because ...", "X loses to Y because ...").
//! - **[`DraftState`] → [`GraphResult`]**: the input and output of the
//!   `graph_candidates` server function. The result carries everything both
//!   twins draw: the ranked list (stage A) and the force-directed canvas
//!   (stage B): candidates with fit, parts and reasons, the locked picks as
//!   pinned nodes, and the edges between candidates and locked picks.
//!
//! The word is **fit** (0 to 100) and it is always explained; nothing here is
//! a win probability.

use serde::{Deserialize, Serialize};

pub use crate::components::ui::Role;

/// Declares a vocabulary enum whose variants serialise as their stored key,
/// with `ALL`, `key()`, `label()` and `parse()`.
macro_rules! vocab {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $var:ident => $key:literal, $label:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum $name {
            $($(#[$vm])* #[serde(rename = $key)] $var),+
        }

        impl $name {
            /// Every value, in scale order.
            pub const ALL: &'static [$name] = &[$($name::$var),+];
            /// The stored spelling.
            pub fn key(self) -> &'static str {
                match self { $($name::$var => $key),+ }
            }
            /// The word shown in the UI.
            pub fn label(self) -> &'static str {
                match self { $($name::$var => $label),+ }
            }
            /// Parse the stored spelling (exact, after trimming).
            pub fn parse(s: &str) -> Option<Self> {
                match s.trim() {
                    $($key => Some($name::$var),)+
                    _ => None,
                }
            }
        }
    };
}

vocab! {
    /// What the champion's damage mostly is. True-heavy: a large share
    /// ignores resistances (still built like physical for the AD/AP mix).
    DamageType {
        Physical => "physical", "Physical",
        Magic => "magic", "Magic",
        Mixed => "mixed", "Mixed",
        TrueHeavy => "true_heavy", "True-heavy",
    }
}

vocab! {
    /// How the damage arrives.
    DamageProfile {
        Burst => "burst", "Burst",
        Sustained => "sustained", "Sustained",
        Poke => "poke", "Poke",
        Hybrid => "hybrid", "Hybrid",
    }
}

vocab! {
    /// Melee; ranged under 525 or a mixed kit; 525 and up or artillery.
    RangeClass {
        Melee => "melee", "Melee",
        Short => "short", "Short",
        Long => "long", "Long",
    }
}

vocab! {
    /// Can the champion start a fight for the team?
    Engage {
        Absent => "none", "None",
        Conditional => "conditional", "Conditional",
        Hard => "hard", "Hard",
    }
}

vocab! {
    /// A three-step scale for disengage, peel and frontline.
    Strength {
        Absent => "none", "None",
        Partial => "some", "Some",
        Strong => "strong", "Strong",
    }
}

vocab! {
    /// Crowd control the kit brings.
    CrowdControl {
        Absent => "none", "None",
        Light => "light", "Light",
        Medium => "medium", "Medium",
        Heavy => "heavy", "Heavy",
    }
}

vocab! {
    /// How well the champion clears minion waves.
    WaveClear {
        Weak => "weak", "Weak",
        Fair => "ok", "OK",
        Strong => "strong", "Strong",
    }
}

vocab! {
    /// When the kit is strongest.
    Scaling {
        Early => "early", "Early",
        Mid => "mid", "Mid",
        Late => "late", "Late",
        Flat => "flat", "Flat",
    }
}

vocab! {
    /// How freely the champion moves (dashes, blinks, speed).
    Mobility {
        Low => "low", "Low",
        Medium => "medium", "Medium",
        High => "high", "High",
    }
}

vocab! {
    /// Fixed playstyle vocabulary.
    Playstyle {
        Poke => "poke", "Poke",
        Dive => "dive", "Dive",
        Pick => "pick", "Pick",
        SplitPush => "split_push", "Split-push",
        Teamfight => "teamfight", "Teamfight",
        ProtectTheCarry => "protect_the_carry", "Protect the carry",
        Skirmish => "skirmish", "Skirmish",
        Siege => "siege", "Siege",
        FrontToBack => "front_to_back", "Front to back",
        Wombo => "wombo", "Wombo",
    }
}

vocab! {
    /// How sure the row is. Low: neutral values where the kit was unclear.
    Confidence {
        Normal => "normal", "Normal",
        Low => "low", "Low",
    }
}

vocab! {
    /// Where a trait row came from.
    TraitSource {
        Default => "default", "Default",
        Maintainer => "maintainer", "Edited",
    }
}

impl Engage {
    /// 0, 1 or 2 (for profiles).
    pub fn level(self) -> u8 {
        match self {
            Engage::Absent => 0,
            Engage::Conditional => 1,
            Engage::Hard => 2,
        }
    }
}

impl Strength {
    /// 0, 1 or 2.
    pub fn level(self) -> u8 {
        match self {
            Strength::Absent => 0,
            Strength::Partial => 1,
            Strength::Strong => 2,
        }
    }
}

impl CrowdControl {
    /// 0 to 3.
    pub fn level(self) -> u8 {
        match self {
            CrowdControl::Absent => 0,
            CrowdControl::Light => 1,
            CrowdControl::Medium => 2,
            CrowdControl::Heavy => 3,
        }
    }
}

impl WaveClear {
    /// 0, 1 or 2.
    pub fn level(self) -> u8 {
        match self {
            WaveClear::Weak => 0,
            WaveClear::Fair => 1,
            WaveClear::Strong => 2,
        }
    }
}

impl Mobility {
    /// 0, 1 or 2.
    pub fn level(self) -> u8 {
        match self {
            Mobility::Low => 0,
            Mobility::Medium => 1,
            Mobility::High => 2,
        }
    }
}

/// What a champion's kit does, one row per Data Dragon champion id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChampionTraits {
    /// Data Dragon champion id ("MonkeyKing", "KSante").
    pub champion: String,
    /// Roles played, most played first (at least one).
    pub roles: Vec<Role>,
    pub damage_type: DamageType,
    pub damage_profile: DamageProfile,
    pub range: RangeClass,
    pub engage: Engage,
    pub disengage: Strength,
    pub peel: Strength,
    pub frontline: Strength,
    pub cc: CrowdControl,
    pub wave_clear: WaveClear,
    pub scaling: Scaling,
    pub mobility: Mobility,
    pub styles: Vec<Playstyle>,
    pub confidence: Confidence,
}

impl ChampionTraits {
    /// A row with the neutral value in every field and `confidence: low`:
    /// what a champion missing from the defaults file gets.
    pub fn neutral(champion: impl Into<String>, roles: Vec<Role>) -> Self {
        Self {
            champion: champion.into(),
            roles,
            damage_type: DamageType::Mixed,
            damage_profile: DamageProfile::Hybrid,
            range: RangeClass::Melee,
            engage: Engage::Absent,
            disengage: Strength::Absent,
            peel: Strength::Absent,
            frontline: Strength::Absent,
            cc: CrowdControl::Absent,
            wave_clear: WaveClear::Fair,
            scaling: Scaling::Mid,
            mobility: Mobility::Medium,
            styles: Vec::new(),
            confidence: Confidence::Low,
        }
    }

    pub fn has_style(&self, s: Playstyle) -> bool {
        self.styles.contains(&s)
    }

    pub fn plays(&self, r: Role) -> bool {
        self.roles.contains(&r)
    }

    /// Rules a trait row must follow beyond its types: an id, at least one
    /// role, no duplicate roles or styles. Returns the first problem.
    pub fn validate(&self) -> Result<(), String> {
        if self.champion.trim().is_empty()
            || !self.champion.chars().all(|c| c.is_ascii_alphanumeric())
        {
            return Err(format!("'{}' is not a champion id", self.champion));
        }
        if self.roles.is_empty() {
            return Err(format!("{} needs at least one role", self.champion));
        }
        let mut roles = self.roles.clone();
        roles.sort();
        roles.dedup();
        if roles.len() != self.roles.len() {
            return Err(format!("{} lists a role twice", self.champion));
        }
        let mut styles = self.styles.clone();
        styles.sort();
        styles.dedup();
        if styles.len() != self.styles.len() {
            return Err(format!("{} lists a playstyle twice", self.champion));
        }
        Ok(())
    }
}

/// A trait row with its provenance, as the admin page and the graph see it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraitRecord {
    pub traits: ChampionTraits,
    /// Display name from Data Dragon ("Wukong"); the id when unknown.
    pub name: String,
    pub source: TraitSource,
    /// Username of the maintainer who last edited it.
    pub edited_by: Option<String>,
    /// RFC 3339.
    pub edited_at: Option<String>,
    /// Patch the maintainer checked the row against ("16.20.1").
    pub reviewed_for_patch: Option<String>,
}

// ---------------------------------------------------------------------------
// Team relations
// ---------------------------------------------------------------------------

/// A relation between two champions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationKind {
    /// Good together (unordered).
    Synergy,
    /// `a` beats `b`.
    Counter,
}

impl RelationKind {
    pub fn key(self) -> &'static str {
        match self {
            RelationKind::Synergy => "synergy",
            RelationKind::Counter => "counter",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "synergy" => Some(RelationKind::Synergy),
            "counter" => Some(RelationKind::Counter),
            _ => None,
        }
    }
}

/// What the team knows about a pair, entered on the draft page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TeamRelation {
    pub id: Option<String>,
    pub team_id: String,
    /// Champion id. For a counter, the one that wins.
    pub a: String,
    /// Champion id. For a counter, the one that loses.
    pub b: String,
    pub kind: RelationKind,
    /// 1 (slight) to 3 (strong).
    pub weight: u8,
    pub reason: String,
    pub author_id: String,
    pub author_name: String,
    pub created_at: Option<String>,
}

/// Longest reason a team note may carry.
pub const RELATION_REASON_MAX: usize = 200;

/// A new relation as the client sends it (team and author come from the
/// session on the server).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewRelation {
    pub a: String,
    pub b: String,
    pub kind: RelationKind,
    pub weight: u8,
    pub reason: String,
}

impl NewRelation {
    /// Shape checks before any lookup: two different champions, weight 1 to
    /// 3, a reason of 3 to 200 characters. Synergies are stored with `a`
    /// before `b` so a pair has one spelling.
    pub fn normalised(mut self) -> Result<Self, String> {
        self.a = self.a.trim().to_string();
        self.b = self.b.trim().to_string();
        self.reason = self.reason.trim().to_string();
        if self.a.is_empty() || self.b.is_empty() {
            return Err("Choose both champions.".into());
        }
        if self.a == self.b {
            return Err("Choose two different champions.".into());
        }
        if !(1..=3).contains(&self.weight) {
            return Err("Weight must be 1, 2 or 3.".into());
        }
        let len = self.reason.chars().count();
        if len < 3 {
            return Err("Say why, in a few words.".into());
        }
        if len > RELATION_REASON_MAX {
            return Err(format!(
                "Keep the reason under {RELATION_REASON_MAX} characters."
            ));
        }
        if self.kind == RelationKind::Synergy && self.a > self.b {
            std::mem::swap(&mut self.a, &mut self.b);
        }
        Ok(self)
    }
}

// ---------------------------------------------------------------------------
// Graph input
// ---------------------------------------------------------------------------

/// The draft as the graph needs it, sent by the draft page.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DraftState {
    /// Champion per slot (display names or ids), index = order - 1.
    pub slots: Vec<Option<String>>,
    /// Role per slot, any stored spelling.
    pub roles: Vec<Option<String>>,
    /// Per slot: the role was guessed, not set by a person.
    pub roles_guessed: Vec<bool>,
    /// The step being decided (0-based), `None` when complete.
    pub current: Option<usize>,
    /// "blue" or "red".
    pub our_side: String,
    /// The draft's composition tags ("teamfight", "poke", ...).
    pub tags: Vec<String>,
    /// The opponent dossier id, when one is linked.
    pub opponent_id: Option<String>,
    /// Champions that cannot be picked besides the board (fearless).
    pub unavailable: Vec<String>,
    /// A role the user chose for the current slot; `None` = infer it.
    pub role: Option<Role>,
}

// ---------------------------------------------------------------------------
// Graph output
// ---------------------------------------------------------------------------

/// The parts of fit, in the contract's priority order, plus the pool modifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Component {
    Synergy,
    Counter,
    Playstyle,
    Timing,
    Pool,
}

impl Component {
    /// The four weighted parts, in priority order.
    pub const WEIGHTED: [Component; 4] = [
        Component::Synergy,
        Component::Counter,
        Component::Playstyle,
        Component::Timing,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Component::Synergy => "Synergy",
            Component::Counter => "Counters",
            Component::Playstyle => "Playstyle fit",
            Component::Timing => "Power spikes",
            Component::Pool => "Player pool",
        }
    }

    /// Weight in percent (sums to 100 over the four weighted parts). Each
    /// part moves fit by at most half its weight in points, either way,
    /// around the neutral 50: synergy ±20, counters ±15, playstyle ±10,
    /// power spikes ±5. The pool modifier is added after, in points.
    pub fn weight(self) -> u8 {
        match self {
            Component::Synergy => 40,
            Component::Counter => 30,
            Component::Playstyle => 20,
            Component::Timing => 10,
            Component::Pool => 0,
        }
    }

    /// The most points the part can add or take away.
    pub fn band(self) -> i32 {
        match self {
            Component::Pool => 8,
            c => c.weight() as i32 / 2,
        }
    }
}

/// Where a reason comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonSource {
    /// A rule over the trait table (`src/server/graph/rules.rs`).
    Rule,
    /// A team note on the pair.
    TeamNote,
    /// The team's own stored games.
    MatchData,
    /// The slot player's champion pool.
    Pool,
    /// The opponent dossier's scouted champions.
    Scouting,
}

impl ReasonSource {
    pub fn label(self) -> &'static str {
        match self {
            ReasonSource::Rule => "Trait rules",
            ReasonSource::TeamNote => "Team notes",
            ReasonSource::MatchData => "Your games",
            ReasonSource::Pool => "Player pool",
            ReasonSource::Scouting => "Scouting",
        }
    }
}

/// One sentence of explanation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reason {
    pub component: Component,
    pub source: ReasonSource,
    /// Rule id for rule reasons ("syn.engage_burst").
    pub rule_id: Option<String>,
    /// The sentence a coach reads ("Malphite starts fights that Syndra's
    /// burst finishes"). For a team note: the note's reason as written.
    pub text: String,
    /// The other champion of the pair (id), when there is one.
    pub with: Option<String>,
    /// Team note: who wrote it, and the note's id (for "remove").
    pub author: Option<String>,
    pub note_id: Option<String>,
    /// Match data: games and wins behind the number.
    pub games: Option<u32>,
    pub wins: Option<u32>,
    /// Signed strength on the relation scale (1 to 3 per relation;
    /// negative works against the candidate).
    pub strength: f32,
}

/// One part's contribution to fit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FitPart {
    pub component: Component,
    /// Signed points added to the neutral 50.
    pub points: i32,
    /// The most this part could add or take away.
    pub band: i32,
}

/// Pool standing of the slot's player for a candidate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PoolStatus {
    /// The player who would play it (`None`: nobody on the roster has the role).
    pub player: Option<String>,
    /// Pool tier ("comfort", "match_ready", ...), `None` = not in the pool.
    pub tier: Option<String>,
    /// Points the pool adds or takes away.
    pub points: i32,
}

/// What a side's locked picks bring, summed over its champions. Each
/// champion adds 0–2 to engage, frontline, peel and wave clear, 0–3 to
/// crowd control, and one to its scaling curve (flat adds none).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompProfile {
    pub picks: u8,
    /// Physical share (mixed counts half to each side).
    pub physical: f32,
    pub magic: f32,
    pub cc: u8,
    pub engage: u8,
    pub frontline: u8,
    pub peel: u8,
    pub wave_clear: u8,
    pub early: u8,
    pub mid: u8,
    pub late: u8,
}

impl CompProfile {
    /// One champion's contribution.
    pub fn of(t: &ChampionTraits) -> Self {
        let (physical, magic) = match t.damage_type {
            DamageType::Physical | DamageType::TrueHeavy => (1.0, 0.0),
            DamageType::Magic => (0.0, 1.0),
            DamageType::Mixed => (0.5, 0.5),
        };
        Self {
            picks: 1,
            physical,
            magic,
            cc: t.cc.level(),
            engage: t.engage.level(),
            frontline: t.frontline.level(),
            peel: t.peel.level(),
            wave_clear: t.wave_clear.level(),
            early: u8::from(t.scaling == Scaling::Early),
            mid: u8::from(t.scaling == Scaling::Mid),
            late: u8::from(t.scaling == Scaling::Late),
        }
    }

    /// The profile of several champions.
    pub fn sum<'a>(traits: impl IntoIterator<Item = &'a ChampionTraits>) -> Self {
        traits
            .into_iter()
            .fold(Self::default(), |acc, t| acc.plus(&Self::of(t)))
    }

    /// Two profiles added ("the comp if Rakan is picked").
    pub fn plus(&self, o: &CompProfile) -> Self {
        Self {
            picks: self.picks + o.picks,
            physical: self.physical + o.physical,
            magic: self.magic + o.magic,
            cc: self.cc + o.cc,
            engage: self.engage + o.engage,
            frontline: self.frontline + o.frontline,
            peel: self.peel + o.peel,
            wave_clear: self.wave_clear + o.wave_clear,
            early: self.early + o.early,
            mid: self.mid + o.mid,
            late: self.late + o.late,
        }
    }

    /// The curve most picks share: Early, Mid, Late, or `None` when no pick
    /// has a curve or two curves tie.
    pub fn plan(&self) -> Option<Scaling> {
        let mut best = [
            (self.early, Scaling::Early),
            (self.mid, Scaling::Mid),
            (self.late, Scaling::Late),
        ];
        best.sort_by_key(|b| std::cmp::Reverse(b.0));
        (best[0].0 > 0 && best[0].0 > best[1].0).then_some(best[0].1)
    }

    /// "3 / 1.5": physical and magic shares, halves kept.
    pub fn damage_mix(&self) -> String {
        format!("{} / {}", half(self.physical), half(self.magic))
    }
}

fn half(v: f32) -> String {
    if (v - v.round()).abs() < 0.01 {
        format!("{}", v.round() as i32)
    } else {
        format!("{v:.1}")
    }
}

/// A champion the graph ranks for the current slot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    /// Data Dragon id.
    pub champion: String,
    pub name: String,
    /// 0 to 100.
    pub fit: u8,
    /// The role it would take; guessed unless the user chose the role.
    pub role: Option<Role>,
    pub role_guessed: bool,
    /// The four weighted parts plus the pool modifier, in priority order.
    pub parts: Vec<FitPart>,
    /// Every reason, strongest first.
    pub reasons: Vec<Reason>,
    /// Our side only.
    pub pool: Option<PoolStatus>,
    /// What it adds to its side's profile.
    pub adds: CompProfile,
    /// Its trait row is still the shipped default.
    pub trait_default: bool,
    /// Its trait row is marked low confidence.
    pub trait_low_confidence: bool,
}

/// A locked pick: a pinned node on the canvas (stage B).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    pub champion: String,
    pub name: String,
    /// "blue" or "red".
    pub side: String,
    /// Board slot (0-based).
    pub slot: usize,
    pub role: Option<Role>,
    /// True for the side the graph ranks for.
    pub own: bool,
}

/// A relation between a candidate and a locked pick (stage B's edges).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Candidate id.
    pub from: String,
    /// Locked pick id.
    pub to: String,
    pub kind: RelationKind,
    /// Signed sum on the relation scale. Synergy: positive helps. Counter:
    /// positive = the candidate beats the pick, negative = the pick beats
    /// the candidate.
    pub strength: f32,
    pub sources: Vec<ReasonSource>,
}

impl GraphEdge {
    /// Weak relations (under 1.5) are drawn dotted on the canvas.
    pub fn is_weak(&self) -> bool {
        self.strength.abs() < 1.5
    }
}

/// The slot the graph ranks for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphTarget {
    pub slot: usize,
    /// "blue" or "red".
    pub side: String,
    /// Whether the slot is our team's.
    pub ours: bool,
    /// Roles still open for that side (or the one the user chose).
    pub roles: Vec<Role>,
    /// The user chose the role (not guessed).
    pub role_chosen: bool,
}

/// What the result stands on, said under the list.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphNotes {
    /// The user has a team (team notes, pools and games count).
    pub team: bool,
    /// Team notes considered.
    pub relations: u32,
    /// Stored team games considered for match evidence.
    pub games: u32,
    /// Minimum games before match evidence is shown.
    pub min_sample: u32,
    /// Trait rows still at their shipped default.
    pub default_rows: u32,
    /// Trait rows marked low confidence.
    pub low_confidence_rows: u32,
    /// The opponent dossier's scouted champions were used.
    pub scouting: bool,
}

/// The output of `graph_candidates`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphResult {
    /// `None` when there is nothing to rank (ban step, complete draft).
    pub target: Option<GraphTarget>,
    /// Why there is no target, in words.
    pub empty_reason: Option<String>,
    /// Ranked, best first; at most [`MAX_CANDIDATES`].
    pub candidates: Vec<Candidate>,
    /// Locked picks of both sides.
    pub locked: Vec<GraphNode>,
    /// Relations between the candidates and the locked picks.
    pub edges: Vec<GraphEdge>,
    /// Profiles of the ranked side and the other side.
    pub own: CompProfile,
    pub enemy: CompProfile,
    pub notes: GraphNotes,
}

/// A champion-pool tier in words ("match_ready" -> "match ready").
pub fn tier_label(tier: &str) -> &'static str {
    match tier {
        "comfort" => "comfort",
        "match_ready" => "match ready",
        "scrim_ready" => "scrim ready",
        "practicing" => "practicing",
        "to_practice" => "to practise",
        _ => "in the pool",
    }
}

/// Candidates returned (the canvas draws at most 24 of them).
pub const MAX_CANDIDATES: usize = 30;
/// Candidates the canvas may draw (stage B).
pub const CANVAS_CANDIDATES: usize = 24;
/// Games below which match evidence is not shown.
pub const MIN_SAMPLE: u32 = 5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocab_keys_round_trip_through_serde_and_parse() {
        for v in Strength::ALL {
            let json = serde_json::to_string(v).unwrap();
            assert_eq!(json, format!("\"{}\"", v.key()));
            assert_eq!(Strength::parse(v.key()), Some(*v));
        }
        assert_eq!(Strength::Partial.key(), "some");
        assert_eq!(WaveClear::parse("ok"), Some(WaveClear::Fair));
        assert_eq!(Playstyle::parse("split_push"), Some(Playstyle::SplitPush));
        assert_eq!(Engage::parse("nope"), None);
        assert_eq!(
            serde_json::to_string(&Role::Support).unwrap(),
            "\"support\""
        );
    }

    #[test]
    fn traits_validate_roles_and_styles() {
        let mut t = ChampionTraits::neutral("Ahri", vec![Role::Mid]);
        assert!(t.validate().is_ok());
        t.roles.clear();
        assert!(t.validate().is_err());
        t.roles = vec![Role::Mid, Role::Mid];
        assert!(t.validate().is_err());
        t.roles = vec![Role::Mid];
        t.styles = vec![Playstyle::Pick, Playstyle::Pick];
        assert!(t.validate().is_err());
        t.styles.clear();
        t.champion = "Ah ri".into();
        assert!(t.validate().is_err());
    }

    #[test]
    fn new_relation_normalises_synergy_order_and_checks_shape() {
        let r = NewRelation {
            a: "Zed".into(),
            b: "Ahri".into(),
            kind: RelationKind::Synergy,
            weight: 2,
            reason: "  dive together  ".into(),
        }
        .normalised()
        .unwrap();
        assert_eq!((r.a.as_str(), r.b.as_str()), ("Ahri", "Zed"));
        assert_eq!(r.reason, "dive together");
        // Counters keep their direction.
        let c = NewRelation {
            a: "Zed".into(),
            b: "Ahri".into(),
            kind: RelationKind::Counter,
            weight: 1,
            reason: "kills her".into(),
        }
        .normalised()
        .unwrap();
        assert_eq!(c.a, "Zed");
        let bad = |a: &str, b: &str, w: u8, why: &str| {
            NewRelation {
                a: a.into(),
                b: b.into(),
                kind: RelationKind::Synergy,
                weight: w,
                reason: why.into(),
            }
            .normalised()
            .is_err()
        };
        assert!(bad("Ahri", "Ahri", 1, "same"));
        assert!(bad("Ahri", "Zed", 0, "weight"));
        assert!(bad("Ahri", "Zed", 4, "weight"));
        assert!(bad("Ahri", "Zed", 2, "x"));
        assert!(bad("Ahri", "", 2, "missing"));
        assert!(bad("Ahri", "Zed", 2, &"a".repeat(201)));
    }

    #[test]
    fn profiles_add_up() {
        let mut a = ChampionTraits::neutral("A", vec![Role::Top]);
        a.damage_type = DamageType::Physical;
        a.engage = Engage::Hard;
        a.cc = CrowdControl::Heavy;
        a.scaling = Scaling::Early;
        let mut b = ChampionTraits::neutral("B", vec![Role::Mid]);
        b.damage_type = DamageType::Mixed;
        b.wave_clear = WaveClear::Strong;
        b.scaling = Scaling::Late;
        let p = CompProfile::sum([&a, &b]);
        assert_eq!(p.picks, 2);
        assert_eq!(p.damage_mix(), "1.5 / 0.5");
        assert_eq!((p.engage, p.cc, p.wave_clear), (2, 3, 3));
        assert_eq!(p.plan(), None, "one early and one late tie");
        let c = p.plus(&CompProfile::of(&b));
        assert_eq!(c.plan(), Some(Scaling::Late));
        assert_eq!(c.damage_mix(), "2 / 1");
    }

    #[test]
    fn weights_follow_the_priority_order() {
        let w: Vec<u8> = Component::WEIGHTED.iter().map(|c| c.weight()).collect();
        assert_eq!(w.iter().map(|x| *x as u32).sum::<u32>(), 100);
        assert!(
            w.windows(2).all(|p| p[0] > p[1]),
            "synergy > counters > playstyle > timing"
        );
        assert_eq!(Component::Synergy.band(), 20);
    }
}

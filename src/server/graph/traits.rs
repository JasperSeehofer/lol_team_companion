//! The champion trait table: the checked-in defaults, the database copy
//! (`champion_trait`), and the process-wide in-memory table the graph reads.
//!
//! Lifecycle:
//! 1. `champion_traits.csv` (next to this file) is compiled in. It is the
//!    maintainer's starting opinion, one kit-based row per champion.
//! 2. At startup and after each Data Dragon refresh, [`sync`] writes rows
//!    that are missing, and refreshes rows that are still `default` when the
//!    file changed. A champion Data Dragon knows but the file does not gets
//!    a neutral row with `confidence: low` (logged). Rows the maintainer
//!    edited (`source: maintainer`) are never touched.
//! 3. [`install`] builds the in-memory [`TraitTable`] (lookups by id and by
//!    display name, plus the trait-rule relations of every pair, computed
//!    once). An admin edit reloads it.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use surrealdb::{engine::local::Db, Surreal};

use crate::models::draft::guess_role_from_tags;
use crate::models::graph::{
    ChampionTraits, Confidence, CrowdControl, DamageProfile, DamageType, Engage, Mobility,
    Playstyle, RangeClass, Role, Scaling, Strength, TraitRecord, TraitSource, WaveClear,
};
use crate::server::db::{self, DbResult};
use crate::server::graph::rules::PairRelations;

/// The defaults file, compiled in.
pub const DEFAULTS_CSV: &str = include_str!("champion_traits.csv");

/// Column order of the defaults file.
pub const COLUMNS: [&str; 15] = [
    "champion",
    "roles",
    "damage_type",
    "damage_profile",
    "range",
    "engage",
    "disengage",
    "peel",
    "frontline",
    "cc",
    "wave_clear",
    "scaling",
    "mobility",
    "styles",
    "confidence",
];

fn field<T>(
    line: usize,
    name: &str,
    raw: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Result<T, String> {
    parse(raw).ok_or_else(|| format!("line {line}: '{raw}' is not a valid {name}"))
}

fn list<T>(
    line: usize,
    name: &str,
    raw: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Result<Vec<T>, String> {
    raw.split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| field(line, name, s, &parse))
        .collect()
}

fn parse_role(s: &str) -> Option<Role> {
    // Only the stored spellings, so the file stays canonical.
    Role::ALL.iter().copied().find(|r| r.key() == s)
}

/// Parse a defaults document: `#` comment lines, one header line naming
/// [`COLUMNS`], then one row per champion. Every value must be in the
/// vocabulary and every row must pass [`ChampionTraits::validate`].
pub fn parse_defaults(csv: &str) -> Result<Vec<ChampionTraits>, String> {
    let mut rows = Vec::new();
    let mut header_seen = false;
    for (i, raw) in csv.lines().enumerate() {
        let line = i + 1;
        let text = raw.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let cells: Vec<&str> = text.split(',').map(str::trim).collect();
        if !header_seen {
            if cells != COLUMNS {
                return Err(format!("line {line}: header must be {}", COLUMNS.join(",")));
            }
            header_seen = true;
            continue;
        }
        if cells.len() != COLUMNS.len() {
            return Err(format!(
                "line {line}: expected {} columns, found {}",
                COLUMNS.len(),
                cells.len()
            ));
        }
        let t = ChampionTraits {
            champion: cells[0].to_string(),
            roles: list(line, "role", cells[1], parse_role)?,
            damage_type: field(line, "damage type", cells[2], DamageType::parse)?,
            damage_profile: field(line, "damage profile", cells[3], DamageProfile::parse)?,
            range: field(line, "range", cells[4], RangeClass::parse)?,
            engage: field(line, "engage", cells[5], Engage::parse)?,
            disengage: field(line, "disengage", cells[6], Strength::parse)?,
            peel: field(line, "peel", cells[7], Strength::parse)?,
            frontline: field(line, "frontline", cells[8], Strength::parse)?,
            cc: field(line, "crowd control", cells[9], CrowdControl::parse)?,
            wave_clear: field(line, "wave clear", cells[10], WaveClear::parse)?,
            scaling: field(line, "scaling", cells[11], Scaling::parse)?,
            mobility: field(line, "mobility", cells[12], Mobility::parse)?,
            styles: list(line, "playstyle", cells[13], Playstyle::parse)?,
            confidence: field(line, "confidence", cells[14], Confidence::parse)?,
        };
        t.validate().map_err(|e| format!("line {line}: {e}"))?;
        rows.push(t);
    }
    if !header_seen {
        return Err("no header line".into());
    }
    let mut ids: Vec<&str> = rows.iter().map(|r| r.champion.as_str()).collect();
    ids.sort_unstable();
    if let Some(w) = ids.windows(2).find(|w| w[0] == w[1]) {
        return Err(format!("{} has two rows", w[0]));
    }
    Ok(rows)
}

/// The shipped defaults, parsed once. A broken file (which the unit tests
/// rule out) logs and yields no defaults rather than stopping the server.
pub fn defaults() -> &'static [ChampionTraits] {
    static DEFAULTS: OnceLock<Vec<ChampionTraits>> = OnceLock::new();
    DEFAULTS.get_or_init(|| match parse_defaults(DEFAULTS_CSV) {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("champion_traits.csv is invalid: {e}");
            Vec::new()
        }
    })
}

/// The shipped default row for a champion, if the file has one.
pub fn default_for(champion: &str) -> Option<&'static ChampionTraits> {
    defaults().iter().find(|t| t.champion == champion)
}

/// Lower-case, letters and digits only ("Kai'Sa" -> "kaisa").
pub fn fold(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The in-memory trait table with lookups and precomputed pair relations.
pub struct TraitTable {
    /// Sorted by champion id (deterministic order everywhere).
    pub rows: Vec<TraitRecord>,
    by_key: HashMap<String, usize>,
    pub relations: PairRelations,
}

impl TraitTable {
    pub fn new(mut rows: Vec<TraitRecord>) -> Self {
        rows.sort_by(|a, b| a.traits.champion.cmp(&b.traits.champion));
        let mut by_key = HashMap::with_capacity(rows.len() * 2);
        for (i, r) in rows.iter().enumerate() {
            by_key.insert(fold(&r.traits.champion), i);
            by_key.entry(fold(&r.name)).or_insert(i);
        }
        let traits: Vec<&ChampionTraits> = rows.iter().map(|r| &r.traits).collect();
        let relations = PairRelations::compute(&traits);
        Self {
            rows,
            by_key,
            relations,
        }
    }

    /// Row index for a champion id or display name ("Wukong", "MonkeyKing",
    /// "kai'sa").
    pub fn index(&self, id_or_name: &str) -> Option<usize> {
        self.by_key.get(&fold(id_or_name)).copied()
    }

    pub fn get(&self, id_or_name: &str) -> Option<&TraitRecord> {
        self.index(id_or_name).map(|i| &self.rows[i])
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

fn state() -> &'static RwLock<Option<Arc<TraitTable>>> {
    static STATE: OnceLock<RwLock<Option<Arc<TraitTable>>>> = OnceLock::new();
    STATE.get_or_init(|| RwLock::new(None))
}

/// The loaded table, if any.
pub fn table() -> Option<Arc<TraitTable>> {
    state().read().unwrap_or_else(|p| p.into_inner()).clone()
}

/// Replace the in-memory table.
pub fn install(rows: Vec<TraitRecord>) -> Arc<TraitTable> {
    let t = Arc::new(TraitTable::new(rows));
    *state().write().unwrap_or_else(|p| p.into_inner()) = Some(t.clone());
    t
}

/// Read the database copy into memory (display names from the Data Dragon
/// cache when it is loaded).
pub async fn reload(db: &Surreal<Db>) -> DbResult<Arc<TraitTable>> {
    let names: HashMap<String, String> = crate::server::data_dragon::snapshot()
        .map(|s| {
            s.champions
                .iter()
                .map(|c| (c.id.clone(), c.name.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut rows = db::list_champion_traits(db).await?;
    for r in &mut rows {
        if let Some(n) = names.get(&r.traits.champion) {
            r.name = n.clone();
        }
    }
    Ok(install(rows))
}

/// The table, loading it from the database when nothing is loaded yet.
pub async fn ensure_loaded(db: &Surreal<Db>) -> DbResult<Arc<TraitTable>> {
    match table() {
        Some(t) => Ok(t),
        None => reload(db).await,
    }
}

/// What [`sync`] did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// Rows written for champions that had none.
    pub created: usize,
    /// Default rows refreshed from a changed file.
    pub refreshed: usize,
    /// Champions Data Dragon knows that the file lacks (neutral rows).
    pub neutral: Vec<String>,
}

/// The rows the database should get: defaults for missing champions, file
/// values for rows still at `default` whose values differ, neutral rows for
/// champions only Data Dragon knows. `existing` maps champion id to
/// (source, traits). `known` is Data Dragon's list of (id, class tags).
pub fn plan_sync(
    defaults: &[ChampionTraits],
    existing: &HashMap<String, (TraitSource, ChampionTraits)>,
    known: &[(String, Vec<String>)],
) -> (Vec<ChampionTraits>, SyncReport) {
    let mut writes = Vec::new();
    let mut report = SyncReport::default();
    for d in defaults {
        match existing.get(&d.champion) {
            None => {
                writes.push(d.clone());
                report.created += 1;
            }
            Some((TraitSource::Default, cur)) if cur != d => {
                writes.push(d.clone());
                report.refreshed += 1;
            }
            _ => {}
        }
    }
    for (id, tags) in known {
        if defaults.iter().any(|d| &d.champion == id) || existing.contains_key(id) {
            continue;
        }
        if !id.chars().all(|c| c.is_ascii_alphanumeric()) || id.is_empty() {
            continue;
        }
        let role = Role::parse(guess_role_from_tags(tags)).unwrap_or(Role::Mid);
        writes.push(ChampionTraits::neutral(id.clone(), vec![role]));
        report.neutral.push(id.clone());
        report.created += 1;
    }
    (writes, report)
}

/// Bring the database copy up to date with the defaults file and Data
/// Dragon's champion list, then load the table into memory.
pub async fn sync(db: &Surreal<Db>) -> DbResult<SyncReport> {
    let existing: HashMap<String, (TraitSource, ChampionTraits)> = db::list_champion_traits(db)
        .await?
        .into_iter()
        .map(|r| (r.traits.champion.clone(), (r.source, r.traits)))
        .collect();
    let known: Vec<(String, Vec<String>)> = crate::server::data_dragon::snapshot()
        .map(|s| {
            s.champions
                .iter()
                .map(|c| (c.id.clone(), c.tags.clone()))
                .collect()
        })
        .unwrap_or_default();
    let (writes, report) = plan_sync(defaults(), &existing, &known);
    if !writes.is_empty() {
        db::write_default_traits(db, writes).await?;
    }
    if !report.neutral.is_empty() {
        tracing::warn!(
            champions = ?report.neutral,
            "champion traits: no default row in champion_traits.csv; wrote neutral rows (confidence low)"
        );
    }
    if report.created + report.refreshed > 0 {
        tracing::info!(
            created = report.created,
            refreshed = report.refreshed,
            "champion traits synced"
        );
    }
    reload(db).await?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_defaults_parse_and_validate() {
        let rows = parse_defaults(DEFAULTS_CSV).expect("champion_traits.csv must parse");
        assert!(rows.len() >= 170, "{} rows", rows.len());
        for r in &rows {
            r.validate().unwrap();
        }
        // Rows are kept in id order, so diffs stay readable.
        let ids: Vec<&str> = rows.iter().map(|r| r.champion.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "keep champion_traits.csv sorted by id");
    }

    /// Audit: every champion of the patch the defaults were written against
    /// (Data Dragon 16.20.1, `tests/fixtures/champion_ids.txt`) has a row.
    /// When a patch adds a champion, add its row to the CSV and its id to
    /// the fixture; until then the server writes a neutral row for it.
    #[test]
    fn every_known_champion_has_a_default_row() {
        let known = include_str!("../../../tests/fixtures/champion_ids.txt");
        let rows = parse_defaults(DEFAULTS_CSV).unwrap();
        let missing: Vec<&str> = known
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter(|id| !rows.iter().any(|r| r.champion == *id))
            .collect();
        assert!(missing.is_empty(), "no default row for {missing:?}");
        let extra: Vec<&str> = rows
            .iter()
            .map(|r| r.champion.as_str())
            .filter(|id| !known.lines().any(|l| l.trim() == *id))
            .collect();
        assert!(extra.is_empty(), "rows for unknown champions {extra:?}");
    }

    #[test]
    fn parser_rejects_values_outside_the_vocabulary() {
        let head = COLUMNS.join(",");
        let ok =
            "Ahri,mid,magic,burst,long,conditional,none,some,none,light,ok,mid,high,pick,normal";
        assert!(parse_defaults(&format!("{head}\n{ok}")).is_ok());
        for bad in [
            ok.replace(",magic,", ",arcane,"),
            ok.replace("Ahri,mid,", "Ahri,middle,"),
            ok.replace(",pick,", ",pick|flank,"),
            ok.replace(",normal", ",maybe"),
            ok.replace("Ahri,mid,", "Ahri,,"),
            format!("{ok},extra"),
        ] {
            assert!(parse_defaults(&format!("{head}\n{bad}")).is_err(), "{bad}");
        }
        assert!(
            parse_defaults(&format!("{head}\n{ok}\n{ok}")).is_err(),
            "duplicate"
        );
        assert!(parse_defaults(ok).is_err(), "no header");
    }

    #[test]
    fn sync_plan_never_touches_maintainer_rows() {
        let d = parse_defaults(DEFAULTS_CSV).unwrap();
        let ahri = d.iter().find(|r| r.champion == "Ahri").unwrap().clone();
        let mut edited = ahri.clone();
        edited.engage = Engage::Hard;
        let mut stale = d.iter().find(|r| r.champion == "Zed").unwrap().clone();
        stale.mobility = Mobility::Low;
        let mut existing = HashMap::new();
        existing.insert("Ahri".to_string(), (TraitSource::Maintainer, edited));
        existing.insert("Zed".to_string(), (TraitSource::Default, stale));
        let known = vec![
            ("Ahri".to_string(), vec!["Mage".to_string()]),
            ("Newcomer".to_string(), vec!["Marksman".to_string()]),
        ];
        let (writes, report) = plan_sync(&d, &existing, &known);
        assert!(!writes.iter().any(|w| w.champion == "Ahri"));
        assert_eq!(report.refreshed, 1, "Zed's stale default is refreshed");
        assert_eq!(report.neutral, vec!["Newcomer".to_string()]);
        let n = writes.iter().find(|w| w.champion == "Newcomer").unwrap();
        assert_eq!(n.confidence, Confidence::Low);
        assert_eq!(n.roles, vec![Role::Bot]);
        assert_eq!(report.created, d.len() - 2 + 1);
    }

    #[test]
    fn table_resolves_ids_and_display_names() {
        let rows: Vec<TraitRecord> = parse_defaults(DEFAULTS_CSV)
            .unwrap()
            .into_iter()
            .map(|t| TraitRecord {
                name: match t.champion.as_str() {
                    "MonkeyKing" => "Wukong".into(),
                    "Kaisa" => "Kai'Sa".into(),
                    other => other.to_string(),
                },
                traits: t,
                source: TraitSource::Default,
                edited_by: None,
                edited_at: None,
                reviewed_for_patch: None,
            })
            .collect();
        let t = TraitTable::new(rows);
        assert_eq!(t.get("Wukong").unwrap().traits.champion, "MonkeyKing");
        assert_eq!(t.get("monkeyking").unwrap().traits.champion, "MonkeyKing");
        assert_eq!(t.get("Kai'Sa").unwrap().traits.champion, "Kaisa");
        assert!(t.get("Nobody").is_none());
    }
}

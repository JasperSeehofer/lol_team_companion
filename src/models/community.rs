//! Shared types and pure helpers for the community loop (M7): surveys, the
//! what-next board, the transparency ledger, admin-approved site texts, the
//! changelog and the admin overview. Everything here compiles for both
//! targets; queries live in `server::community::store`.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Length limits (characters, after trimming)
// ---------------------------------------------------------------------------

pub const MAX_BUG_DESCRIPTION: usize = 4000;
pub const MAX_PUBLIC_TITLE: usize = 120;
pub const MAX_PUBLIC_NOTE: usize = 600;
pub const MAX_ADMIN_NOTE: usize = 2000;
pub const MAX_PROPOSAL_TITLE: usize = 100;
pub const MAX_PROPOSAL_SUMMARY: usize = 1000;
pub const MAX_STATUS_NOTE: usize = 600;
pub const MAX_SURVEY_QUESTION: usize = 200;
pub const MAX_SURVEY_CHOICE: usize = 60;
pub const MAX_SCALE_LABEL: usize = 30;
pub const MAX_SURVEY_TEXT_ANSWER: usize = 500;
pub const MAX_SURVEY_NOTE: usize = 500;
pub const MAX_LEDGER_NOTE: usize = 600;
pub const MAX_LEDGER_OTHER_LABEL: usize = 120;
pub const MAX_SITE_TEXT: usize = 3000;

/// Trim and drop control characters other than newlines (every stored text).
pub fn clean_text(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|c| *c == '\n' || !c.is_control())
        .collect()
}

/// `clean_text`, or an error message when it is longer than `max` characters.
pub fn clean_limited(value: &str, max: usize) -> Result<String, String> {
    let v = clean_text(value);
    if v.chars().count() > max {
        Err(format!(
            "That text is too long (up to {} characters).",
            group_thousands(max as i64)
        ))
    } else {
        Ok(v)
    }
}

fn group_thousands(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

/// "14 Oct 2026" from an RFC 3339 timestamp (or a `YYYY-MM-DD` date). The
/// date part is used as written (UTC); unparseable input is returned as is.
pub fn short_date(iso: &str) -> String {
    let date = iso.get(..10).unwrap_or(iso);
    let mut parts = date.split('-');
    let (Some(y), Some(m), Some(d)) = (parts.next(), parts.next(), parts.next()) else {
        return iso.to_string();
    };
    let Some(month) = m
        .parse::<usize>()
        .ok()
        .and_then(|m| MONTHS_SHORT.get(m.wrapping_sub(1)))
    else {
        return iso.to_string();
    };
    let day = d.trim_start_matches('0');
    format!("{day} {month} {y}")
}

const MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// "October 2026" from "2026-10"; the input as written when it is not a month.
pub fn month_label(month: &str) -> String {
    match parse_month(month) {
        Some((y, m)) => format!("{} {y}", MONTHS_LONG[(m - 1) as usize]),
        None => month.to_string(),
    }
}

/// `(year, month)` from "YYYY-MM".
pub fn parse_month(month: &str) -> Option<(i32, u32)> {
    let (y, m) = month.split_once('-')?;
    if y.len() != 4 || m.len() != 2 || !y.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let m: u32 = m.parse().ok()?;
    if !(1..=12).contains(&m) {
        return None;
    }
    Some((y.parse().ok()?, m))
}

/// "1 vote", "12 votes".
pub fn votes_label(n: u32) -> String {
    if n == 1 {
        "1 vote".to_string()
    } else {
        format!("{n} votes")
    }
}

// ---------------------------------------------------------------------------
// Surveys
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurveyKind {
    SingleChoice,
    Scale,
    ShortText,
}

impl SurveyKind {
    pub const ALL: [SurveyKind; 3] = [
        SurveyKind::SingleChoice,
        SurveyKind::Scale,
        SurveyKind::ShortText,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            SurveyKind::SingleChoice => "single_choice",
            SurveyKind::Scale => "scale",
            SurveyKind::ShortText => "short_text",
        }
    }
    pub fn parse(v: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == v)
    }
    pub fn label(self) -> &'static str {
        match self {
            SurveyKind::SingleChoice => "Single choice",
            SurveyKind::Scale => "Scale 1 to 5",
            SurveyKind::ShortText => "Short text",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurveyTrigger {
    DraftSaved3,
    ReviewSaved,
    PatchChanged,
    WhatNextVisit,
}

impl SurveyTrigger {
    pub const ALL: [SurveyTrigger; 4] = [
        SurveyTrigger::DraftSaved3,
        SurveyTrigger::ReviewSaved,
        SurveyTrigger::PatchChanged,
        SurveyTrigger::WhatNextVisit,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            SurveyTrigger::DraftSaved3 => "draft_saved_3",
            SurveyTrigger::ReviewSaved => "review_saved",
            SurveyTrigger::PatchChanged => "patch_changed",
            SurveyTrigger::WhatNextVisit => "what_next_visit",
        }
    }
    pub fn parse(v: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == v)
    }
    pub fn label(self) -> &'static str {
        match self {
            SurveyTrigger::DraftSaved3 => "After the 3rd draft",
            SurveyTrigger::ReviewSaved => "After a review",
            SurveyTrigger::PatchChanged => "After a patch",
            SurveyTrigger::WhatNextVisit => "On What next",
        }
    }
    pub fn explanation(self) -> &'static str {
        match self {
            SurveyTrigger::DraftSaved3 => "Asked on the next page after a user saves their third draft (or any later draft, if they already have three).",
            SurveyTrigger::ReviewSaved => "Asked on the next page after a user saves a new review.",
            SurveyTrigger::PatchChanged => "Asked once per patch, within 7 days after the app detects it. Use {patch} in the question for the version.",
            SurveyTrigger::WhatNextVisit => "Asked when a user opens the What next board.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurveyStatus {
    Draft,
    Approved,
    Live,
    Paused,
    Closed,
}

impl SurveyStatus {
    pub const ALL: [SurveyStatus; 5] = [
        SurveyStatus::Draft,
        SurveyStatus::Approved,
        SurveyStatus::Live,
        SurveyStatus::Paused,
        SurveyStatus::Closed,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            SurveyStatus::Draft => "draft",
            SurveyStatus::Approved => "approved",
            SurveyStatus::Live => "live",
            SurveyStatus::Paused => "paused",
            SurveyStatus::Closed => "closed",
        }
    }
    pub fn parse(v: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == v)
    }
    pub fn label(self) -> &'static str {
        match self {
            SurveyStatus::Draft => "Draft",
            SurveyStatus::Approved => "Approved",
            SurveyStatus::Live => "Live",
            SurveyStatus::Paused => "Paused",
            SurveyStatus::Closed => "Closed",
        }
    }
    /// Allowed admin status changes (approval itself is a separate action).
    pub fn can_move_to(self, to: SurveyStatus) -> bool {
        use SurveyStatus::*;
        matches!(
            (self, to),
            (Approved, Live) | (Live, Paused) | (Paused, Live) | (_, Closed)
        ) && self != to
    }
}

/// The parts of a survey a user reads. Its canonical JSON is what the
/// wording hash covers: an approval holds for exactly this wording.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SurveyWording {
    pub question: String,
    pub kind: String,
    pub choices: Vec<String>,
    pub scale_low_label: Option<String>,
    pub scale_high_label: Option<String>,
}

impl SurveyWording {
    /// Stable serialisation (fixed field order, no whitespace) for hashing.
    pub fn canonical_json(&self) -> String {
        serde_json::json!([
            self.question,
            self.kind,
            self.choices,
            self.scale_low_label,
            self.scale_high_label
        ])
        .to_string()
    }
}

/// An admin's survey form, sent as JSON.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SurveyInput {
    pub id: Option<String>,
    pub question: String,
    pub kind: String,
    /// One option per line (single choice).
    pub choices_text: String,
    pub scale_low_label: String,
    pub scale_high_label: String,
    pub trigger: String,
    pub note: String,
}

/// Check a survey form; returns the cleaned wording, trigger and note.
pub fn validate_survey(
    input: &SurveyInput,
) -> Result<(SurveyWording, SurveyTrigger, Option<String>), String> {
    let question = clean_text(&input.question);
    if question.is_empty() {
        return Err("Write a question.".into());
    }
    if question.chars().count() > MAX_SURVEY_QUESTION {
        return Err("The question can have up to 200 characters.".into());
    }
    let kind = SurveyKind::parse(&input.kind).ok_or("Choose a type.")?;
    let trigger = SurveyTrigger::parse(&input.trigger).ok_or("Choose when to ask.")?;
    if question.contains("{patch}") && trigger != SurveyTrigger::PatchChanged {
        return Err("{patch} only works for questions asked after a patch.".into());
    }
    let mut choices = Vec::new();
    let (mut low, mut high) = (None, None);
    match kind {
        SurveyKind::SingleChoice => {
            choices = input
                .choices_text
                .lines()
                .map(clean_text)
                .filter(|c| !c.is_empty())
                .collect();
            if !(2..=5).contains(&choices.len()) {
                return Err("A single-choice question needs two to five options.".into());
            }
            if choices
                .iter()
                .any(|c| c.chars().count() > MAX_SURVEY_CHOICE)
            {
                return Err("Each option can have up to 60 characters.".into());
            }
        }
        SurveyKind::Scale => {
            let l = clean_text(&input.scale_low_label);
            let h = clean_text(&input.scale_high_label);
            if l.is_empty() || h.is_empty() {
                return Err("A scale needs a label for 1 and for 5.".into());
            }
            if l.chars().count() > MAX_SCALE_LABEL || h.chars().count() > MAX_SCALE_LABEL {
                return Err("Scale labels can have up to 30 characters.".into());
            }
            low = Some(l);
            high = Some(h);
        }
        SurveyKind::ShortText => {}
    }
    let note = clean_limited(&input.note, MAX_SURVEY_NOTE)?;
    Ok((
        SurveyWording {
            question,
            kind: kind.as_str().to_string(),
            choices,
            scale_low_label: low,
            scale_high_label: high,
        },
        trigger,
        Some(note).filter(|n| !n.is_empty()),
    ))
}

/// How many prompts of a survey are in each state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SurveyCounts {
    pub pending: u32,
    pub shown: u32,
    pub answered: u32,
    pub dismissed: u32,
    pub expired: u32,
}

impl SurveyCounts {
    /// Shown at least once (answered and dismissed prompts were shown).
    pub fn delivered(&self) -> u32 {
        self.shown + self.answered + self.dismissed
    }
}

/// A survey as admins see it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Survey {
    pub id: String,
    pub seed_key: Option<String>,
    pub wording: SurveyWording,
    pub trigger: String,
    pub status: String,
    pub wording_hash: String,
    pub approved_hash: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub note: Option<String>,
    pub updated_at: Option<String>,
    pub counts: SurveyCounts,
}

impl Survey {
    /// The approval covers the current wording.
    pub fn approval_valid(&self) -> bool {
        self.approved_hash.as_deref() == Some(self.wording_hash.as_str())
    }
    /// Approved once, but the wording changed since.
    pub fn approval_withdrawn(&self) -> bool {
        self.approved_hash.is_some() && !self.approval_valid()
    }
}

/// What the survey band shows a user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurveyPromptView {
    pub prompt_id: String,
    pub question: String,
    pub kind: String,
    pub choices: Vec<String>,
    pub scale_low_label: Option<String>,
    pub scale_high_label: Option<String>,
}

/// A user's answer, sent as JSON.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SurveyAnswer {
    Choice(u32),
    Scale(u32),
    Text(String),
}

impl SurveyAnswer {
    /// Check the answer against the question it answers.
    pub fn validate(&self, kind: SurveyKind, choice_count: usize) -> Result<SurveyAnswer, String> {
        const MISMATCH: &str = "That answer does not fit this question.";
        match (self, kind) {
            (SurveyAnswer::Choice(i), SurveyKind::SingleChoice) if (*i as usize) < choice_count => {
                Ok(self.clone())
            }
            (SurveyAnswer::Scale(v), SurveyKind::Scale) if (1..=5).contains(v) => Ok(self.clone()),
            (SurveyAnswer::Text(t), SurveyKind::ShortText) => {
                let t = clean_text(t);
                if t.is_empty() {
                    Err(MISMATCH.into())
                } else if t.chars().count() > MAX_SURVEY_TEXT_ANSWER {
                    Err("Please keep it under 500 characters.".into())
                } else {
                    Ok(SurveyAnswer::Text(t))
                }
            }
            _ => Err(MISMATCH.into()),
        }
    }
}

/// Results for one wording version of a survey.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SurveyVersionResult {
    pub hash: String,
    /// The wording as it was approved (the current one when it matches).
    pub question: String,
    pub answers: u32,
    /// (option, count) for single choice.
    pub choice_counts: Vec<(String, u32)>,
    /// Counts for scores 1 to 5.
    pub scale_counts: Vec<u32>,
    /// Text answers, newest first: (date, text). Never a name.
    pub texts: Vec<(String, String)>,
}

impl SurveyVersionResult {
    /// The average score, shown only from 5 answers.
    pub fn mean(&self) -> Option<f64> {
        let n: u32 = self.scale_counts.iter().sum();
        if n < 5 {
            return None;
        }
        let total: u32 = self
            .scale_counts
            .iter()
            .enumerate()
            .map(|(i, c)| (i as u32 + 1) * c)
            .sum();
        Some(total as f64 / n as f64)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SurveyResults {
    pub kind: String,
    pub counts: SurveyCounts,
    pub versions: Vec<SurveyVersionResult>,
}

/// Share of `part` in `whole` as a whole percent ("33%").
pub fn percent(part: u32, whole: u32) -> String {
    if whole == 0 {
        "0%".to_string()
    } else {
        format!("{}%", ((part as f64 / whole as f64) * 100.0).round() as u32)
    }
}

// ---------------------------------------------------------------------------
// What-next board
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    Proposed,
    Planned,
    Building,
    Shipped,
    Declined,
}

impl ProposalStatus {
    pub const ALL: [ProposalStatus; 6] = [
        ProposalStatus::Draft,
        ProposalStatus::Proposed,
        ProposalStatus::Planned,
        ProposalStatus::Building,
        ProposalStatus::Shipped,
        ProposalStatus::Declined,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            ProposalStatus::Draft => "draft",
            ProposalStatus::Proposed => "proposed",
            ProposalStatus::Planned => "planned",
            ProposalStatus::Building => "building",
            ProposalStatus::Shipped => "shipped",
            ProposalStatus::Declined => "declined",
        }
    }
    pub fn parse(v: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == v)
    }
    pub fn label(self) -> &'static str {
        match self {
            ProposalStatus::Draft => "Draft (admins only)",
            ProposalStatus::Proposed => "Proposed",
            ProposalStatus::Planned => "Planned",
            ProposalStatus::Building => "Building",
            ProposalStatus::Shipped => "Shipped",
            ProposalStatus::Declined => "Declined",
        }
    }
    /// Voting is open.
    pub fn is_open(self) -> bool {
        matches!(
            self,
            ProposalStatus::Proposed | ProposalStatus::Planned | ProposalStatus::Building
        )
    }
}

pub fn proposal_source_label(source: &str) -> &'static str {
    match source {
        "mission" => "MISSION",
        "backlog" => "Backlog",
        "wishlist" => "Wishlist report",
        _ => "Admin",
    }
}

/// A proposal with its vote count.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProposalView {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub status_note: Option<String>,
    pub shipped_in: Option<String>,
    pub shipped_in_title: Option<String>,
    pub votes: u32,
    pub voted_by_me: bool,
    pub created_at: String,
    pub status_changed_at: Option<String>,
    /// Admin fields (empty or false for users).
    pub published: bool,
    pub source: String,
    pub wishlist_report: Option<String>,
}

/// An admin's proposal form, sent as JSON.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProposalInput {
    pub id: Option<String>,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub status_note: String,
    pub shipped_in: String,
    pub published: bool,
}

/// Board order: open ideas by votes (most first), ties by age (older
/// first), then by id; shipped and declined ideas by the date their status
/// changed (newest first). Status never reorders open ideas.
pub fn sort_proposals(list: &mut [ProposalView]) {
    fn group(p: &ProposalView) -> u8 {
        match ProposalStatus::parse(&p.status) {
            Some(s) if s.is_open() => 0,
            Some(ProposalStatus::Shipped) => 1,
            Some(ProposalStatus::Declined) => 2,
            _ => 3,
        }
    }
    list.sort_by(|a, b| {
        group(a).cmp(&group(b)).then_with(|| {
            if group(a) == 0 {
                b.votes
                    .cmp(&a.votes)
                    .then_with(|| a.created_at.cmp(&b.created_at))
                    .then_with(|| a.id.cmp(&b.id))
            } else {
                b.status_changed_at
                    .cmp(&a.status_changed_at)
                    .then_with(|| a.id.cmp(&b.id))
            }
        })
    });
}

// ---------------------------------------------------------------------------
// Transparency ledger
// ---------------------------------------------------------------------------

/// Largest amount accepted in one field: one million euros.
pub const MAX_CENTS: i64 = 100_000_000;

/// One month of the ledger. Amounts in euro cents.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LedgerMonth {
    pub month: String,
    pub support_cents: i64,
    pub hosting_cents: i64,
    pub domain_cents: i64,
    pub ai_tokens_cents: i64,
    pub other_cents: i64,
    pub other_label: Option<String>,
    pub note: Option<String>,
    pub published: bool,
    pub published_at: Option<String>,
    pub revised_at: Option<String>,
    pub updated_at: Option<String>,
    pub updated_by: Option<String>,
}

impl LedgerMonth {
    pub fn costs_cents(&self) -> i64 {
        self.hosting_cents + self.domain_cents + self.ai_tokens_cents + self.other_cents
    }
    pub fn to_maintainer_cents(&self) -> i64 {
        to_maintainer_cents(self.support_cents, self.costs_cents())
    }
    pub fn not_covered_cents(&self) -> i64 {
        shortfall_cents(self.support_cents, self.costs_cents())
    }
}

/// Costs first, then the remainder to the maintainer.
pub fn to_maintainer_cents(support: i64, costs: i64) -> i64 {
    (support - costs).max(0)
}

/// Costs that support did not cover.
pub fn shortfall_cents(support: i64, costs: i64) -> i64 {
    (costs - support).max(0)
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LedgerTotals {
    pub months: u32,
    pub support_cents: i64,
    pub costs_cents: i64,
    pub to_maintainer_cents: i64,
    pub not_covered_cents: i64,
}

/// Totals over the given months (each month's split computed on its own, so
/// a surplus in one month never hides a shortfall in another).
pub fn ledger_totals(months: &[LedgerMonth]) -> LedgerTotals {
    let mut t = LedgerTotals::default();
    for m in months {
        t.months += 1;
        t.support_cents += m.support_cents;
        t.costs_cents += m.costs_cents();
        t.to_maintainer_cents += m.to_maintainer_cents();
        t.not_covered_cents += m.not_covered_cents();
    }
    t
}

/// "€1,234.50".
pub fn format_euros(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let c = cents.abs();
    format!("{sign}€{}.{:02}", group_thousands(c / 100), c % 100)
}

/// Euro cents from "12", "12.5" or "12,50": one decimal separator (comma
/// or dot), no thousands separators.
/// Rejects negatives, more than two decimals and more than €1,000,000.
pub fn parse_euros(value: &str) -> Result<i64, String> {
    const BAD: &str = "Amounts must be zero or more, in euros with up to two decimals.";
    let v = value.trim().trim_start_matches('€').trim();
    if v.is_empty() {
        return Ok(0);
    }
    let v = v.replace(',', ".");
    let (whole, frac) = match v.split_once('.') {
        Some((w, f)) => (w.to_string(), f.to_string()),
        None => (v.clone(), String::new()),
    };
    if whole.is_empty()
        || !whole.chars().all(|c| c.is_ascii_digit())
        || !frac.chars().all(|c| c.is_ascii_digit())
        || frac.len() > 2
        || whole.len() > 9
    {
        return Err(BAD.into());
    }
    let whole: i64 = whole.parse().map_err(|_| BAD.to_string())?;
    let frac: i64 = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().map_err(|_| BAD.to_string())? * 10,
        _ => frac.parse().map_err(|_| BAD.to_string())?,
    };
    let cents = whole * 100 + frac;
    if cents > MAX_CENTS {
        return Err("That amount looks too large. Please check it.".into());
    }
    Ok(cents)
}

/// Cents as an editable amount ("12.50", "" for zero).
pub fn cents_to_input(cents: i64) -> String {
    if cents == 0 {
        String::new()
    } else {
        format!("{}.{:02}", cents / 100, cents % 100)
    }
}

/// The admin's month form (amounts as typed), sent as JSON.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LedgerInput {
    /// Sent by "Add month": refuse a month that already has a row instead
    /// of overwriting it (edits go through the row's Edit button).
    #[serde(default)]
    pub is_new: bool,
    pub month: String,
    pub support: String,
    pub hosting: String,
    pub domain: String,
    pub ai_tokens: String,
    pub other: String,
    pub other_label: String,
    pub note: String,
}

/// Validate a month form against the current month ("YYYY-MM", server clock).
pub fn validate_ledger(input: &LedgerInput, current_month: &str) -> Result<LedgerMonth, String> {
    let month = input.month.trim().to_string();
    let Some(ym) = parse_month(&month) else {
        return Err("Enter the month as YYYY-MM.".into());
    };
    if let Some(now) = parse_month(current_month) {
        if ym > now {
            return Err("That month has not started yet.".into());
        }
    }
    let other_cents = parse_euros(&input.other)?;
    let other_label = clean_limited(&input.other_label, MAX_LEDGER_OTHER_LABEL)?;
    if other_cents > 0 && other_label.is_empty() {
        return Err("Say what the other costs were.".into());
    }
    let note = clean_limited(&input.note, MAX_LEDGER_NOTE)?;
    Ok(LedgerMonth {
        month,
        support_cents: parse_euros(&input.support)?,
        hosting_cents: parse_euros(&input.hosting)?,
        domain_cents: parse_euros(&input.domain)?,
        ai_tokens_cents: parse_euros(&input.ai_tokens)?,
        other_cents,
        other_label: Some(other_label).filter(|s| !s.is_empty()),
        note: Some(note).filter(|s| !s.is_empty()),
        ..Default::default()
    })
}

// ---------------------------------------------------------------------------
// Site texts (owner-approved public wording)
// ---------------------------------------------------------------------------

/// The money policy shown under "How support is used" (paragraphs separated
/// by a blank line).
pub const TEXT_MONEY_POLICY: &str = "money_policy";
/// The public ledger labels, one per line, in [`LEDGER_LABEL_ORDER`].
pub const TEXT_LEDGER_LABELS: &str = "ledger_labels";
/// Caption above the ledger table explaining the split bar.
pub const TEXT_SPLIT_LEGEND: &str = "ledger_split_legend";
/// Caption under a month whose costs were above support.
pub const TEXT_NOT_COVERED: &str = "ledger_not_covered_note";
/// The sentence directly above the support button.
pub const TEXT_SUPPORT_SENTENCE: &str = "support_sentence";

/// Every site text, in admin order: (slug, what it is for).
pub const SITE_TEXTS: &[(&str, &str)] = &[
    (
        TEXT_MONEY_POLICY,
        "Policy under \"How support is used\" on the transparency page. Separate paragraphs with a blank line.",
    ),
    (
        TEXT_LEDGER_LABELS,
        "Labels of the public readouts and table, one per line in this order: support received, hosting, domain, AI tokens, other, running costs, to the maintainer, not covered by support.",
    ),
    (
        TEXT_SPLIT_LEGEND,
        "Caption above the table that explains the split bar.",
    ),
    (
        TEXT_NOT_COVERED,
        "Caption under a month whose costs were above support.",
    ),
    (
        TEXT_SUPPORT_SENTENCE,
        "Sentence above the support button (the button shows only when SUPPORT_URL is set).",
    ),
];

/// A site text as admins see it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteText {
    pub slug: String,
    pub purpose: String,
    pub body: String,
    pub approved: bool,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub updated_by: Option<String>,
    pub updated_at: Option<String>,
}

/// The ledger labels in the order the `ledger_labels` text lists them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedgerLabels {
    pub support: String,
    pub hosting: String,
    pub domain: String,
    pub ai_tokens: String,
    pub other: String,
    pub costs: String,
    pub to_maintainer: String,
    pub not_covered: String,
}

pub const LEDGER_LABEL_COUNT: usize = 8;

impl LedgerLabels {
    /// Exactly eight non-empty lines.
    pub fn parse(body: &str) -> Option<LedgerLabels> {
        let lines: Vec<String> = body
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        if lines.len() != LEDGER_LABEL_COUNT || lines.iter().any(|l| l.chars().count() > 40) {
            return None;
        }
        let mut it = lines.into_iter();
        let mut next = || it.next().unwrap_or_default();
        Some(LedgerLabels {
            support: next(),
            hosting: next(),
            domain: next(),
            ai_tokens: next(),
            other: next(),
            costs: next(),
            to_maintainer: next(),
            not_covered: next(),
        })
    }
}

/// Validate a site text body for its slug.
pub fn validate_site_text(slug: &str, body: &str) -> Result<String, String> {
    if !SITE_TEXTS.iter().any(|(s, _)| *s == slug) {
        return Err("Unknown text.".into());
    }
    let body: String = body
        .replace("\r\n", "\n")
        .chars()
        .filter(|c| *c == '\n' || !c.is_control())
        .collect::<String>()
        .trim()
        .to_string();
    if body.is_empty() {
        return Err("Write the text.".into());
    }
    if body.chars().count() > MAX_SITE_TEXT {
        return Err("That text is too long (up to 3,000 characters).".into());
    }
    if slug == TEXT_LEDGER_LABELS && LedgerLabels::parse(&body).is_none() {
        return Err("Write exactly eight labels, one per line, each up to 40 characters.".into());
    }
    let hits = forbidden_money_words(&body);
    if !hits.is_empty() {
        return Err(format!(
            "Please avoid these words: {}. The word is \"support\".",
            hits.join(", ")
        ));
    }
    Ok(body)
}

/// Paragraphs of a text (split on blank lines).
pub fn paragraphs(body: &str) -> Vec<String> {
    body.split("\n\n")
        .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|p| !p.is_empty())
        .collect()
}

/// Words never used for the act of supporting Companion (MISSION section 4).
/// "donation" is allowed only inside "no donation receipt is issued".
pub const FORBIDDEN_MONEY_WORDS: &[&str] = &[
    "donation",
    "donations",
    "donate",
    "donating",
    "spende",
    "spenden",
    "tip",
    "tips",
    "sponsor",
    "sponsoring",
    "perk",
    "perks",
    "reward",
    "rewards",
    "thank-you gift",
    "buy us a coffee",
    "supporter badge",
];

/// The forbidden words found in `text` (whole words, any case), ignoring
/// the one allowed sentence about donation receipts.
pub fn forbidden_money_words(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    let lower = lower
        .replace("no donation receipt is issued", "")
        .replace("no donation receipt", "");
    let is_word = |c: char| c.is_alphanumeric() || c == '-';
    let mut hits = Vec::new();
    for w in FORBIDDEN_MONEY_WORDS {
        let mut start = 0;
        while let Some(pos) = lower[start..].find(w) {
            let at = start + pos;
            let before = lower[..at].chars().next_back();
            let after = lower[at + w.len()..].chars().next();
            if !before.is_some_and(is_word) && !after.is_some_and(is_word) {
                hits.push(*w);
                break;
            }
            start = at + w.len();
        }
    }
    hits
}

/// The support link section (only with `SUPPORT_URL` and an approved sentence).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupportLink {
    pub url: String,
    pub sentence: String,
}

/// The public transparency page. Every money text is present only once an
/// admin approved it; otherwise the page shows its neutral empty state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LedgerView {
    /// Approved policy paragraphs (`None`: not approved, show nothing about money).
    pub policy: Option<Vec<String>>,
    /// Approved labels; months are listed only with them.
    pub labels: Option<LedgerLabels>,
    pub split_legend: Option<String>,
    pub not_covered_note: Option<String>,
    /// Published months, newest first (empty without approved labels).
    pub months: Vec<LedgerMonth>,
    /// Over the last 12 published months.
    pub totals: LedgerTotals,
    pub support: Option<SupportLink>,
}

// ---------------------------------------------------------------------------
// Changelog
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanKind {
    Text,
    Code,
    Strong,
}

/// A run of text inside a changelog item.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub kind: SpanKind,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangelogSection {
    pub heading: String,
    pub items: Vec<Vec<Span>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub slug: String,
    pub title: String,
    /// `YYYY-MM-DD` when the file records it.
    pub date: Option<String>,
    pub sections: Vec<ChangelogSection>,
}

/// A detected game patch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatchNote {
    pub version: String,
    pub previous: Option<String>,
    pub detected_at: String,
}

/// Signed-in extras under one changelog entry: (id, title) pairs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EntryLinks {
    pub slug: String,
    pub fixed_reports: Vec<(String, String)>,
    pub shipped_proposals: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChangelogView {
    pub entries: Vec<ChangelogEntry>,
    pub patches: Vec<PatchNote>,
    pub links: Vec<EntryLinks>,
}

/// (slug, title) of every changelog entry of the running build, for the
/// admin "Fixed in" / "Shipped in" selects.
pub type ChangelogChoice = (String, String);

// ---------------------------------------------------------------------------
// Admin hub
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdminOverview {
    pub bugs_new: u32,
    pub bugs_triaged: u32,
    pub surveys_waiting: u32,
    pub surveys_live: u32,
    pub proposals_open: u32,
    pub proposals_unpublished: u32,
    pub votes: u32,
    pub ledger_last_published: Option<String>,
    pub ledger_drafts: u32,
    pub invites_open: u32,
    pub texts_waiting: u32,
    pub texts_total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: &str, status: &str, votes: u32, created: &str, changed: Option<&str>) -> ProposalView {
        ProposalView {
            id: id.into(),
            title: id.into(),
            summary: String::new(),
            status: status.into(),
            status_note: None,
            shipped_in: None,
            shipped_in_title: None,
            votes,
            voted_by_me: false,
            created_at: created.into(),
            status_changed_at: changed.map(String::from),
            published: true,
            source: "admin".into(),
            wishlist_report: None,
        }
    }

    #[test]
    fn proposals_sort_by_votes_then_age_then_id() {
        let mut list = vec![
            p("d", "shipped", 50, "2026-01-01", Some("2026-03-01")),
            p("a", "proposed", 2, "2026-02-01", None),
            p("b", "planned", 5, "2026-03-01", None),
            p("c", "building", 2, "2026-01-01", None),
            p("e", "shipped", 1, "2026-01-01", Some("2026-04-01")),
            p("f", "declined", 9, "2026-01-01", Some("2026-02-01")),
            p("g", "proposed", 2, "2026-01-01", None),
        ];
        sort_proposals(&mut list);
        let ids: Vec<&str> = list.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["b", "c", "g", "a", "e", "d", "f"]);
    }

    #[test]
    fn ledger_maths() {
        assert_eq!(to_maintainer_cents(1000, 400), 600);
        assert_eq!(to_maintainer_cents(100, 400), 0);
        assert_eq!(shortfall_cents(100, 400), 300);
        assert_eq!(shortfall_cents(500, 400), 0);
        let a = LedgerMonth {
            month: "2026-09".into(),
            support_cents: 5000,
            hosting_cents: 1000,
            domain_cents: 100,
            ai_tokens_cents: 900,
            ..Default::default()
        };
        let b = LedgerMonth {
            month: "2026-10".into(),
            support_cents: 500,
            hosting_cents: 1000,
            ..Default::default()
        };
        assert_eq!(a.costs_cents(), 2000);
        assert_eq!(a.to_maintainer_cents(), 3000);
        let t = ledger_totals(&[a, b]);
        assert_eq!(t.months, 2);
        assert_eq!(t.support_cents, 5500);
        assert_eq!(t.costs_cents, 3000);
        assert_eq!(t.to_maintainer_cents, 3000);
        assert_eq!(t.not_covered_cents, 500);
    }

    #[test]
    fn euros_format_and_parse() {
        assert_eq!(format_euros(0), "€0.00");
        assert_eq!(format_euros(123450), "€1,234.50");
        assert_eq!(format_euros(5), "€0.05");
        assert_eq!(format_euros(100_000_000), "€1,000,000.00");
        assert_eq!(parse_euros("12"), Ok(1200));
        assert_eq!(parse_euros("12,5"), Ok(1250));
        assert_eq!(parse_euros("12.50"), Ok(1250));
        assert_eq!(parse_euros(" €3.07 "), Ok(307));
        assert_eq!(parse_euros(""), Ok(0));
        assert!(parse_euros("-1").is_err());
        assert!(parse_euros("1.234").is_err());
        assert!(parse_euros("1,234.50").is_err());
        assert!(parse_euros("abc").is_err());
        assert!(parse_euros("1000000.01").is_err());
        assert_eq!(parse_euros("1000000"), Ok(MAX_CENTS));
        assert_eq!(cents_to_input(1250), "12.50");
        assert_eq!(cents_to_input(0), "");
    }

    #[test]
    fn months_and_dates_read_naturally() {
        assert_eq!(month_label("2026-10"), "October 2026");
        assert_eq!(month_label("2026-13"), "2026-13");
        assert_eq!(parse_month("2026-1"), None);
        assert_eq!(short_date("2026-10-04T10:00:00Z"), "4 Oct 2026");
        assert_eq!(short_date("2026-10-14"), "14 Oct 2026");
        assert_eq!(short_date("junk"), "junk");
        assert_eq!(votes_label(1), "1 vote");
        assert_eq!(votes_label(0), "0 votes");
        assert_eq!(percent(1, 3), "33%");
        assert_eq!(percent(0, 0), "0%");
    }

    #[test]
    fn ledger_form_validation() {
        let mut input = LedgerInput {
            month: "2026-10".into(),
            support: "20".into(),
            hosting: "5.5".into(),
            ..Default::default()
        };
        let m = validate_ledger(&input, "2026-10").unwrap();
        assert_eq!(m.support_cents, 2000);
        assert_eq!(m.hosting_cents, 550);
        assert!(validate_ledger(&input, "2026-09").is_err(), "future month");
        input.month = "10/2026".into();
        assert!(validate_ledger(&input, "2026-10").is_err());
        input.month = "2026-10".into();
        input.other = "3".into();
        assert_eq!(
            validate_ledger(&input, "2026-10").unwrap_err(),
            "Say what the other costs were."
        );
        input.other_label = "Payment fees".into();
        assert_eq!(validate_ledger(&input, "2026-10").unwrap().other_cents, 300);
    }

    #[test]
    fn wording_hash_input_covers_wording_only() {
        let w = SurveyWording {
            question: "Q?".into(),
            kind: "scale".into(),
            choices: vec![],
            scale_low_label: Some("Hard".into()),
            scale_high_label: Some("Easy".into()),
        };
        let base = w.canonical_json();
        assert_eq!(base, w.clone().canonical_json(), "stable");
        let mut changed = w.clone();
        changed.question = "Q2?".into();
        assert_ne!(base, changed.canonical_json());
        let mut changed = w.clone();
        changed.scale_high_label = Some("Very easy".into());
        assert_ne!(base, changed.canonical_json());
        let mut changed = w.clone();
        changed.kind = "short_text".into();
        assert_ne!(base, changed.canonical_json());
        let mut changed = w;
        changed.choices = vec!["a".into()];
        assert_ne!(base, changed.canonical_json());
    }

    #[test]
    fn survey_validation_per_kind() {
        let mut input = SurveyInput {
            question: "Did it help?".into(),
            kind: "single_choice".into(),
            choices_text: "Yes\n\nNo\n".into(),
            trigger: "review_saved".into(),
            ..Default::default()
        };
        let (w, t, note) = validate_survey(&input).unwrap();
        assert_eq!(w.choices, vec!["Yes", "No"]);
        assert_eq!(t, SurveyTrigger::ReviewSaved);
        assert_eq!(note, None);
        input.choices_text = "Only one".into();
        assert!(validate_survey(&input).is_err());
        input.kind = "scale".into();
        assert!(validate_survey(&input).is_err(), "scale needs labels");
        input.scale_low_label = "Hard".into();
        input.scale_high_label = "Easy".into();
        let (w, _, _) = validate_survey(&input).unwrap();
        assert!(w.choices.is_empty());
        input.question = "Patch {patch}?".into();
        assert!(
            validate_survey(&input).is_err(),
            "{{patch}} needs the patch trigger"
        );
        input.trigger = "patch_changed".into();
        assert!(validate_survey(&input).is_ok());
        input.question = "   ".into();
        assert!(validate_survey(&input).is_err());
    }

    #[test]
    fn survey_status_moves() {
        use SurveyStatus::*;
        assert!(Approved.can_move_to(Live));
        assert!(Live.can_move_to(Paused));
        assert!(Paused.can_move_to(Live));
        assert!(Draft.can_move_to(Closed));
        assert!(!Draft.can_move_to(Live));
        assert!(!Closed.can_move_to(Live));
        assert!(!Closed.can_move_to(Closed));
    }

    #[test]
    fn answers_must_fit_the_question() {
        assert!(SurveyAnswer::Choice(1)
            .validate(SurveyKind::SingleChoice, 2)
            .is_ok());
        assert!(SurveyAnswer::Choice(2)
            .validate(SurveyKind::SingleChoice, 2)
            .is_err());
        assert!(SurveyAnswer::Scale(5)
            .validate(SurveyKind::Scale, 0)
            .is_ok());
        assert!(SurveyAnswer::Scale(0)
            .validate(SurveyKind::Scale, 0)
            .is_err());
        assert!(SurveyAnswer::Scale(3)
            .validate(SurveyKind::ShortText, 0)
            .is_err());
        assert_eq!(
            SurveyAnswer::Text("  hi  ".into()).validate(SurveyKind::ShortText, 0),
            Ok(SurveyAnswer::Text("hi".into()))
        );
        assert!(SurveyAnswer::Text("x".repeat(501))
            .validate(SurveyKind::ShortText, 0)
            .is_err());
        assert!(SurveyAnswer::Text(" ".into())
            .validate(SurveyKind::ShortText, 0)
            .is_err());
    }

    #[test]
    fn mean_needs_five_answers() {
        let mut r = SurveyVersionResult {
            scale_counts: vec![1, 0, 0, 0, 3],
            ..Default::default()
        };
        assert_eq!(r.mean(), None);
        r.scale_counts = vec![1, 0, 0, 0, 4];
        assert_eq!(r.mean(), Some(21.0 / 5.0));
    }

    #[test]
    fn forbidden_words_are_found_as_whole_words() {
        assert!(forbidden_money_words("No donation receipt is issued.").is_empty());
        assert_eq!(forbidden_money_words("Make a donation"), vec!["donation"]);
        assert_eq!(forbidden_money_words("Donate now"), vec!["donate"]);
        assert_eq!(forbidden_money_words("Eine Spende"), vec!["spende"]);
        assert_eq!(forbidden_money_words("supporter perks"), vec!["perks"]);
        assert!(forbidden_money_words("multiple tooltips; tipping point").is_empty());
        assert!(forbidden_money_words("Support Companion").is_empty());
    }

    #[test]
    fn ledger_labels_need_eight_lines() {
        let body = "Support received\nHosting\nDomain\nAI tokens\nOther\nRunning costs\nTo the maintainer\nNot covered by support";
        let l = LedgerLabels::parse(body).unwrap();
        assert_eq!(l.ai_tokens, "AI tokens");
        assert_eq!(l.not_covered, "Not covered by support");
        assert!(LedgerLabels::parse("a\nb").is_none());
        assert!(validate_site_text(TEXT_LEDGER_LABELS, "a\nb").is_err());
        assert!(validate_site_text("nope", "x").is_err());
        assert!(validate_site_text(TEXT_SUPPORT_SENTENCE, "Please donate").is_err());
        assert_eq!(
            paragraphs("One\nline.\n\n\nTwo."),
            vec!["One line.".to_string(), "Two.".to_string()]
        );
    }

    #[test]
    fn text_cleaning() {
        assert_eq!(clean_text("  a\u{7}b\nc  "), "ab\nc");
        assert!(clean_limited(&"x".repeat(11), 10).is_err());
        assert_eq!(group_thousands(4000), "4,000");
    }
}

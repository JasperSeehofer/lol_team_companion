//! Tournament draft: the fixed 20-step order and the compact two-column
//! board that the draft page, the tree drafter and the gallery all use.
//!
//! Order (blue starts, red opens ban phase 2):
//! - Ban phase 1: B R B R B R (three bans each)
//! - Pick phase 1: B1, R1, R2, B2, B3, R3
//! - Ban phase 2: R B R B (two bans each)
//! - Pick phase 2: R4, B4, B5, R5
//!
//! The board is two columns, blue left and red right. Each column reads
//! top to bottom and, once the draft is complete, is that team's final
//! draft: five bans at the top (grouped 3 + 2 by ban rotation), then five
//! picks in pick order, bracketed by pick rotation (B1 · R1 R2 · B2 B3 · R3
//! · R4 · B4 B5 · R5). The step being decided is the one lit thing.

use leptos::prelude::*;

use super::a11y::focus_id;
use super::avatar::{AvatarSize, AvatarState, ChampionAvatar};
use super::badge::Badge;
use super::button::Keycap;
use super::markers::{Role, RoleLabel, SideMarker, SideText};
use super::text::join;

/// A side of the draft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftSide {
    Blue,
    Red,
}

impl DraftSide {
    pub fn label(self) -> &'static str {
        match self {
            DraftSide::Blue => "Blue",
            DraftSide::Red => "Red",
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            DraftSide::Blue => "blue",
            DraftSide::Red => "red",
        }
    }
    /// Parse the stored spelling ("blue" / "red").
    pub fn parse(s: &str) -> Option<DraftSide> {
        match s.trim().to_ascii_lowercase().as_str() {
            "blue" => Some(DraftSide::Blue),
            "red" => Some(DraftSide::Red),
            _ => None,
        }
    }
    pub fn other(self) -> DraftSide {
        match self {
            DraftSide::Blue => DraftSide::Red,
            DraftSide::Red => DraftSide::Blue,
        }
    }
}

/// Ban or pick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftAction {
    Ban,
    Pick,
}

/// The four phases of a tournament draft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftPhase {
    Ban1,
    Pick1,
    Ban2,
    Pick2,
}

impl DraftPhase {
    pub const ALL: [DraftPhase; 4] = [
        DraftPhase::Ban1,
        DraftPhase::Pick1,
        DraftPhase::Ban2,
        DraftPhase::Pick2,
    ];
    pub fn label(self) -> &'static str {
        match self {
            DraftPhase::Ban1 => "Ban phase 1",
            DraftPhase::Pick1 => "Pick phase 1",
            DraftPhase::Ban2 => "Ban phase 2",
            DraftPhase::Pick2 => "Pick phase 2",
        }
    }
}

/// One step of the draft: its order number (1..=20), phase, side, action,
/// and the per-side count ("B2" is blue's second pick, "Ban 4" red's fourth ban).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DraftStep {
    pub order: u8,
    pub phase: DraftPhase,
    pub side: DraftSide,
    pub action: DraftAction,
    pub nth: u8,
}

impl DraftStep {
    /// Short code: "B1" for blue's first pick, "R ban 4" for red's fourth ban.
    pub fn code(&self) -> String {
        let s = match self.side {
            DraftSide::Blue => "B",
            DraftSide::Red => "R",
        };
        match self.action {
            DraftAction::Pick => format!("{s}{}", self.nth),
            DraftAction::Ban => format!("{s} ban {}", self.nth),
        }
    }

    /// Spoken description: "Blue pick 1", "Red ban 4".
    pub fn describe(&self) -> String {
        let a = match self.action {
            DraftAction::Ban => "ban",
            DraftAction::Pick => "pick",
        };
        format!("{} {a} {}", self.side.label(), self.nth)
    }
}

const fn step(
    order: u8,
    phase: DraftPhase,
    side: DraftSide,
    action: DraftAction,
    nth: u8,
) -> DraftStep {
    DraftStep {
        order,
        phase,
        side,
        action,
        nth,
    }
}

use DraftAction::{Ban, Pick};
use DraftPhase::{Ban1, Ban2, Pick1, Pick2};
use DraftSide::{Blue, Red};

/// The standard tournament draft order. Index = order - 1.
pub const TOURNAMENT_ORDER: [DraftStep; 20] = [
    step(1, Ban1, Blue, Ban, 1),
    step(2, Ban1, Red, Ban, 1),
    step(3, Ban1, Blue, Ban, 2),
    step(4, Ban1, Red, Ban, 2),
    step(5, Ban1, Blue, Ban, 3),
    step(6, Ban1, Red, Ban, 3),
    step(7, Pick1, Blue, Pick, 1),
    step(8, Pick1, Red, Pick, 1),
    step(9, Pick1, Red, Pick, 2),
    step(10, Pick1, Blue, Pick, 2),
    step(11, Pick1, Blue, Pick, 3),
    step(12, Pick1, Red, Pick, 3),
    step(13, Ban2, Red, Ban, 4),
    step(14, Ban2, Blue, Ban, 4),
    step(15, Ban2, Red, Ban, 5),
    step(16, Ban2, Blue, Ban, 5),
    step(17, Pick2, Red, Pick, 4),
    step(18, Pick2, Blue, Pick, 4),
    step(19, Pick2, Blue, Pick, 5),
    step(20, Pick2, Red, Pick, 5),
];

/// The draft format. Only the tournament draft exists; the enum leaves
/// room for another format (for example Flex Queue's simultaneous bans)
/// without rewriting the board.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DraftFormat {
    #[default]
    Tournament,
}

impl DraftFormat {
    /// Every step of the format in draft order (index = slot).
    pub fn steps(self) -> &'static [DraftStep; 20] {
        match self {
            DraftFormat::Tournament => &TOURNAMENT_ORDER,
        }
    }
}

/// Slots (0-based, index = order - 1) of one side's bans in ban order.
pub fn ban_slots(side: DraftSide) -> [usize; 5] {
    match side {
        DraftSide::Blue => [0, 2, 4, 13, 15],
        DraftSide::Red => [1, 3, 5, 12, 14],
    }
}

/// Slots (0-based) of one side's picks in pick order.
pub fn pick_slots(side: DraftSide) -> [usize; 5] {
    match side {
        DraftSide::Blue => [6, 9, 10, 17, 18],
        DraftSide::Red => [7, 8, 11, 16, 19],
    }
}

/// Where a pick sits in its rotation (the picks one side makes in a row).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupPos {
    /// The rotation is this one pick.
    Single,
    /// First of two.
    First,
    /// Second of two.
    Last,
}

/// Pick rotation of a slot: its number (1..=7) and its place in the
/// rotation. Consecutive picks by the same side form one rotation.
/// `None` for bans.
pub fn pick_rotation(slot: usize) -> Option<(u8, GroupPos)> {
    let s = TOURNAMENT_ORDER.get(slot)?;
    if s.action != Pick {
        return None;
    }
    let same = |i: Option<usize>| {
        i.and_then(|i| TOURNAMENT_ORDER.get(i))
            .is_some_and(|o| o.action == Pick && o.side == s.side)
    };
    let prev = same(slot.checked_sub(1));
    let next = same(Some(slot + 1));
    let pos = match (prev, next) {
        (false, false) => GroupPos::Single,
        (false, true) => GroupPos::First,
        _ => GroupPos::Last,
    };
    // Number rotations by counting rotation starts up to this slot.
    let mut n = 0u8;
    for i in 0..=slot {
        let o = &TOURNAMENT_ORDER[i];
        if o.action != Pick {
            continue;
        }
        let starts = i == 0 || {
            let p = &TOURNAMENT_ORDER[i - 1];
            !(p.action == Pick && p.side == o.side)
        };
        if starts {
            n += 1;
        }
    }
    Some((n, pos))
}

/// "first rotation" or "second rotation" for a ban slot.
pub fn ban_rotation(slot: usize) -> Option<&'static str> {
    match TOURNAMENT_ORDER.get(slot)?.phase {
        Ban1 => Some("first rotation"),
        Ban2 => Some("second rotation"),
        _ => None,
    }
}

/// The step counter: "Ban 3 of 10 · Blue to ban", "Pick 7 of 10 · Red to
/// pick", or "Draft complete" when there is no current step.
pub fn step_status(current: Option<usize>) -> String {
    let Some(i) = current.filter(|i| *i < 20) else {
        return "Draft complete".to_string();
    };
    let s = TOURNAMENT_ORDER[i];
    let nth = TOURNAMENT_ORDER[..=i]
        .iter()
        .filter(|o| o.action == s.action)
        .count();
    match s.action {
        Ban => format!("Ban {nth} of 10 \u{00b7} {} to ban", s.side.label()),
        Pick => format!("Pick {nth} of 10 \u{00b7} {} to pick", s.side.label()),
    }
}

/// The slot an arrow key (or Home/End) moves to on the board, or `None`
/// when the key does nothing there. Left/Right cross between the columns
/// (and walk the ban row), Up/Down walk a column (bans, then picks 1 to
/// 5), Home and End jump to the first and last step of the draft.
pub fn board_move(key: &str, from: usize) -> Option<usize> {
    if from >= 20 {
        return None;
    }
    match key {
        "Home" => return Some(0),
        "End" => return Some(19),
        _ => {}
    }
    let s = TOURNAMENT_ORDER[from];
    let side = s.side;
    match s.action {
        Ban => {
            let bans = ban_slots(side);
            let x = bans.iter().position(|b| *b == from)?;
            match (key, side) {
                ("ArrowRight", _) if x < 4 => Some(bans[x + 1]),
                ("ArrowRight", DraftSide::Blue) => Some(ban_slots(DraftSide::Red)[0]),
                ("ArrowLeft", _) if x > 0 => Some(bans[x - 1]),
                ("ArrowLeft", DraftSide::Red) => Some(ban_slots(DraftSide::Blue)[4]),
                ("ArrowDown", _) => Some(pick_slots(side)[0]),
                _ => None,
            }
        }
        Pick => {
            let picks = pick_slots(side);
            let y = picks.iter().position(|p| *p == from)?;
            match (key, side) {
                ("ArrowRight", DraftSide::Blue) => Some(pick_slots(DraftSide::Red)[y]),
                ("ArrowLeft", DraftSide::Red) => Some(pick_slots(DraftSide::Blue)[y]),
                ("ArrowDown", _) if y < 4 => Some(picks[y + 1]),
                ("ArrowUp", _) if y > 0 => Some(picks[y - 1]),
                ("ArrowUp", _) => Some(ban_slots(side)[0]),
                _ => None,
            }
        }
    }
}

/// DOM id of a slot on a board.
pub fn slot_dom_id(board: &str, slot: usize) -> String {
    format!("{board}-slot-{slot}")
}

/// Accessible name of a slot: everything the visuals say.
fn slot_label(
    slot: usize,
    champion: Option<&str>,
    role: Option<&str>,
    note: Option<&str>,
    warning: Option<&str>,
    current: bool,
) -> String {
    let s = TOURNAMENT_ORDER[slot];
    let mut l = s.describe();
    match s.action {
        Ban => {
            if let Some(r) = ban_rotation(slot) {
                l.push_str(&format!(", {r}"));
            }
        }
        Pick => {
            if slot == 6 {
                l.push_str(", first pick");
            }
            if let Some((n, _)) = pick_rotation(slot) {
                l.push_str(&format!(", rotation {n}"));
            }
        }
    }
    match champion {
        Some(c) => l.push_str(&format!(": {c}")),
        None if current => {}
        None => l.push_str(": empty"),
    }
    if let Some(r) = role.and_then(Role::parse) {
        l.push_str(&format!(", {}", r.label()));
    }
    if current {
        l.push_str(if champion.is_some() {
            ", changing now"
        } else {
            ", selecting now"
        });
    }
    if let Some(w) = warning {
        l.push_str(&format!(". Warning: {w}"));
    }
    if let Some(n) = note {
        l.push_str(&format!(". Note: {n}"));
    }
    l
}

/// Read slot `i` of a per-slot list signal.
fn at(list: &Signal<Vec<Option<String>>>, i: usize) -> Option<String> {
    list.with(|v| v.get(i).cloned().flatten())
}

/// The tournament draft board: two columns (blue left, red right), each
/// with five bans grouped 3 + 2 by ban rotation, then five picks in pick
/// order bracketed by pick rotation. The current step is the one lit slot;
/// a step counter above says "Pick 7 of 10 · Red to pick".
///
/// A11y: a `role="grid"` named by `label`, with a header row (column
/// headers "Blue side", "Red side"), a ban row and five pick rows of
/// `role="gridcell"` slots. One Tab stop (roving tabindex): arrow keys move
/// between slots, Home/End jump to the first and last step, Enter (or
/// Space) chooses the slot (`on_select`, then `on_open`, which opens the
/// champion picker), Backspace or Delete clears a filled slot
/// (`on_clear`). Each slot's name says its step, rotation, champion, role,
/// note and state ("Red pick 2, rotation 2: Syndra, Mid"); the current one
/// has `aria-current="step"`. The step counter is a polite live region.
/// Sides are a circle and a square plus words, never colour alone.
#[component]
pub fn DraftBoard(
    /// Prefix for DOM ids; unique on the page.
    id: &'static str,
    /// Accessible name of the grid ("Draft board").
    #[prop(into)]
    label: String,
    /// Champion per slot, index = order - 1 (20 entries; missing = empty).
    #[prop(into)]
    champions: Signal<Vec<Option<String>>>,
    /// Role per slot (picks), any stored spelling.
    #[prop(optional, into)]
    roles: Signal<Vec<Option<String>>>,
    /// A short note per slot (pick rationale), shown under the name.
    #[prop(optional, into)]
    notes: Signal<Vec<Option<String>>>,
    /// A warning per slot ("Not in Kim's pool"), shown with a "!" glyph.
    #[prop(optional, into)]
    warnings: Signal<Vec<Option<String>>>,
    /// The step being decided (0-based slot), `None` when complete.
    #[prop(optional, into)]
    current: Signal<Option<usize>>,
    /// Team names for the column headers.
    #[prop(optional, into)]
    blue_team: Signal<Option<String>>,
    #[prop(optional, into)] red_team: Signal<Option<String>>,
    /// Click, Enter or Space on a slot (0-based). Makes the board editable.
    #[prop(optional)]
    on_select: Option<Callback<usize>>,
    /// Enter or Space after `on_select`: open the champion picker.
    #[prop(optional)]
    on_open: Option<Callback<usize>>,
    /// Backspace or Delete on a filled slot.
    #[prop(optional)]
    on_clear: Option<Callback<usize>>,
    /// A champion dropped on a slot (pointer only; the keyboard path is
    /// Enter and the picker).
    #[prop(optional)]
    on_drop: Option<Callback<(usize, String)>>,
    /// Show the keycap legend under the board.
    #[prop(optional)]
    hints: bool,
    #[prop(optional)] format: DraftFormat,
) -> impl IntoView {
    let steps = format.steps();
    let interactive = on_select.is_some();
    let focus_idx: RwSignal<Option<usize>> = RwSignal::new(None);
    let tab_stop = Memo::new(move |_| focus_idx.get().or(current.get()).unwrap_or(0));
    let status_id = format!("{id}-status");

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let from = focus_idx
            .get_untracked()
            .unwrap_or_else(|| tab_stop.get_untracked());
        let key = ev.key();
        if let Some(next) = board_move(&key, from) {
            ev.prevent_default();
            focus_idx.set(Some(next));
            focus_id(&slot_dom_id(id, next));
            return;
        }
        match key.as_str() {
            "Enter" | " " => {
                if let Some(cb) = on_select {
                    ev.prevent_default();
                    cb.run(from);
                    if let Some(open) = on_open {
                        open.run(from);
                    }
                }
            }
            "Backspace" | "Delete" => {
                if let Some(cb) = on_clear {
                    if at(&champions, from).is_some() {
                        ev.prevent_default();
                        cb.run(from);
                    }
                }
            }
            _ => {}
        }
    };

    // One slot (ban square or pick row).
    let cell = move |slot: usize| {
        let s = steps[slot];
        let is_ban = s.action == Ban;
        let is_current = Memo::new(move |_| current.get() == Some(slot));
        let champ = Memo::new(move |_| at(&champions, slot));
        let label = move || {
            slot_label(
                slot,
                champ.get().as_deref(),
                at(&roles, slot).as_deref(),
                at(&notes, slot).as_deref(),
                at(&warnings, slot).as_deref(),
                is_current.get(),
            )
        };
        let class = move || {
            let filled = champ.with(|c| c.is_some());
            let state = match (is_current.get(), filled) {
                (true, false) => "border-dashed border-accent bg-accent-soft",
                (true, true) => "border-accent bg-accent-soft",
                (false, true) => "border-outline bg-ink-1",
                (false, false) => "border-dashed border-outline",
            };
            let shape = if is_ban {
                "relative grid size-6 shrink-0 place-items-center overflow-hidden rounded-ctl border sm:size-8"
            } else {
                "relative flex h-11 w-full min-w-0 items-center gap-2 rounded-ctl border px-1.5"
            };
            join(&[
                shape,
                state,
                if interactive {
                    "cursor-pointer transition-colors hover:bg-ink-2"
                } else {
                    ""
                },
            ])
        };
        let dom_id = slot_dom_id(id, slot);
        let inner = move || {
            let name = champ.get();
            if is_ban {
                return match name {
                    Some(n) => view! {
                        <ChampionAvatar name=n size=AvatarSize::Md state=AvatarState::Banned decorative=true />
                    }
                    .into_any(),
                    None => ().into_any(),
                };
            }
            let role = at(&roles, slot).as_deref().and_then(Role::parse);
            let note = at(&notes, slot);
            let warning = at(&warnings, slot);
            let lit = is_current.get();
            let title = match (&name, lit) {
                (Some(n), _) => view! {
                    <span class="truncate text-label font-semibold text-primary">{n.clone()}</span>
                }
                .into_any(),
                (None, true) => view! {
                    <span class="truncate text-label font-semibold text-accent">"Selecting\u{2026}"</span>
                }
                .into_any(),
                (None, false) => ().into_any(),
            };
            let caption = match (warning, note) {
                (Some(w), _) => Some(
                    view! {
                        <span class="flex min-w-0 items-center gap-1 text-caption text-warning">
                            <span class="figures font-semibold" aria-hidden="true">"!"</span>
                            <span class="truncate">{w}</span>
                        </span>
                    }
                    .into_any(),
                ),
                (None, Some(n)) => Some(
                    view! {
                        <span class="truncate text-caption text-muted">{n}</span>
                    }
                    .into_any(),
                ),
                (None, None) if slot == 6 => Some(
                    view! {
                        <span class="truncate text-caption text-muted">"First pick"</span>
                    }
                    .into_any(),
                ),
                _ => None,
            };
            view! {
                {match name {
                    Some(n) => view! { <ChampionAvatar name=n size=AvatarSize::Md decorative=true /> }.into_any(),
                    None => view! { <span class="size-8 shrink-0" aria-hidden="true"></span> }.into_any(),
                }}
                <span class="flex min-w-0 flex-1 flex-col leading-tight">
                    {title}
                    {caption}
                </span>
                {role.map(|r| view! { <RoleLabel role=r /> })}
            }
            .into_any()
        };
        view! {
            <div
                id=dom_id
                role="gridcell"
                class=class
                tabindex=move || if tab_stop.get() == slot { "0" } else { "-1" }
                aria-label=label
                aria-current=move || is_current.get().then_some("step")
                aria-selected=move || if is_current.get() { "true" } else { "false" }
                data-slot=slot
                data-step=s.order
                data-side=s.side.as_str()
                data-action=if is_ban { "ban" } else { "pick" }
                data-filled=move || if champ.with(|c| c.is_some()) { "true" } else { "false" }
                draggable=move || if interactive && champ.with(|c| c.is_some()) { "true" } else { "false" }
                on:focus=move |_| focus_idx.set(Some(slot))
                on:click=move |_| {
                    focus_idx.set(Some(slot));
                    if let Some(cb) = on_select {
                        cb.run(slot);
                    }
                }
                on:dragstart=move |ev: web_sys::DragEvent| {
                    if let (Some(name), Some(dt)) = (champ.get_untracked(), ev.data_transfer()) {
                        let _ = dt.set_data("text/plain", &name);
                        let _ = dt.set_data("text/x-source-slot", &slot.to_string());
                    }
                }
                on:dragover=move |ev: web_sys::DragEvent| {
                    if on_drop.is_some() {
                        ev.prevent_default();
                    }
                }
                on:drop=move |ev: web_sys::DragEvent| {
                    let Some(cb) = on_drop else { return };
                    ev.prevent_default();
                    let Some(dt) = ev.data_transfer() else { return };
                    let source = dt
                        .get_data("text/x-source-slot")
                        .ok()
                        .and_then(|s| s.parse::<usize>().ok());
                    if let Ok(name) = dt.get_data("text/plain") {
                        if !name.is_empty() && source != Some(slot) {
                            if let (Some(src), Some(clear)) = (source, on_clear) {
                                clear.run(src);
                            }
                            cb.run((slot, name));
                        }
                    }
                }
            >
                {inner}
            </div>
        }
    };

    // A pick slot with its rotation bracket in the outer gutter.
    let pick_cell = move |slot: usize| {
        let side = steps[slot].side;
        let (n, pos) = pick_rotation(slot).unwrap_or((0, GroupPos::Single));
        let bracket = match (side, pos) {
            (DraftSide::Blue, GroupPos::Single) => "left-3 top-1 bottom-1 border-y border-l",
            (DraftSide::Blue, GroupPos::First) => "left-3 top-1 -bottom-1.5 border-t border-l",
            (DraftSide::Blue, GroupPos::Last) => "left-3 top-0 bottom-1 border-b border-l",
            (DraftSide::Red, GroupPos::Single) => "right-3 top-1 bottom-1 border-y border-r",
            (DraftSide::Red, GroupPos::First) => "right-3 top-1 -bottom-1.5 border-t border-r",
            (DraftSide::Red, GroupPos::Last) => "right-3 top-0 bottom-1 border-b border-r",
        };
        let number_pos = match side {
            DraftSide::Blue => "left-0",
            DraftSide::Red => "right-0",
        };
        let wrap = match side {
            DraftSide::Blue => "relative min-w-0 pl-5",
            DraftSide::Red => "relative min-w-0 pr-5",
        };
        let show_number = pos != GroupPos::Last;
        view! {
            <div role="none" class=wrap>
                <span class=format!("pointer-events-none absolute w-1.5 border-outline {bracket}") aria-hidden="true"></span>
                {show_number.then(|| view! {
                    <span
                        class=format!("figures pointer-events-none absolute top-1 text-caption leading-none text-muted {number_pos}")
                        aria-hidden="true"
                    >{n}</span>
                })}
                {cell(slot)}
            </div>
        }
    };

    let ban_row = move |side: DraftSide| {
        let bans = ban_slots(side);
        let wrap = match side {
            DraftSide::Blue => "flex items-center gap-2 pl-5 sm:gap-3",
            DraftSide::Red => "flex items-center gap-2 pr-5 sm:gap-3",
        };
        view! {
            <div role="none" class=wrap>
                <div role="none" class="flex gap-0.5 sm:gap-1">
                    {bans[..3].iter().map(|&b| cell(b)).collect_view()}
                </div>
                <div role="none" class="flex gap-0.5 sm:gap-1">
                    {bans[3..].iter().map(|&b| cell(b)).collect_view()}
                </div>
            </div>
        }
    };

    let header = move |side: DraftSide| {
        let team = match side {
            DraftSide::Blue => blue_team,
            DraftSide::Red => red_team,
        };
        let wrap = match side {
            DraftSide::Blue => "flex min-w-0 flex-col gap-1 pb-1 pl-5",
            DraftSide::Red => "flex min-w-0 flex-col gap-1 pb-1 pr-5",
        };
        view! {
            <div role="columnheader" class=wrap>
                <span class="flex min-h-5 items-center gap-2">
                    <span class="font-semibold"><SideMarker side=side text=SideText::Long /></span>
                    {(side == DraftSide::Blue).then(|| view! { <Badge>"First pick"</Badge> })}
                </span>
                <span class="min-h-5 truncate text-label text-primary">
                    {move || team.get().unwrap_or_default()}
                </span>
            </div>
        }
    };

    let pick_row = move |y: usize| {
        view! {
            <div role="row" class="contents">
                {pick_cell(pick_slots(DraftSide::Blue)[y])}
                {pick_cell(pick_slots(DraftSide::Red)[y])}
            </div>
        }
    };

    view! {
        <div class="flex min-w-0 flex-col gap-3" data-draft-board=id>
            <p
                id=status_id.clone()
                class="font-display text-heading font-semibold text-primary"
                aria-live="polite"
                data-draft-status=""
            >
                {move || match current.get() {
                    Some(c) => step_status(Some(c)),
                    None => {
                        let filled = champions.with(|v| v.iter().filter(|c| c.is_some()).count());
                        if filled >= 20 {
                            step_status(None)
                        } else {
                            format!("{filled} of 20 steps decided")
                        }
                    }
                }}
            </p>
            <div
                role="grid"
                aria-label=label
                aria-describedby=status_id
                class="grid grid-cols-2 gap-x-3 gap-y-1.5 sm:gap-x-6"
                on:keydown=on_keydown
            >
                <div role="row" class="contents">
                    {header(DraftSide::Blue)}
                    {header(DraftSide::Red)}
                </div>
                <div role="row" class="contents">
                    {ban_row(DraftSide::Blue)}
                    {ban_row(DraftSide::Red)}
                </div>
                <span class="col-span-2 h-1.5" aria-hidden="true"></span>
                {pick_row(0)}
                {pick_row(1)}
                {pick_row(2)}
                <div class="col-span-2 flex items-center gap-3 px-5 py-0.5 text-caption text-muted" aria-hidden="true">
                    <span class="h-px flex-1 border-t border-dashed border-rule"></span>
                    "bans 4 and 5"
                    <span class="h-px flex-1 border-t border-dashed border-rule"></span>
                </div>
                {pick_row(3)}
                {pick_row(4)}
            </div>
            <p class="px-5 text-caption text-muted">
                "Brackets group the picks a side makes in one turn; the numbers give the turn order."
            </p>
            {hints.then(|| view! {
                <p class="flex flex-wrap items-center gap-x-3 gap-y-1.5 px-5 text-caption text-muted" data-draft-keys="">
                    <span class="inline-flex items-center gap-1"><Keycap keys="\u{2190}" /><Keycap keys="\u{2191}" /><Keycap keys="\u{2192}" /><Keycap keys="\u{2193}" />" move"</span>
                    <span class="inline-flex items-center gap-1"><Keycap keys="Enter" />" choose champion"</span>
                    <span class="inline-flex items-center gap-1"><Keycap keys="Esc" />" back to the board"</span>
                    <span class="inline-flex items-center gap-1"><Keycap keys="Backspace" />" clear"</span>
                </p>
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::draft_board::slot_meta;

    #[test]
    fn order_numbers_run_one_to_twenty() {
        for (i, s) in TOURNAMENT_ORDER.iter().enumerate() {
            assert_eq!(s.order as usize, i + 1);
        }
    }

    #[test]
    fn five_bans_and_five_picks_per_side() {
        for side in [DraftSide::Blue, DraftSide::Red] {
            for action in [DraftAction::Ban, DraftAction::Pick] {
                let nths: Vec<u8> = TOURNAMENT_ORDER
                    .iter()
                    .filter(|s| s.side == side && s.action == action)
                    .map(|s| s.nth)
                    .collect();
                assert_eq!(nths, vec![1, 2, 3, 4, 5], "{side:?} {action:?}");
            }
        }
    }

    #[test]
    fn matches_the_tournament_pattern() {
        let code: String = TOURNAMENT_ORDER
            .iter()
            .map(|s| match s.side {
                DraftSide::Blue => 'B',
                DraftSide::Red => 'R',
            })
            .collect();
        // Ban 1: BRBRBR, Pick 1: BRRBBR, Ban 2: RBRB, Pick 2: RBBR.
        assert_eq!(code, "BRBRBRBRRBBRRBRBRBBR");
        let picks: Vec<String> = TOURNAMENT_ORDER
            .iter()
            .filter(|s| s.action == DraftAction::Pick)
            .map(|s| s.code())
            .collect();
        assert_eq!(
            picks,
            ["B1", "R1", "R2", "B2", "B3", "R3", "R4", "B4", "B5", "R5"]
        );
    }

    #[test]
    fn agrees_with_the_draft_board_slot_order() {
        for (i, s) in TOURNAMENT_ORDER.iter().enumerate() {
            let (side, action, _) = slot_meta(i);
            assert_eq!(side, s.side.as_str(), "step {}", s.order);
            assert_eq!(
                action,
                match s.action {
                    DraftAction::Ban => "ban",
                    DraftAction::Pick => "pick",
                },
                "step {}",
                s.order
            );
        }
    }

    #[test]
    fn side_slot_lists_cover_the_order() {
        for side in [DraftSide::Blue, DraftSide::Red] {
            for (k, slot) in ban_slots(side).iter().enumerate() {
                let s = TOURNAMENT_ORDER[*slot];
                assert_eq!((s.side, s.action, s.nth), (side, Ban, k as u8 + 1));
            }
            for (k, slot) in pick_slots(side).iter().enumerate() {
                let s = TOURNAMENT_ORDER[*slot];
                assert_eq!((s.side, s.action, s.nth), (side, Pick, k as u8 + 1));
            }
        }
    }

    #[test]
    fn pick_rotations_group_consecutive_picks() {
        // (B1) (R1 R2) (B2 B3) (R3) | (R4) (B4 B5) (R5)
        let rot: Vec<(usize, u8, GroupPos)> = (0..20)
            .filter_map(|i| pick_rotation(i).map(|(n, p)| (i, n, p)))
            .collect();
        use GroupPos::*;
        assert_eq!(
            rot,
            vec![
                (6, 1, Single),
                (7, 2, First),
                (8, 2, Last),
                (9, 3, First),
                (10, 3, Last),
                (11, 4, Single),
                (16, 5, Single),
                (17, 6, First),
                (18, 6, Last),
                (19, 7, Single),
            ]
        );
        assert_eq!(pick_rotation(0), None);
        assert_eq!(ban_rotation(4), Some("first rotation"));
        assert_eq!(ban_rotation(13), Some("second rotation"));
    }

    #[test]
    fn step_counter_counts_bans_and_picks_separately() {
        assert_eq!(step_status(Some(0)), "Ban 1 of 10 \u{00b7} Blue to ban");
        assert_eq!(step_status(Some(12)), "Ban 7 of 10 \u{00b7} Red to ban");
        assert_eq!(step_status(Some(6)), "Pick 1 of 10 \u{00b7} Blue to pick");
        assert_eq!(step_status(Some(16)), "Pick 7 of 10 \u{00b7} Red to pick");
        assert_eq!(step_status(None), "Draft complete");
    }

    #[test]
    fn arrow_keys_walk_the_board() {
        // Ban row: right along blue's bans, then across to red's.
        assert_eq!(board_move("ArrowRight", 0), Some(2));
        assert_eq!(board_move("ArrowRight", 15), Some(1));
        assert_eq!(board_move("ArrowLeft", 1), Some(15));
        assert_eq!(board_move("ArrowRight", 14), None);
        // Down from the bans to pick 1, down the column, up back to bans.
        assert_eq!(board_move("ArrowDown", 4), Some(6));
        assert_eq!(board_move("ArrowDown", 6), Some(9));
        assert_eq!(board_move("ArrowDown", 18), None);
        assert_eq!(board_move("ArrowUp", 6), Some(0));
        assert_eq!(board_move("ArrowUp", 8), Some(7));
        // Across a pick row.
        assert_eq!(board_move("ArrowRight", 9), Some(8));
        assert_eq!(board_move("ArrowLeft", 19), Some(18));
        assert_eq!(board_move("ArrowRight", 19), None);
        assert_eq!(board_move("Home", 12), Some(0));
        assert_eq!(board_move("End", 3), Some(19));
        assert_eq!(board_move("Enter", 3), None);
    }

    #[test]
    fn slot_names_say_everything() {
        assert_eq!(
            slot_label(6, None, None, None, None, true),
            "Blue pick 1, first pick, rotation 1, selecting now"
        );
        assert_eq!(
            slot_label(8, Some("Syndra"), Some("mid"), None, None, false),
            "Red pick 2, rotation 2: Syndra, Mid"
        );
        assert_eq!(
            slot_label(12, None, None, None, None, false),
            "Red ban 4, second rotation: empty"
        );
    }
}

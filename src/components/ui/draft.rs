//! Tournament draft: the fixed 20-step order, one slot per step, and the
//! sequence layout that shows the order with the current step highlighted.
//!
//! Order (blue starts, red opens ban phase 2):
//! - Ban phase 1: B R B R B R (three bans each)
//! - Pick phase 1: B1, R1, R2, B2, B3, R3
//! - Ban phase 2: R B R B (two bans each)
//! - Pick phase 2: R4, B4, B5, R5

use leptos::prelude::*;

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

/// State of one slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlotState {
    /// Not reached yet.
    #[default]
    Empty,
    /// The step being decided now.
    Selecting,
    /// Decided.
    Locked,
}

impl SlotState {
    pub fn label(self) -> &'static str {
        match self {
            SlotState::Empty => "not yet",
            SlotState::Selecting => "selecting now",
            SlotState::Locked => "locked",
        }
    }
}

/// State of each of the 20 steps given the champions so far and the
/// current step (1-based; `None` once the draft is complete).
pub fn slot_states(champions: &[Option<String>], current: Option<u8>) -> [SlotState; 20] {
    let mut out = [SlotState::Empty; 20];
    for (i, s) in out.iter_mut().enumerate() {
        let order = i as u8 + 1;
        let filled = champions.get(i).is_some_and(|c| c.is_some());
        *s = if current == Some(order) {
            SlotState::Selecting
        } else if filled {
            SlotState::Locked
        } else {
            SlotState::Empty
        };
    }
    out
}

/// One step of the draft board.
///
/// A11y: a list item whose accessible name says everything the visuals do
/// ("Step 19, Blue pick 5, Support: selecting now"); side is also shown by
/// shape (round blue marker, square red marker) and lane, never colour
/// alone. The selecting step has `aria-current="step"`. With `on_select` it
/// renders a real `<button>` (Enter/Space); otherwise it is static.
#[component]
pub fn DraftSlot(
    step: DraftStep,
    #[prop(optional)] state: SlotState,
    #[prop(optional_no_strip)] champion: Option<String>,
    /// Role for picks ("Top", "Support"), shown in the slot.
    #[prop(optional_no_strip)]
    role: Option<String>,
    #[prop(optional_no_strip)] on_select: Option<Callback<u8>>,
) -> impl IntoView {
    let is_ban = step.action == DraftAction::Ban;
    let box_class = join(&[
        "flex w-full min-w-0 items-center gap-2 rounded-ctl border px-2 py-1.5 text-left",
        match state {
            SlotState::Empty => "border-dashed border-rule",
            SlotState::Selecting => "border-dashed border-accent bg-accent-soft",
            SlotState::Locked => "border-rule bg-ink-1",
        },
        if on_select.is_some() {
            "cursor-pointer transition-colors hover:border-outline"
        } else {
            ""
        },
    ]);
    let marker = match step.side {
        DraftSide::Blue => "size-2.5 shrink-0 rounded-full bg-side-blue",
        DraftSide::Red => "size-2.5 shrink-0 rounded-[1px] bg-side-red",
    };
    let name_text = match (state, &champion) {
        (SlotState::Selecting, _) => "Selecting\u{2026}".to_string(),
        (_, Some(c)) => c.clone(),
        (SlotState::Empty, None) => "\u{2014}".to_string(),
        (SlotState::Locked, None) => "No champion".to_string(),
    };
    let name_class = match state {
        SlotState::Selecting => "truncate text-label font-semibold text-accent",
        SlotState::Empty => "truncate text-label text-muted",
        SlotState::Locked if is_ban => "truncate text-label text-muted line-through decoration-1",
        SlotState::Locked => "truncate text-label font-semibold text-primary",
    };
    let label = {
        let mut l = format!("Step {}, {}", step.order, step.describe());
        if let Some(r) = &role {
            l.push_str(&format!(", {r}"));
        }
        match (&champion, state) {
            (_, SlotState::Selecting) => l.push_str(": selecting now"),
            (Some(c), _) => l.push_str(&format!(": {c}, {}", state.label())),
            (None, s) => l.push_str(&format!(": {}", s.label())),
        }
        l
    };
    let caption = match (&role, step.action) {
        (Some(r), DraftAction::Pick) => format!("{} \u{00b7} {r}", step.code()),
        _ => step.code(),
    };
    let inner = view! {
        <span class=marker aria-hidden="true"></span>
        <span class="figures w-5 shrink-0 text-caption text-muted" aria-hidden="true">
            {format!("{:02}", step.order)}
        </span>
        <span class="flex min-w-0 flex-col leading-tight" aria-hidden="true">
            <span class="truncate text-caption text-muted">{caption}</span>
            <span class=name_class>{name_text}</span>
        </span>
    };
    let current = (state == SlotState::Selecting).then_some("step");
    let order = step.order;
    match on_select {
        Some(cb) => view! {
            <button
                type="button"
                class=box_class
                aria-label=label
                aria-current=current
                data-step=order
                on:click=move |_| cb.run(order)
            >
                {inner}
            </button>
        }
        .into_any(),
        None => view! {
            <div class=box_class role="img" aria-label=label aria-current=current data-step=order>
                {inner}
            </div>
        }
        .into_any(),
    }
}

/// The whole draft in order: four phases, each with a blue and a red lane,
/// so the order (B R B R B R, B R R B B R, R B R B, R B B R) reads top to
/// bottom.
///
/// A11y: each phase is a named group (`section` with a heading) holding an
/// ordered list of steps in draft order; the current step is announced by
/// its slot (`aria-current="step"`). The lanes are visual only; every slot
/// names its side.
#[component]
pub fn DraftSequence(
    /// Champion per step, index = order - 1 (missing entries are empty).
    champions: Vec<Option<String>>,
    /// Role per step (picks only), index = order - 1.
    #[prop(optional)]
    roles: Vec<Option<String>>,
    /// The step being decided (1..=20), or `None` when complete.
    #[prop(optional)]
    current: Option<u8>,
    #[prop(optional)] on_select: Option<Callback<u8>>,
    /// Heading level for the phase titles (3 by default).
    #[prop(optional)]
    heading_h4: bool,
) -> impl IntoView {
    let states = slot_states(&champions, current);
    view! {
        <div class="grid grid-cols-1 gap-x-6 gap-y-6 sm:grid-cols-2 xl:grid-cols-4">
            {DraftPhase::ALL
                .into_iter()
                .map(|phase| {
                    let steps: Vec<DraftStep> = TOURNAMENT_ORDER
                        .iter()
                        .copied()
                        .filter(|s| s.phase == phase)
                        .collect();
                    let first = steps.first().map(|s| s.order).unwrap_or(0);
                    let last = steps.last().map(|s| s.order).unwrap_or(0);
                    let active = current.is_some_and(|c| c >= first && c <= last);
                    let title = phase.label();
                    let range = format!("{first}\u{2013}{last}");
                    let group_label = format!("{title}, steps {range}");
                    let heading = if heading_h4 {
                        view! { <h4 class="text-label font-semibold text-primary">{title}</h4> }.into_any()
                    } else {
                        view! { <h3 class="text-label font-semibold text-primary">{title}</h3> }.into_any()
                    };
                    view! {
                        <section class="min-w-0" aria-label=group_label>
                            <div class=if active {
                                "mb-2 flex items-baseline justify-between gap-3 border-b-2 border-accent pb-1.5"
                            } else {
                                "mb-2 flex items-baseline justify-between gap-3 border-b border-rule pb-1.5"
                            }>
                                {heading}
                                <span class="figures text-caption text-muted">{range}</span>
                            </div>
                            <div class="mb-1 grid grid-cols-2 gap-2 text-caption text-muted" aria-hidden="true">
                                <span class="inline-flex items-center gap-1.5">
                                    <span class="size-2 rounded-full bg-side-blue"></span>"Blue"
                                </span>
                                <span class="inline-flex items-center gap-1.5">
                                    <span class="size-2 rounded-[1px] bg-side-red"></span>"Red"
                                </span>
                            </div>
                            <ol class="flex flex-col gap-1.5">
                                {steps
                                    .into_iter()
                                    .map(|s| {
                                        let i = (s.order - 1) as usize;
                                        let champ = champions.get(i).cloned().flatten();
                                        let role = roles.get(i).cloned().flatten();
                                        let state = states[i];
                                        let slot = view! {
                                            <DraftSlot
                                                step=s
                                                state=state
                                                champion=champ
                                                role=role
                                                on_select=on_select
                                            />
                                        };
                                        let lane = "h-px self-center border-t border-dotted border-rule";
                                        match s.side {
                                            DraftSide::Blue => view! {
                                                <li class="grid grid-cols-2 items-center gap-2">
                                                    {slot}
                                                    <span class=lane aria-hidden="true"></span>
                                                </li>
                                            }
                                            .into_any(),
                                            DraftSide::Red => view! {
                                                <li class="grid grid-cols-2 items-center gap-2">
                                                    <span class=lane aria-hidden="true"></span>
                                                    {slot}
                                                </li>
                                            }
                                            .into_any(),
                                        }
                                    })
                                    .collect_view()}
                            </ol>
                        </section>
                    }
                })
                .collect_view()}
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
    fn states_follow_the_current_step() {
        let mut champs = vec![None; 20];
        champs[0] = Some("Yasuo".to_string());
        let s = slot_states(&champs, Some(2));
        assert_eq!(s[0], SlotState::Locked);
        assert_eq!(s[1], SlotState::Selecting);
        assert_eq!(s[2], SlotState::Empty);
        let done = slot_states(&vec![Some("x".to_string()); 20], None);
        assert!(done.iter().all(|s| *s == SlotState::Locked));
    }
}

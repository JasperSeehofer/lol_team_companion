//! SegmentedControl (one choice out of two to five, shown side by side),
//! Switch (on or off, effective immediately) and Checkbox (a checklist
//! item or a yes/no in a form).

use leptos::prelude::*;

use super::a11y::{focus_id, roving_index};
use super::button::Force;

/// One option of a [`SegmentedControl`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub value: String,
    pub label: String,
}

impl Segment {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// Height of a [`SegmentedControl`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentSize {
    /// 28px: toolbars, the nav.
    Sm,
    /// 32px.
    #[default]
    Md,
}

impl SegmentSize {
    fn class(self) -> &'static str {
        match self {
            SegmentSize::Sm => "h-6 px-2.5 text-label",
            SegmentSize::Md => "h-7 px-3 text-label",
        }
    }
}

/// DOM id of a segment.
pub fn segment_id(group: &str, index: usize) -> String {
    format!("{group}-seg-{index}")
}

const SEG_ON: &str =
    "cursor-pointer whitespace-nowrap rounded-[2px] bg-ink-3 font-semibold text-primary";
const SEG_OFF: &str = "cursor-pointer whitespace-nowrap rounded-[2px] text-muted transition-colors hover:text-primary data-[force=hover]:text-primary";

/// A single choice shown as adjacent segments (Solo / Team, Blue / Red).
/// Use it for two to five short options that switch a view or a mode
/// immediately; for tabs with panels use `Tabs`, for long lists `Select`.
///
/// A11y: `role="radiogroup"` with a name, segments are `role="radio"` with
/// `aria-checked`. Roving tabindex: one Tab stop (the checked segment, or
/// the first one when nothing is checked); Left/Right and Up/Down move and
/// select (wrapping), Home/End jump. With `manual` the arrows only move
/// focus and Enter, Space or a click selects: use it when selecting has a
/// side effect such as a reload (the nav's workspace switch). The checked
/// segment is marked by fill and weight, not colour alone.
#[component]
pub fn SegmentedControl(
    /// Prefix for DOM ids; unique on the page.
    id: &'static str,
    /// Accessible name of the group ("Workspace").
    #[prop(into)]
    label: String,
    options: Vec<Segment>,
    value: RwSignal<String>,
    #[prop(optional)] size: SegmentSize,
    /// Runs after the value changes (for example to save it).
    #[prop(optional)]
    on_change: Option<Callback<String>>,
    #[prop(optional, into)] disabled: Signal<bool>,
    /// Manual activation: arrow keys move focus only; Enter, Space or a
    /// click selects.
    #[prop(optional)]
    manual: bool,
    /// Gallery only: force a state on the first unchecked segment.
    #[prop(optional)]
    force: Option<Force>,
) -> impl IntoView {
    let values: Vec<String> = options.iter().map(|o| o.value.clone()).collect();
    let values = StoredValue::new(values);
    let len = options.len();
    let select = move |i: usize| {
        let Some(v) = values.with_value(|vs| vs.get(i).cloned()) else {
            return;
        };
        if value.get_untracked() != v {
            value.set(v.clone());
            if let Some(cb) = on_change {
                cb.run(v);
            }
        }
    };
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if disabled.get_untracked() {
            return;
        }
        let key = ev.key();
        let cur = if manual { focused_index(id, len) } else { None }.or_else(|| {
            values.with_value(|vs| vs.iter().position(|v| *v == value.get_untracked()))
        });
        let horizontal = roving_index(&key, cur, len, false);
        let vertical = roving_index(&key, cur, len, true);
        if let Some(next) = horizontal.or(vertical) {
            ev.prevent_default();
            if !manual {
                select(next);
            }
            focus_id(&segment_id(id, next));
        }
    };
    // Nothing checked (a value no option has): the first segment takes the
    // Tab stop so the group stays reachable.
    let none_checked = Memo::new(move |_| {
        let v = value.get();
        values.with_value(|vs| !vs.contains(&v))
    });
    let first_unchecked_forced = StoredValue::new(false);
    view! {
        <div
            role="radiogroup"
            aria-label=label
            aria-disabled=move || disabled.get().then_some("true")
            class="inline-flex items-stretch gap-0.5 rounded-ctl border border-outline p-0.5 aria-disabled:opacity-50"
            data-segmented=id
            on:keydown=on_keydown
        >
            {options
                .into_iter()
                .enumerate()
                .map(|(i, opt)| {
                    let v_checked = opt.value.clone();
                    let checked = Memo::new(move |_| value.get() == v_checked);
                    let seg_force = force.filter(|_| {
                        let take = !first_unchecked_forced.get_value() && i > 0;
                        if take {
                            first_unchecked_forced.set_value(true);
                        }
                        take
                    });
                    let class = move || {
                        let state = if checked.get() { SEG_ON } else { SEG_OFF };
                        format!("{state} {}", size.class())
                    };
                    view! {
                        <button
                            type="button"
                            role="radio"
                            id=segment_id(id, i)
                            aria-checked=move || if checked.get() { "true" } else { "false" }
                            tabindex=move || {
                                if checked.get() || (i == 0 && none_checked.get()) { "0" } else { "-1" }
                            }
                            disabled=move || disabled.get()
                            data-force=seg_force.map(Force::as_str)
                            data-value=opt.value.clone()
                            class=class
                            on:click=move |_| select(i)
                        >
                            {opt.label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Index of the segment that has focus, if focus is inside the group.
fn focused_index(group: &str, len: usize) -> Option<usize> {
    #[cfg(feature = "hydrate")]
    {
        let active = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element())
            .map(|e| e.id())?;
        (0..len).find(|i| segment_id(group, *i) == active)
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (group, len);
        None
    }
}

/// An on/off switch with a visible label, for settings that apply at once
/// (no Save button). For choices that are submitted with a form use a
/// checkbox.
///
/// A11y: a `<button role="switch">` with `aria-checked`; Space and Enter
/// toggle (native button). The visible label is the accessible name (or
/// `aria-label` when `label_hidden`); the state is drawn by the thumb
/// position and the fill, and written out as "On"/"Off" next to it when
/// `show_state` is set. Track and thumb are round (toggles are one of the
/// two round things).
#[component]
pub fn Switch(
    #[prop(into)] label: String,
    checked: RwSignal<bool>,
    /// Hide the visible label (the switch still has it as its name).
    #[prop(optional)]
    label_hidden: bool,
    /// Write "On"/"Off" after the switch.
    #[prop(optional)]
    show_state: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(optional)] force: Option<Force>,
) -> impl IntoView {
    let label_for_aria = label_hidden.then(|| label.clone());
    let toggle = move |_| {
        let next = !checked.get_untracked();
        checked.set(next);
        if let Some(cb) = on_change {
            cb.run(next);
        }
    };
    view! {
        <button
            type="button"
            role="switch"
            aria-checked=move || if checked.get() { "true" } else { "false" }
            aria-label=label_for_aria
            disabled=move || disabled.get()
            data-force=force.map(Force::as_str)
            class="group/sw inline-flex min-h-6 cursor-pointer items-center gap-2.5 text-label text-primary disabled:cursor-not-allowed disabled:opacity-50"
            on:click=toggle
        >
            <span
                class="relative inline-flex h-5 w-9 shrink-0 items-center rounded-full border border-outline bg-ink-2 transition-colors group-hover/sw:border-secondary group-data-[force=hover]/sw:border-secondary group-aria-checked/sw:border-accent group-aria-checked/sw:bg-accent"
                aria-hidden="true"
            >
                <span class="ml-0.5 size-3.5 rounded-full bg-secondary transition-transform group-aria-checked/sw:translate-x-4 group-aria-checked/sw:bg-accent-ink"></span>
            </span>
            {(!label_hidden).then(|| view! { <span>{label}</span> })}
            {show_state.then(|| view! {
                <span class="figures text-caption text-muted" aria-hidden="true">
                    {move || if checked.get() { "On" } else { "Off" }}
                </span>
            })}
        </button>
    }
}

/// A checkbox with its label: one item of a checklist, or a yes/no inside a
/// form. A square box with a check mark (round is for switches).
///
/// A11y: a native `<input type="checkbox">` inside its `<label>`, so the
/// text is the name and Space toggles it; the checked state is the mark and
/// the fill, and `strike` also crosses the text out (decorative, the state
/// is read from the box).
#[component]
pub fn Checkbox(
    #[prop(into)] label: String,
    checked: RwSignal<bool>,
    /// Cross the label out while checked (done items in a checklist).
    #[prop(optional)]
    strike: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_change: Option<Callback<bool>>,
    #[prop(optional, into)] id: Option<String>,
) -> impl IntoView {
    let text_class = move || {
        if strike && checked.get() {
            "text-body text-muted line-through"
        } else {
            "text-body text-primary"
        }
    };
    view! {
        <label class="group/cb inline-flex min-h-6 cursor-pointer items-start gap-2.5 has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50">
            <span class="relative mt-0.5 inline-grid size-4 shrink-0 place-items-center">
                <input
                    id=id
                    type="checkbox"
                    class="peer size-4 cursor-pointer appearance-none rounded-ctl border border-outline bg-ink-2 transition-colors checked:border-accent checked:bg-accent hover:border-secondary disabled:cursor-not-allowed"
                    prop:checked=move || checked.get()
                    disabled=move || disabled.get()
                    on:change=move |ev| {
                        let v = event_target_checked(&ev);
                        checked.set(v);
                        if let Some(cb) = on_change {
                            cb.run(v);
                        }
                    }
                />
                <svg class="pointer-events-none absolute size-3 text-accent-ink opacity-0 peer-checked:opacity-100" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <path d="M2.5 6.2l2.3 2.3 4.7-4.9" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
            </span>
            <span class=text_class>{label}</span>
        </label>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_ids_are_stable() {
        assert_eq!(segment_id("mode", 1), "mode-seg-1");
        assert_eq!(Segment::new("solo", "Solo").label, "Solo");
    }
}

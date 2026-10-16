//! Form controls: Field (label, help, error), TextField, Select.
//!
//! `Field` owns the ids and wires them to the control inside it through
//! context, so a control never needs its own label plumbing.

use leptos::prelude::*;

use super::button::Force;
use super::text::join;

/// What a control inside a [`Field`] needs to label and describe itself.
#[derive(Clone)]
pub struct FieldContext {
    pub id: String,
    pub described_by: Signal<Option<String>>,
    pub invalid: Signal<bool>,
    pub required: bool,
}

/// Label, optional help text and error message around one control.
///
/// A11y: the label is always visible and bound with `for`/`id`; help and
/// error text are bound with `aria-describedby`; the control gets
/// `aria-invalid` while there is an error, and the error is announced
/// (`role="alert"`). "Required" is written out, not only marked.
#[component]
pub fn Field(
    /// DOM id of the control. Must be unique on the page and stable
    /// between server and browser (derive it from the field's purpose).
    #[prop(into)]
    id: String,
    #[prop(into)] label: String,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(optional, into)] error: Signal<Option<String>>,
    #[prop(optional)] required: bool,
    children: Children,
) -> impl IntoView {
    let hint_id = format!("{id}-hint");
    let error_id = format!("{id}-error");
    let has_hint = hint.is_some();
    let described_by = {
        let hint_id = hint_id.clone();
        let error_id = error_id.clone();
        Signal::derive(move || {
            let mut ids = Vec::new();
            if has_hint {
                ids.push(hint_id.clone());
            }
            if error.try_with(|e| e.is_some()).unwrap_or(false) {
                ids.push(error_id.clone());
            }
            (!ids.is_empty()).then(|| ids.join(" "))
        })
    };
    // `try_*` reads: a field inside a view that is being torn down (a
    // dialog in a panel that unmounts) may run once more after its Field's
    // signals are gone; that must not panic.
    let invalid = Signal::derive(move || error.try_with(|e| e.is_some()).unwrap_or(false));
    provide_context(FieldContext {
        id: id.clone(),
        described_by,
        invalid,
        required,
    });

    view! {
        <div class="flex flex-col gap-1.5">
            <label for=id class="text-label font-medium text-secondary">
                {label}
                {required.then(|| view! { <span class="font-normal text-muted">" (required)"</span> })}
            </label>
            {children()}
            {hint.map(|h| view! { <p id=hint_id class="text-caption text-muted">{h}</p> })}
            {move || error.get().map(|e| view! {
                <p id=error_id.clone() role="alert" class="flex items-start gap-1.5 text-caption text-danger">
                    <svg class="mt-px size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                        <circle cx="8" cy="8" r="6.25" />
                        <path d="M8 4.75v3.75M8 10.75v.5" stroke-linecap="round" />
                    </svg>
                    <span>{e}</span>
                </p>
            })}
        </div>
    }
}

/// Input type of a [`TextField`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputKind {
    #[default]
    Text,
    Email,
    Password,
    Search,
    Number,
    Date,
    /// A clock time (`HH:MM`), paired with a `Date` field.
    Time,
    Url,
}

impl InputKind {
    pub fn as_str(self) -> &'static str {
        match self {
            InputKind::Text => "text",
            InputKind::Email => "email",
            InputKind::Password => "password",
            InputKind::Search => "search",
            InputKind::Number => "number",
            InputKind::Date => "date",
            InputKind::Time => "time",
            InputKind::Url => "url",
        }
    }
}

/// Height of a text field or select.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldSize {
    /// 32px, for dense toolbars and table filters.
    Sm,
    /// 36px, the default.
    #[default]
    Md,
}

impl FieldSize {
    fn class(self) -> &'static str {
        match self {
            FieldSize::Sm => "h-8 text-label",
            FieldSize::Md => "h-9 text-body",
        }
    }
}

/// Makes a [`TextField`] the input of a combobox (WAI-ARIA 1.2): the
/// popup list is `controls`, the highlighted option is `active` (its DOM
/// id), announced through `aria-activedescendant`.
#[derive(Clone)]
pub struct Combobox {
    /// DOM id of the `role="listbox"` the input controls.
    pub controls: String,
    /// DOM id of the active option, if any.
    pub active: Signal<Option<String>>,
}

/// Which key press submits a field's attached action (DESIGN 6.6, "Enter
/// submits"): plain Enter in a single-line field, Ctrl+Enter (or Cmd+Enter)
/// in a textarea, where Enter stays a newline. Never while an input method
/// is composing.
pub fn submits_on(
    key: &str,
    ctrl_or_meta: bool,
    shift_or_alt: bool,
    composing: bool,
    multiline: bool,
) -> bool {
    if key != "Enter" || composing || shift_or_alt {
        return false;
    }
    if multiline {
        ctrl_or_meta
    } else {
        !ctrl_or_meta
    }
}

fn submit_handler(submit: Callback<()>, multiline: bool) -> Callback<leptos::ev::KeyboardEvent> {
    Callback::new(move |ev: leptos::ev::KeyboardEvent| {
        if submits_on(
            &ev.key(),
            ev.ctrl_key() || ev.meta_key(),
            ev.shift_key() || ev.alt_key(),
            ev.is_composing(),
            multiline,
        ) {
            ev.prevent_default();
            submit.run(());
        }
    })
}

/// `on_keydown` for a single-line [`TextField`] with an attached primary
/// button: Enter runs `submit` (the button's action).
pub fn enter_submits(submit: Callback<()>) -> Callback<leptos::ev::KeyboardEvent> {
    submit_handler(submit, false)
}

/// `on_keydown` for a textarea with an attached button: Ctrl+Enter (Cmd+Enter
/// on a Mac) runs `submit`; Enter alone stays a newline.
pub fn ctrl_enter_submits(submit: Callback<()>) -> Callback<leptos::ev::KeyboardEvent> {
    submit_handler(submit, true)
}

const CONTROL: &str = "w-full rounded-ctl border border-outline bg-ink-2 px-3 text-primary placeholder:text-muted transition-colors hover:border-secondary data-[force=hover]:border-secondary disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger";

/// A single-line text input, or a textarea when `rows` is set.
///
/// A11y: put it inside a [`Field`]; it takes the field's id, description
/// and invalid state. Without a field, pass `aria_label`. The edge is the
/// 3:1 outline token, focus is the global chalk outline, and `value` is
/// bound with `prop:value` so it follows the signal. With `combobox` the
/// input gets `role="combobox"`, `aria-controls`, `aria-expanded="true"`,
/// `aria-autocomplete="list"` and `aria-activedescendant`; the caller
/// handles the arrow keys through `on_keydown`.
#[component]
pub fn TextField(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(optional)] kind: InputKind,
    #[prop(optional)] size: FieldSize,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional)] autocomplete: Option<&'static str>,
    #[prop(optional)] minlength: Option<u32>,
    #[prop(optional)] maxlength: Option<u32>,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] disabled: Signal<bool>,
    /// Multi-line: render a textarea with this many rows.
    #[prop(optional)]
    rows: Option<u32>,
    /// Overrides the id from the surrounding [`Field`].
    #[prop(optional, into)]
    id: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] force: Option<Force>,
    /// Key presses (Enter to submit a row, arrows in a combobox, Escape).
    #[prop(optional)]
    on_keydown: Option<Callback<leptos::ev::KeyboardEvent>>,
    /// Focus left the field (commit an inline edit).
    #[prop(optional)]
    on_blur: Option<Callback<()>>,
    /// Combobox wiring (single-line fields only).
    #[prop(optional)]
    combobox: Option<Combobox>,
    /// Extra classes for a single-line input, literal at the call site
    /// (`pl-10` leaves room for a leading icon slot).
    #[prop(optional)]
    class: &'static str,
) -> impl IntoView {
    let extra = class;
    let ctx = use_context::<FieldContext>();
    let id = id.or_else(|| ctx.as_ref().map(|c| c.id.clone()));
    let described_by = ctx.as_ref().map(|c| c.described_by);
    let invalid = ctx.as_ref().map(|c| c.invalid);
    let required = required || ctx.as_ref().is_some_and(|c| c.required);
    let keydown = move |ev: leptos::ev::KeyboardEvent| {
        if let Some(cb) = on_keydown {
            cb.run(ev);
        }
    };
    let blur = move |_: leptos::ev::FocusEvent| {
        if let Some(cb) = on_blur {
            cb.run(());
        }
    };
    let cb_role = combobox.as_ref().map(|_| "combobox");
    let cb_controls = combobox.as_ref().map(|c| c.controls.clone());
    let cb_expanded = combobox.as_ref().map(|_| "true");
    let cb_auto = combobox.as_ref().map(|_| "list");
    let cb_active = combobox.as_ref().map(|c| c.active);
    let aria_active = move || cb_active.and_then(|a| a.get());
    let force = force.map(Force::as_str);
    let on_input = move |ev: leptos::ev::Event| {
        if let Some(v) = value {
            v.set(event_target_value(&ev));
        }
    };
    let aria_invalid = move || {
        invalid
            .and_then(|i| i.try_get())
            .and_then(|b| b.then_some("true"))
    };
    let aria_described = move || described_by.and_then(|d| d.try_get().flatten());

    // Only a bound field follows a signal. An unbound one (a plain form
    // field such as the sign-in email) must not get `prop:value`: the
    // property is set during hydration, which would wipe whatever the
    // browser autofilled or the user typed before the page went live.
    match (rows, value) {
        (Some(rows), Some(v)) => {
            let class = join(&[CONTROL, "py-2 text-body min-h-20 resize-y leading-relaxed"]);
            view! {
                <textarea
                    id=id
                    name=name
                    class=class
                    rows=rows
                    placeholder=placeholder
                    required=required
                    minlength=minlength
                    maxlength=maxlength
                    disabled=move || disabled.get()
                    aria-label=aria_label
                    aria-invalid=aria_invalid
                    aria-describedby=aria_described
                    data-force=force
                    prop:value=move || v.get()
                    on:input=on_input
                    on:keydown=keydown
                    on:blur=blur
                ></textarea>
            }
            .into_any()
        }
        (Some(rows), None) => {
            let class = join(&[CONTROL, "py-2 text-body min-h-20 resize-y leading-relaxed"]);
            view! {
                <textarea
                    id=id
                    name=name
                    class=class
                    rows=rows
                    placeholder=placeholder
                    required=required
                    minlength=minlength
                    maxlength=maxlength
                    disabled=move || disabled.get()
                    aria-label=aria_label
                    aria-invalid=aria_invalid
                    aria-describedby=aria_described
                    data-force=force
                    on:keydown=keydown
                    on:blur=blur
                ></textarea>
            }
            .into_any()
        }
        (None, Some(v)) => {
            let class = join(&[CONTROL, size.class(), extra]);
            view! {
                <input
                    id=id
                    type=kind.as_str()
                    name=name
                    class=class
                    placeholder=placeholder
                    autocomplete=autocomplete
                    required=required
                    minlength=minlength
                    maxlength=maxlength
                    disabled=move || disabled.get()
                    aria-label=aria_label
                    aria-invalid=aria_invalid
                    aria-describedby=aria_described
                    data-force=force
                    role=cb_role
                    aria-controls=cb_controls
                    aria-expanded=cb_expanded
                    aria-autocomplete=cb_auto
                    aria-activedescendant=aria_active
                    prop:value=move || v.get()
                    on:input=on_input
                    on:keydown=keydown
                    on:blur=blur
                />
            }
            .into_any()
        }
        (None, None) => {
            let class = join(&[CONTROL, size.class(), extra]);
            view! {
                <input
                    id=id
                    type=kind.as_str()
                    name=name
                    class=class
                    placeholder=placeholder
                    autocomplete=autocomplete
                    required=required
                    minlength=minlength
                    maxlength=maxlength
                    disabled=move || disabled.get()
                    aria-label=aria_label
                    aria-invalid=aria_invalid
                    aria-describedby=aria_described
                    data-force=force
                    role=cb_role
                    aria-controls=cb_controls
                    aria-expanded=cb_expanded
                    aria-autocomplete=cb_auto
                    aria-activedescendant=aria_active
                    on:keydown=keydown
                    on:blur=blur
                />
            }
            .into_any()
        }
    }
}

/// One option of a [`Select`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
}

/// A native select, styled to match [`TextField`].
///
/// A11y: native `<select>` semantics and keyboard (type-ahead, arrows,
/// Space/Enter). Inside a [`Field`] it takes the field's id, description and
/// invalid state; the chevron is decorative.
#[component]
pub fn Select(
    #[prop(optional)] value: Option<RwSignal<String>>,
    #[prop(into)] options: Vec<SelectOption>,
    #[prop(optional)] size: FieldSize,
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] force: Option<Force>,
) -> impl IntoView {
    let ctx = use_context::<FieldContext>();
    let id = id.or_else(|| ctx.as_ref().map(|c| c.id.clone()));
    let described_by = ctx.as_ref().map(|c| c.described_by);
    let invalid = ctx.as_ref().map(|c| c.invalid);
    let required = ctx.as_ref().is_some_and(|c| c.required);
    let force = force.map(Force::as_str);
    let class = join(&[CONTROL, size.class(), "appearance-none pr-9 cursor-pointer"]);

    let option_views = options
        .into_iter()
        .map(|o| {
            let selected = value.map(|v| v.get_untracked() == o.value).unwrap_or(false);
            view! {
                <option value=o.value selected=selected disabled=o.disabled>{o.label}</option>
            }
        })
        .collect_view();
    let aria_invalid = move || {
        invalid
            .and_then(|i| i.try_get())
            .and_then(|b| b.then_some("true"))
    };
    let aria_described = move || described_by.and_then(|d| d.try_get().flatten());
    // Only a bound select follows a signal; an unbound one keeps the
    // browser's default (the first option) instead of an empty value.
    let select = match value {
        Some(v) => view! {
            <select
                id=id
                name=name
                class=class
                required=required
                disabled=move || disabled.get()
                aria-label=aria_label
                aria-invalid=aria_invalid
                aria-describedby=aria_described
                data-force=force
                prop:value=move || v.get()
                on:change=move |ev| v.set(event_target_value(&ev))
            >
                {option_views}
            </select>
        }
        .into_any(),
        None => view! {
            <select
                id=id
                name=name
                class=class
                required=required
                disabled=move || disabled.get()
                aria-label=aria_label
                aria-invalid=aria_invalid
                aria-describedby=aria_described
                data-force=force
            >
                {option_views}
            </select>
        }
        .into_any(),
    };
    view! {
        <div class="relative">
            {select}
            <svg
                class="pointer-events-none absolute right-3 top-1/2 size-3.5 -translate-y-1/2 text-muted"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="1.75"
                aria-hidden="true"
            >
                <path d="M4 6l4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </div>
    }
}

#[cfg(test)]
mod enter_tests {
    use super::*;

    #[test]
    fn enter_submits_single_line_and_ctrl_enter_submits_textareas() {
        assert!(submits_on("Enter", false, false, false, false));
        assert!(!submits_on("Enter", false, false, true, false), "composing");
        assert!(!submits_on("Enter", false, true, false, false), "shift");
        assert!(!submits_on("a", false, false, false, false));
        assert!(!submits_on("Enter", false, false, false, true), "newline");
        assert!(submits_on("Enter", true, false, false, true));
    }
}

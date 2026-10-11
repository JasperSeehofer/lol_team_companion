//! Toast: a short note about something that just happened ("Note saved"),
//! optionally with one action ("Undo"). Inline, lasting messages are
//! `Callout`s instead.

use leptos::callback::Callback;
use leptos::prelude::*;

use super::callout::{glyph_for, CalloutTone};

/// The kind of a toast notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Info,
    Warning,
    /// Stays until dismissed.
    Error,
}

impl ToastKind {
    fn tone(self) -> CalloutTone {
        match self {
            ToastKind::Success => CalloutTone::Success,
            ToastKind::Info => CalloutTone::Info,
            ToastKind::Warning => CalloutTone::Warning,
            ToastKind::Error => CalloutTone::Danger,
        }
    }

    fn edge(self) -> &'static str {
        match self {
            ToastKind::Success => "border-success/50",
            ToastKind::Info => "border-outline/70",
            ToastKind::Warning => "border-warning/60",
            ToastKind::Error => "border-danger/60",
        }
    }
}

/// How long a toast stays, in milliseconds; `None` means until dismissed.
/// A toast with an action stays longer, so "Undo" can be reached.
pub fn toast_duration(kind: ToastKind, has_action: bool) -> Option<u32> {
    match (kind, has_action) {
        (ToastKind::Error, _) => None,
        (ToastKind::Warning, _) => Some(8000),
        (_, true) => Some(8000),
        (_, false) => Some(5000),
    }
}

/// The one action a toast may carry ("Undo", "View").
#[derive(Clone)]
pub struct ToastAction {
    pub label: String,
    pub on_action: Callback<()>,
}

/// A toast to show.
#[derive(Clone)]
pub struct Toast {
    pub kind: ToastKind,
    pub message: String,
    pub action: Option<ToastAction>,
}

impl Toast {
    pub fn new(kind: ToastKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            action: None,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Success, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Error, message)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Info, message)
    }

    /// Add the action button ("Undo"); it runs `on_action` and dismisses.
    pub fn with_action(mut self, label: impl Into<String>, on_action: Callback<()>) -> Self {
        self.action = Some(ToastAction {
            label: label.into(),
            on_action,
        });
        self
    }
}

/// A single toast notification entry.
#[derive(Clone)]
pub struct ToastEntry {
    pub id: u64,
    pub kind: ToastKind,
    pub message: String,
    pub action: Option<ToastAction>,
}

/// Context injected by [`ToastProvider`]. `show` is the short form
/// (`toast.show.run((ToastKind::Success, "Saved".into()))`), `push` takes a
/// full [`Toast`] with an action.
#[derive(Clone, Copy)]
pub struct ToastContext {
    pub show: Callback<(ToastKind, String)>,
    pub push: Callback<Toast>,
    pub dismiss: Callback<u64>,
}

/// At most this many toasts are visible; the oldest non-error goes first.
const MAX_TOASTS: usize = 3;

/// Push `entry`, dropping the oldest toast (preferring non-errors) when
/// the stack is full.
pub fn push_capped(stack: &mut Vec<ToastEntry>, entry: ToastEntry) {
    if stack.len() >= MAX_TOASTS {
        let pos = stack
            .iter()
            .position(|t| t.kind != ToastKind::Error)
            .unwrap_or(0);
        stack.remove(pos);
    }
    stack.push(entry);
}

/// Provides toasts to the whole app. Place it once, unconditionally, in
/// `App` (not in a cfg block) so server rendering can call `use_context`.
///
/// A11y: one polite live region (`aria-live="polite"`) that exists from the
/// first paint, so additions are announced without stealing focus; the
/// region is a labelled landmark ("Notifications"). Success and info leave
/// after 5 s, with an action or a warning after 8 s, errors stay until
/// dismissed. Hovering or focusing a toast pauses its timer (it restarts
/// when the pointer or focus leaves). Action and dismiss are real buttons
/// of at least 24px; the kind is said in words.
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toasts = RwSignal::new(Vec::<ToastEntry>::new());
    let next_id = StoredValue::new(0u64);
    #[allow(unused_variables)]
    let timers = StoredValue::new(Vec::<(u64, i32)>::new());

    let dismiss = Callback::new(move |id: u64| {
        toasts.update(|v| v.retain(|t| t.id != id));
        #[cfg(feature = "hydrate")]
        clear_timer(timers, id);
    });

    // Starts (or restarts) the auto-dismiss timer of a toast.
    #[allow(unused_variables)]
    let arm = Callback::new(move |id: u64| {
        let Some((kind, has_action)) = toasts.with_untracked(|v| {
            v.iter()
                .find(|t| t.id == id)
                .map(|t| (t.kind, t.action.is_some()))
        }) else {
            return;
        };
        #[cfg(feature = "hydrate")]
        if let Some(ms) = toast_duration(kind, has_action) {
            use wasm_bindgen::prelude::*;
            clear_timer(timers, id);
            let cb = Closure::once(move || dismiss.run(id));
            if let Some(win) = web_sys::window() {
                if let Ok(handle) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    ms as i32,
                ) {
                    timers.update_value(|t| t.push((id, handle)));
                }
            }
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = (kind, has_action);
    });

    #[allow(unused_variables)]
    let pause = Callback::new(move |id: u64| {
        #[cfg(feature = "hydrate")]
        clear_timer(timers, id);
    });

    let push = Callback::new(move |toast: Toast| {
        let id = next_id.get_value();
        next_id.set_value(id + 1);
        toasts.update(|v| {
            push_capped(
                v,
                ToastEntry {
                    id,
                    kind: toast.kind,
                    message: toast.message,
                    action: toast.action,
                },
            )
        });
        arm.run(id);
    });
    let show = Callback::new(move |(kind, message): (ToastKind, String)| {
        push.run(Toast::new(kind, message));
    });

    provide_context(ToastContext {
        show,
        push,
        dismiss,
    });

    view! {
        {children()}
        <section
            aria-label="Notifications"
            class="pointer-events-none fixed inset-x-4 bottom-4 z-(--z-toast) sm:right-auto sm:w-96"
        >
            <ol aria-live="polite" aria-relevant="additions text" class="flex flex-col gap-2">
                <For each=move || toasts.get() key=|t| t.id let:t>
                    <ToastItem entry=t dismiss=dismiss pause=pause resume=arm />
                </For>
            </ol>
        </section>
    }
}

#[cfg(feature = "hydrate")]
fn clear_timer(timers: StoredValue<Vec<(u64, i32)>>, id: u64) {
    timers.update_value(|t| {
        t.retain(|(tid, handle)| {
            if *tid == id {
                if let Some(win) = web_sys::window() {
                    win.clear_timeout_with_handle(*handle);
                }
                false
            } else {
                true
            }
        })
    });
}

/// One toast in the stack.
///
/// A11y: see [`ToastProvider`].
#[component]
fn ToastItem(
    entry: ToastEntry,
    dismiss: Callback<u64>,
    pause: Callback<u64>,
    resume: Callback<u64>,
) -> impl IntoView {
    let id = entry.id;
    let tone = entry.kind.tone();
    let class = format!(
        "pointer-events-auto flex items-start gap-3 rounded-ctl border bg-ink-2 px-3.5 py-3 shadow-overlay {}",
        entry.kind.edge()
    );
    let action = entry.action.map(|a| {
        let on_action = a.on_action;
        view! {
            <button
                type="button"
                class="-my-1 h-7 shrink-0 cursor-pointer rounded-ctl border border-outline px-2.5 text-label font-semibold text-primary transition-colors hover:bg-ink-3"
                on:click=move |_| {
                    on_action.run(());
                    dismiss.run(id);
                }
            >
                {a.label}
            </button>
        }
    });
    view! {
        <li
            class=class
            data-toast=tone.word()
            on:mouseenter=move |_| pause.run(id)
            on:mouseleave=move |_| resume.run(id)
            on:focusin=move |_| pause.run(id)
            on:focusout=move |_| resume.run(id)
        >
            {glyph_for(tone)}
            <p class="min-w-0 flex-1 text-body text-primary">
                <span class="sr-only">{format!("{}: ", tone.word())}</span>
                {entry.message}
            </p>
            {action}
            <button
                type="button"
                class="-my-1 -mr-1 grid size-7 shrink-0 cursor-pointer place-items-center rounded-ctl text-muted transition-colors hover:bg-ink-3 hover:text-primary"
                aria-label="Dismiss"
                on:click=move |_| dismiss.run(id)
            >
                <svg class="size-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                    <path d="M4 4l8 8M12 4l-8 8" stroke-linecap="round" />
                </svg>
            </button>
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: u64, kind: ToastKind) -> ToastEntry {
        ToastEntry {
            id,
            kind,
            message: String::new(),
            action: None,
        }
    }

    #[test]
    fn errors_stay_and_actions_get_more_time() {
        assert_eq!(toast_duration(ToastKind::Error, false), None);
        assert_eq!(toast_duration(ToastKind::Error, true), None);
        assert_eq!(toast_duration(ToastKind::Success, false), Some(5000));
        assert_eq!(toast_duration(ToastKind::Success, true), Some(8000));
        assert_eq!(toast_duration(ToastKind::Warning, false), Some(8000));
    }

    #[test]
    fn the_stack_keeps_three_and_drops_the_oldest_non_error() {
        let mut v = Vec::new();
        push_capped(&mut v, entry(0, ToastKind::Error));
        push_capped(&mut v, entry(1, ToastKind::Success));
        push_capped(&mut v, entry(2, ToastKind::Info));
        push_capped(&mut v, entry(3, ToastKind::Success));
        let ids: Vec<u64> = v.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![0, 2, 3]);
    }
}

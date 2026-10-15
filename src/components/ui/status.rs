//! Save state and load failure: the two small status patterns every editor
//! and list shares (DESIGN 6.6, "Save state in the header").
//!
//! `SaveStatus` is the one line in a `PageHeader` meta that says whether the
//! last change reached the server; `SaveTracker` drives it from a page's save
//! calls; `LoadFailed` replaces the hand-written "X did not load" callouts
//! with one that offers "Try again".

use leptos::prelude::*;

use super::{Button, ButtonSize, ButtonVariant, Callout, CalloutTone};

/// Where the page's last change stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SaveState {
    /// Nothing to say (no change yet, or "Saved" has faded). The live region
    /// stays mounted and empty.
    #[default]
    Idle,
    /// A new document that has never been saved.
    NotSaved,
    /// Edits that are not saved yet (an autosave is pending).
    Unsaved,
    /// A save is on its way.
    Saving,
    /// The last save reached the server.
    Saved,
    /// The last save failed; the text says what did not save.
    Failed(String),
}

impl SaveState {
    /// The words shown and announced (DESIGN 6.6).
    pub fn label(&self) -> String {
        match self {
            SaveState::Idle => String::new(),
            SaveState::NotSaved => "Not saved yet".into(),
            SaveState::Unsaved => "Unsaved changes".into(),
            SaveState::Saving => "Saving\u{2026}".into(),
            SaveState::Saved => "Saved".into(),
            SaveState::Failed(what) if what.is_empty() => "Could not save".into(),
            SaveState::Failed(what) => format!("Could not save: {what}"),
        }
    }

    fn class(&self) -> &'static str {
        match self {
            SaveState::Saved => "inline-flex items-center gap-2 text-label text-secondary",
            SaveState::Failed(_) => "inline-flex items-center gap-2 text-label text-danger",
            _ => "inline-flex items-center gap-2 text-label text-muted",
        }
    }
}

/// How long "Saved" stays before the line goes quiet.
pub const SAVED_FADE_MS: i32 = 4000;

/// Drives a [`SaveState`] from a page's save calls. `Copy`, so it goes into
/// every closure. "Saved" goes quiet after [`SAVED_FADE_MS`] (in the browser
/// only); a pending fade is cancelled by the next change and on cleanup.
#[derive(Clone, Copy)]
pub struct SaveTracker {
    pub state: RwSignal<SaveState>,
    #[allow(dead_code)]
    timer: RwSignal<Option<i32>>,
}

impl SaveTracker {
    pub fn saving(&self) {
        self.cancel_fade();
        self.state.set(SaveState::Saving);
    }

    pub fn saved(&self) {
        self.cancel_fade();
        self.state.set(SaveState::Saved);
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let state = self.state;
            let timer = self.timer;
            let cb = Closure::once(move || {
                let _ = timer.try_set(None);
                if state.try_get_untracked() == Some(SaveState::Saved) {
                    let _ = state.try_set(SaveState::Idle);
                }
            });
            if let Some(win) = web_sys::window() {
                if let Ok(id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    SAVED_FADE_MS,
                ) {
                    self.timer.set(Some(id));
                }
            }
            cb.forget();
        }
    }

    pub fn failed(&self, what: impl Into<String>) {
        self.cancel_fade();
        self.state.set(SaveState::Failed(what.into()));
    }

    fn cancel_fade(&self) {
        #[cfg(feature = "hydrate")]
        if let Some(Some(id)) = self.timer.try_get_untracked() {
            if let Some(win) = web_sys::window() {
                win.clear_timeout_with_handle(id);
            }
            let _ = self.timer.try_set(None);
        }
    }
}

/// A [`SaveTracker`] owned by the calling component (its fade timer is
/// cleared when the component goes away).
pub fn use_save_state() -> SaveTracker {
    let tracker = SaveTracker {
        state: RwSignal::new(SaveState::Idle),
        timer: RwSignal::new(None),
    };
    on_cleanup(move || tracker.cancel_fade());
    tracker
}

/// The save state line for a `PageHeader` meta.
///
/// A11y: a `role="status"` region that is always mounted (so changes are
/// announced) and says the state in words, never by colour alone. Use one
/// per page. When the state is `Failed` and `on_retry` is set, a "Retry"
/// button follows the text.
#[component]
pub fn SaveStatus(
    #[prop(into)] state: Signal<SaveState>,
    #[prop(optional)] on_retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <span role="status" class=move || state.with(|s| s.class()) data-save-state=move || state.with(|s| match s {
            SaveState::Idle => "idle",
            SaveState::NotSaved => "not-saved",
            SaveState::Unsaved => "unsaved",
            SaveState::Saving => "saving",
            SaveState::Saved => "saved",
            SaveState::Failed(_) => "failed",
        })>
            <span>{move || state.with(|s| s.label())}</span>
            {move || {
                let failed = state.with(|s| matches!(s, SaveState::Failed(_)));
                (failed && on_retry.is_some()).then(|| view! {
                    <Button
                        size=ButtonSize::Sm
                        variant=ButtonVariant::Quiet
                        on_click=Callback::new(move |_| {
                            if let Some(cb) = on_retry {
                                cb.run(());
                            }
                        })
                    >
                        "Retry"
                    </Button>
                })
            }}
        </span>
    }
}

/// A section that did not load, with "Try again".
///
/// A11y: a danger [`Callout`] (`role="alert"`, announced when it appears)
/// titled "{what} did not load"; the "Try again" button is a native button
/// that refetches.
#[component]
pub fn LoadFailed(
    /// What did not load, as a sentence start: "Your pool", "The drafts".
    #[prop(into)]
    what: String,
    on_retry: Callback<()>,
) -> impl IntoView {
    view! {
        <Callout
            tone=CalloutTone::Danger
            title=format!("{what} did not load")
            action=move || view! {
                <Button size=ButtonSize::Sm on_click=Callback::new(move |_| on_retry.run(()))>"Try again"</Button>
            }
        >
            "The server did not answer. Nothing you saved is lost."
        </Callout>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_design_6_6() {
        assert_eq!(SaveState::Idle.label(), "");
        assert_eq!(SaveState::NotSaved.label(), "Not saved yet");
        assert_eq!(SaveState::Unsaved.label(), "Unsaved changes");
        assert_eq!(SaveState::Saving.label(), "Saving\u{2026}");
        assert_eq!(SaveState::Saved.label(), "Saved");
        assert_eq!(SaveState::Failed(String::new()).label(), "Could not save");
        assert_eq!(
            SaveState::Failed("Ahri's tier".into()).label(),
            "Could not save: Ahri's tier"
        );
    }

    #[test]
    fn failure_is_drawn_as_danger() {
        assert!(SaveState::Failed("x".into())
            .class()
            .contains("text-danger"));
        assert!(SaveState::Saved.class().contains("text-secondary"));
    }
}

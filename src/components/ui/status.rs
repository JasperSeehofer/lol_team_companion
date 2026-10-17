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

    /// Whether leaving the page now would lose work: an edit not saved yet,
    /// a save on its way, or a save that failed. (`NotSaved` guards only
    /// when the new document has content: the page decides that.)
    pub fn guards_leave(&self) -> bool {
        matches!(
            self,
            SaveState::Unsaved | SaveState::Saving | SaveState::Failed(_)
        )
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
    /// A failure since the last successful save: retries and further edits
    /// stay in the same failure, so its toast is shown once.
    failing: RwSignal<bool>,
}

impl SaveTracker {
    pub fn saving(&self) {
        self.cancel_fade();
        self.state.set(SaveState::Saving);
    }

    /// The change reached the server; "Saved" stays (an editor's document
    /// state, DESIGN 6.6), unlike [`SaveTracker::saved`], which fades.
    pub fn saved_steady(&self) {
        self.cancel_fade();
        self.failing.set(false);
        self.state.set(SaveState::Saved);
    }

    /// Show a state that is neither a save nor a failure ("Not saved yet",
    /// "Unsaved changes", nothing). Ends a failure only when `state` says
    /// the document has no unsaved work left to lose (`Idle`, `NotSaved`
    /// for a document just opened or started).
    pub fn show(&self, state: SaveState) {
        self.cancel_fade();
        if matches!(state, SaveState::Idle | SaveState::NotSaved) {
            self.failing.set(false);
        }
        self.state.set(state);
    }

    pub fn saved(&self) {
        self.cancel_fade();
        self.failing.set(false);
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

    /// The last save failed. Returns `true` only for the first failure since
    /// the last successful save (the page shows its one error toast then,
    /// never per retry or per autosave attempt).
    pub fn failed(&self, what: impl Into<String>) -> bool {
        self.cancel_fade();
        let entered = !self.failing.get_untracked();
        self.failing.set(true);
        self.state.set(SaveState::Failed(what.into()));
        entered
    }

    /// Whether the last save failed and nothing has succeeded since.
    pub fn is_failing(&self) -> bool {
        self.failing.get_untracked()
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

/// The toast shown once when an editor's save starts failing (DESIGN 6.6).
pub const SAVE_FAILED_TOAST: &str = "Your changes are not saved. Keep this tab open and use Retry.";

/// What did not work, for "Could not save: {reason}": a message the server
/// sent is shown as written (without the Leptos wrapper); anything else (no
/// answer, a gateway error, a body that is not ours) is "the server did not
/// answer". Never the raw error text.
pub fn save_failure_reason(e: &ServerFnError) -> String {
    match e {
        ServerFnError::ServerError(msg) if !msg.trim().is_empty() => {
            crate::models::utils::error_text(msg)
        }
        _ => "the server did not answer".into(),
    }
}

/// Asks the browser to confirm leaving the page (a reload, closing the tab,
/// a link out of the app) while `dirty` is true: a `beforeunload` listener
/// is added when it turns true and removed when it turns false or the
/// calling component goes away. Browser only; nothing on the server.
///
/// A11y: the browser's own confirmation, which every assistive technology
/// already knows; the page adds no dialog of its own.
pub fn use_unsaved_guard(dirty: Signal<bool>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        type Listener = Closure<dyn FnMut(web_sys::BeforeUnloadEvent)>;
        let listener = StoredValue::new_local(None::<Listener>);
        let remove = move || {
            if let Some(Some(cb)) = listener.try_update_value(|l| l.take()) {
                if let Some(win) = web_sys::window() {
                    let _ = win.remove_event_listener_with_callback(
                        "beforeunload",
                        cb.as_ref().unchecked_ref(),
                    );
                }
            }
        };
        Effect::new(move |_| {
            if dirty.get() {
                if listener.with_value(|l| l.is_some()) {
                    return;
                }
                let cb: Listener = Closure::new(move |ev: web_sys::BeforeUnloadEvent| {
                    ev.prevent_default();
                    // Older Chromium and Safari still need a return value.
                    ev.set_return_value("unsaved");
                });
                if let Some(win) = web_sys::window() {
                    if win
                        .add_event_listener_with_callback(
                            "beforeunload",
                            cb.as_ref().unchecked_ref(),
                        )
                        .is_ok()
                    {
                        listener.set_value(Some(cb));
                    }
                }
            } else {
                remove();
            }
        });
        on_cleanup(remove);
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = dirty;
}

/// A [`SaveTracker`] owned by the calling component (its fade timer is
/// cleared when the component goes away).
pub fn use_save_state() -> SaveTracker {
    let tracker = SaveTracker {
        state: RwSignal::new(SaveState::Idle),
        timer: RwSignal::new(None),
        failing: RwSignal::new(false),
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
                        size=ButtonSize::Touch
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
                <Button size=ButtonSize::Touch on_click=Callback::new(move |_| on_retry.run(()))>"Try again"</Button>
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
    fn leaving_is_guarded_only_with_work_at_risk() {
        assert!(!SaveState::Idle.guards_leave());
        assert!(!SaveState::NotSaved.guards_leave());
        assert!(!SaveState::Saved.guards_leave());
        assert!(SaveState::Unsaved.guards_leave());
        assert!(SaveState::Saving.guards_leave());
        assert!(SaveState::Failed("x".into()).guards_leave());
    }

    #[test]
    fn failure_reason_shows_server_words_and_hides_the_rest() {
        assert_eq!(
            save_failure_reason(&ServerFnError::ServerError(
                "Create or join your team first".into()
            )),
            "Create or join your team first"
        );
        assert_eq!(
            save_failure_reason(&ServerFnError::Deserialization(
                "Invalid format: missing delimiter in \"\"".into()
            )),
            "the server did not answer"
        );
        assert_eq!(
            save_failure_reason(&ServerFnError::Request("NetworkError".into())),
            "the server did not answer"
        );
        assert_eq!(
            save_failure_reason(&ServerFnError::ServerError("  ".into())),
            "the server did not answer"
        );
    }

    #[test]
    fn failed_reports_only_the_first_failure_until_a_save_works() {
        let owner = Owner::new();
        owner.with(|| {
            let t = SaveTracker {
                state: RwSignal::new(SaveState::Unsaved),
                timer: RwSignal::new(None),
                failing: RwSignal::new(false),
            };
            assert!(t.failed("a"));
            // A retry and an edit stay in the same failure.
            t.saving();
            assert!(!t.failed("b"));
            t.show(SaveState::Unsaved);
            assert!(!t.failed("c"));
            assert!(t.is_failing());
            assert_eq!(t.state.get_untracked(), SaveState::Failed("c".into()));
            // A save that works ends it.
            t.saved_steady();
            assert!(!t.is_failing());
            assert_eq!(t.state.get_untracked(), SaveState::Saved);
            assert!(t.failed("d"));
            // Opening another document ends it too.
            t.show(SaveState::NotSaved);
            assert!(t.failed("e"));
        });
    }

    #[test]
    fn failure_is_drawn_as_danger() {
        assert!(SaveState::Failed("x".into())
            .class()
            .contains("text-danger"));
        assert!(SaveState::Saved.class().contains("text-secondary"));
    }
}

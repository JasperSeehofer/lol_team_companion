//! PrepSequence: the Match Day "what to do next" checklist. Typographic,
//! numbered, with the next step emphasised; no progress art.

use leptos::prelude::*;

/// Where a preparation step stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrepState {
    Done,
    /// The one step to do now. At most one per sequence.
    Next,
    Todo,
    /// Needs attention (for example open points on the game plan).
    Attention,
}

impl PrepState {
    pub fn label(self) -> &'static str {
        match self {
            PrepState::Done => "Done",
            PrepState::Next => "Next",
            PrepState::Todo => "To do",
            PrepState::Attention => "Needs attention",
        }
    }
}

/// One step of a [`PrepSequence`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepStep {
    pub title: String,
    /// One line under the title ("Plan 3 not started").
    pub detail: Option<String>,
    /// Short status on the right ("2 of 3", "Ready").
    pub status: Option<String>,
    pub state: PrepState,
    /// Where the step is done.
    pub href: Option<String>,
    /// Label of the action link on the next step ("Open the brief").
    pub action: Option<String>,
}

impl PrepStep {
    pub fn new(title: impl Into<String>, state: PrepState) -> Self {
        Self {
            title: title.into(),
            detail: None,
            status: None,
            state,
            href: None,
            action: None,
        }
    }
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
    pub fn status(mut self, s: impl Into<String>) -> Self {
        self.status = Some(s.into());
        self
    }
    pub fn link(mut self, href: impl Into<String>, action: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self.action = Some(action.into());
        self
    }
}

/// How loudly the next step asks to be done.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrepEmphasis {
    /// The next step's link is the view's primary action (chalk fill):
    /// Match Day.
    #[default]
    Primary,
    /// The view has its own primary action (the game plan's "Save plan"):
    /// the next step is outlined, not filled.
    Quiet,
}

/// A numbered list of preparation steps with the next one emphasised.
///
/// A11y: an ordered list; each item states its status in words ("Done",
/// "Next", "Needs attention"), with the shape of the number box as a second
/// cue. The next step carries `aria-current="step"` and a real link to
/// where it is done. With `emphasis` Quiet the next step is outlined, so
/// the page's own primary action stays the only chalk fill.
#[component]
pub fn PrepSequence(
    #[prop(into)] label: String,
    steps: Vec<PrepStep>,
    #[prop(optional)] emphasis: PrepEmphasis,
) -> impl IntoView {
    view! {
        <ol class="flex flex-col" aria-label=label>
            {steps
                .into_iter()
                .enumerate()
                .map(|(i, s)| {
                    let n = format!("{}", i + 1);
                    let is_next = s.state == PrepState::Next;
                    let row = if is_next {
                        "grid grid-cols-[1.75rem_minmax(0,1fr)_auto] items-start gap-x-3 gap-y-2 border-t border-rule py-4"
                    } else {
                        "grid grid-cols-[1.75rem_minmax(0,1fr)_auto] items-baseline gap-x-3 border-t border-rule py-3"
                    };
                    let num = match s.state {
                        PrepState::Done => "figures inline-grid size-6 place-items-center rounded-ctl bg-ink-3 text-caption text-secondary",
                        PrepState::Next if emphasis == PrepEmphasis::Quiet => "figures inline-grid size-6 place-items-center rounded-ctl border border-accent text-caption font-semibold text-accent",
                        PrepState::Next => "figures inline-grid size-6 place-items-center rounded-ctl bg-accent text-caption font-semibold text-accent-ink",
                        PrepState::Todo => "figures inline-grid size-6 place-items-center rounded-ctl border border-outline text-caption text-muted",
                        PrepState::Attention => "figures inline-grid size-6 place-items-center rounded-ctl border border-warning text-caption text-warning",
                    };
                    let title_class = match s.state {
                        PrepState::Done => "font-medium text-secondary",
                        PrepState::Next => "font-display text-heading font-semibold text-primary",
                        _ => "font-medium text-primary",
                    };
                    let status_class = match s.state {
                        PrepState::Done => "figures text-label text-success",
                        PrepState::Next => "figures text-label font-semibold text-accent",
                        PrepState::Todo => "figures text-label text-muted",
                        PrepState::Attention => "figures text-label text-warning",
                    };
                    let status = s.status.clone().unwrap_or_else(|| s.state.label().to_string());
                    let status_view = match s.state {
                        PrepState::Done => view! {
                            <span class=status_class>
                                <svg class="mr-1 inline size-3.5 align-[-2px]" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                    <path d="M3.5 8.5l3 3 6-7" stroke-linecap="round" stroke-linejoin="round" />
                                </svg>
                                {status}
                            </span>
                        }
                        .into_any(),
                        PrepState::Attention => view! {
                            <span class=status_class>
                                <span class="mr-1" aria-hidden="true">"!"</span>
                                {status}
                            </span>
                        }
                        .into_any(),
                        _ => view! { <span class=status_class>{status}</span> }.into_any(),
                    };
                    let state_word = s.state.label();
                    let title_view = match (&s.href, is_next) {
                        (Some(href), false) => view! {
                            <a href=href.clone() class="hover:underline underline-offset-4">{s.title.clone()}</a>
                        }
                        .into_any(),
                        _ => view! { <span>{s.title.clone()}</span> }.into_any(),
                    };
                    let action = match (is_next, s.href.clone(), s.action.clone()) {
                        (true, Some(href), Some(action)) => Some(view! {
                            <a
                                href=href
                                class=match emphasis {
                                    PrepEmphasis::Primary => "col-start-2 inline-flex h-8 w-fit items-center gap-2 rounded-ctl border border-accent bg-accent px-3 text-label font-semibold text-accent-ink transition-colors hover:bg-accent-hover",
                                    PrepEmphasis::Quiet => "col-start-2 inline-flex h-8 w-fit items-center gap-2 rounded-ctl border border-outline px-3 text-label font-medium text-primary transition-colors hover:bg-ink-3",
                                }
                            >
                                {action}
                                <span aria-hidden="true">"\u{2192}"</span>
                            </a>
                        }),
                        _ => None,
                    };
                    view! {
                        <li class=row aria-current=is_next.then_some("step")>
                            <span class=num aria-hidden="true">{n}</span>
                            <div class="min-w-0">
                                <p class=title_class>
                                    <span class="sr-only">{format!("{state_word}: ")}</span>
                                    {title_view}
                                </p>
                                {s.detail.clone().map(|d| view! { <p class="mt-0.5 text-label text-muted">{d}</p> })}
                            </div>
                            {status_view}
                            {action}
                        </li>
                    }
                })
                .collect_view()}
        </ol>
    }
}

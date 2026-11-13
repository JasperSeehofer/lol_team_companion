//! M165 "Review together": the session page's local pieces (design brief
//! R1, recipe cards 2 to 7). Single-surface components, not design-system
//! primitives: they live here, not in `ui/`, and build on `ui` primitives.
//!
//! Anonymity rule (TC-J11): nothing here takes an author for a card of the
//! anonymous prompt. [`ReviewNote`] renders an author line only when it is
//! given one, and the page never gives one for "What didn't go to plan";
//! the row markup is otherwise identical for every card and every viewer.
//! Card words are always text nodes, never HTML.

use leptos::prelude::*;

use crate::components::ui::{AvatarSize, Button, ButtonSize, ButtonVariant, UserAvatar};
use crate::models::review_together::{Prompt, ANONYMOUS_CAPTION};

/// One note's words in the team-note serif, with its author on its own line
/// in the sans caption (attributed prompts only).
///
/// A11y: two paragraphs in reading order (the words, then the plain
/// username); no quotation marks or dash to read out. Without `author`
/// there is no author element at all, not an empty one.
#[component]
pub fn ReviewNote(
    #[prop(into)] text: String,
    /// The plain username; `None` for the anonymous prompt.
    author: Option<String>,
) -> impl IntoView {
    view! {
        <p class="note text-reading text-primary max-w-prose [overflow-wrap:anywhere]">{text}</p>
        {author.map(|a| view! { <p class="text-caption text-muted">{a}</p> })}
    }
}

/// One prompt's revealed notes: the prompt as an `h2` (an `h3` when
/// `nested`, inside the review page's "Reviewed together" section), the
/// note count, the anonymity caption under "What didn't go to plan",
/// optional facilitator tools, then the notes as a ruled list (`children`
/// are its `<li>` rows).
///
/// A11y: a `section` labelled by its heading, one level under the section
/// that holds it; the notes are a list, one item per note or merged group;
/// the caption is plain text under the heading, read before the notes.
#[component]
pub fn NoteGroup(
    prompt: Prompt,
    #[prop(into)] count: Signal<usize>,
    /// Inside another section (the review page): an `h3` in `text-heading`.
    #[prop(optional)]
    nested: bool,
    /// Facilitator tools above the list (near-duplicates).
    #[prop(optional, into)]
    tools: Option<ViewFn>,
    children: Children,
) -> impl IntoView {
    let heading_id = format!("notes-{}", prompt.key());
    view! {
        <section class="flex flex-col gap-3" aria-labelledby=heading_id.clone() data-note-group=prompt.key()>
            <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
                {if nested {
                    view! { <h3 id=heading_id.clone() class="font-display text-heading font-semibold text-primary">{prompt.label()}</h3> }.into_any()
                } else {
                    view! { <h2 id=heading_id.clone() class="font-display text-title font-semibold tracking-[-0.01em] text-primary">{prompt.label()}</h2> }.into_any()
                }}
                <span class="figures text-caption text-muted">
                    {move || match count.get() {
                        1 => "1 note".to_string(),
                        n => format!("{n} notes"),
                    }}
                </span>
            </div>
            {prompt.is_anonymous().then(|| view! {
                <p class="-mt-1 text-label text-muted" data-anonymous-caption="">{ANONYMOUS_CAPTION}</p>
            })}
            {tools.map(|t| t.run())}
            <ul class="divide-y divide-rule border-y border-rule">{children()}</ul>
        </section>
    }
}

/// "+1": agree with a note. Uniform on every note, your own included, with
/// the count beside it and no list of who agreed.
///
/// A11y: a toggle button (`aria-pressed` = your own +1) whose name says
/// what it does and how many agree ("Agree, 3 so far"); the visible "+1"
/// and the figure are hidden from screen readers, the name carries both.
#[component]
pub fn PlusOne(
    #[prop(into)] count: Signal<u32>,
    #[prop(into)] pressed: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    view! {
        <span class="inline-flex items-center gap-2">
            <Button
                size=ButtonSize::Touch
                pressed=pressed
                disabled=disabled
                on_click=Callback::new(move |_| on_toggle.run(()))
                attr:data-plus-one=""
            >
                <span aria-hidden="true">"+1"</span>
                <span class="sr-only">{move || format!("Agree, {} so far", count.get())}</span>
            </Button>
            <span class="figures text-label text-secondary" aria-hidden="true">{move || count.get()}</span>
        </span>
    }
}

/// "Here now": who has the session open, alphabetical, one entry per
/// person. No dot, ring, check or typing cue; nothing about writing.
///
/// A11y: a list labelled "Here now" (not a live region: arrivals are not
/// announced); a failed poll is said in words, "Reconnecting…".
#[component]
pub fn PresenceStrip(
    #[prop(into)] names: Signal<Vec<String>>,
    #[prop(into)] reconnecting: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2" data-presence="">
            <p class="eyebrow">"Here now"</p>
            <ul role="list" aria-label="Here now" class="flex flex-wrap gap-x-4 gap-y-2">
                {move || {
                    let mut list = names.get();
                    list.sort_by_key(|n| n.to_lowercase());
                    list.dedup();
                    list.into_iter()
                        .map(|n| view! {
                            <li class="inline-flex items-center gap-2">
                                <UserAvatar name=n.clone() size=AvatarSize::Sm />
                                <span class="text-label text-secondary">{n}</span>
                            </li>
                        })
                        .collect_view()
                }}
            </ul>
            {move || reconnecting.get().then(|| view! {
                <p class="text-caption text-muted">"Reconnecting\u{2026}"</p>
            })}
        </div>
    }
}

/// The facilitator's one count: members who have written or passed, over
/// all four prompts. Never per prompt, never who.
///
/// A11y: plain text, "4 of 5" then "have written or passed"; not a live
/// region (it changes quietly as people write).
#[component]
pub fn WrittenCount(
    #[prop(into)] written: Signal<u32>,
    #[prop(into)] members: Signal<u32>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-0.5" data-written-count="">
            <p class="figures text-title text-primary">{move || format!("{} of {}", written.get(), members.get())}</p>
            <p class="text-label text-secondary">"have written or passed"</p>
        </div>
    }
}

/// "2 of 3 votes left" in the Decide heading line; "No votes left" at the
/// cap.
///
/// A11y: a polite status region, mounted from the start, so a change of
/// the count is announced without moving focus.
#[component]
pub fn VotesLeft(#[prop(into)] left: Signal<u32>, total: u32) -> impl IntoView {
    view! {
        <span role="status" class="figures text-label text-secondary" data-votes-left="">
            {move || match left.get() {
                0 => "No votes left".to_string(),
                n => format!("{n} of {total} votes left"),
            }}
        </span>
    }
}

/// The owner of an accepted proposal: one chip per current member's
/// username; pressing the chosen one again clears it.
///
/// A11y: a group named "Owner of: <text>" of toggle buttons
/// (`aria-pressed` on the owner); the page writes the choice in words
/// under it ("Accepted · owner X"), so it is not told by the pressed state
/// alone.
#[component]
pub fn OwnerChips(
    #[prop(into)] text: String,
    members: Vec<String>,
    #[prop(into)] owner: Signal<Option<String>>,
    on_pick: Callback<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2">
            <div role="group" aria-label=format!("Owner of: {text}") class="flex flex-wrap gap-2" data-owner-chips="">
                {members
                    .into_iter()
                    .map(|m| {
                        let name = m.clone();
                        let pressed = Signal::derive({
                            let m = m.clone();
                            move || owner.get().is_some_and(|o| o.eq_ignore_ascii_case(&m))
                        });
                        view! {
                            <Button
                                size=ButtonSize::Touch
                                pressed=pressed
                                on_click=Callback::new(move |_| {
                                    let chosen = owner.get_untracked().is_some_and(|o| o.eq_ignore_ascii_case(&name));
                                    on_pick.run((!chosen).then(|| name.clone()));
                                })
                            >
                                {m}
                            </Button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

/// The facilitator's quiet nudge: a count of players quiet in the last
/// three reviews, never who. `NudgeLine` markup without an action link
/// (there is no honest action), with "Don't show this again".
///
/// A11y: a paragraph and one button; not a live region (it renders with
/// the page); muting hides it at once.
#[component]
pub fn QuietNudge(count: u8) -> impl IntoView {
    use crate::models::guidance::NudgeKind;
    let key = NudgeKind::QuietPlayer.key();
    let hidden = RwSignal::new(false);
    let mute = move |_| {
        hidden.set(true);
        leptos::task::spawn_local(async move {
            let _ = crate::components::guidance::mark_guidance(key.to_string(), true).await;
        });
    };
    view! {
        <Show when=move || !hidden.get()>
            <p class="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-t border-rule pt-3 text-body text-secondary" data-nudge=key>
                <span>{crate::models::review_together::quiet_text(count)}</span>
                <button
                    type="button"
                    class="cursor-pointer text-label text-muted underline decoration-rule underline-offset-4 hover:text-primary"
                    on:click=mute
                    data-testid="mute-nudge"
                >
                    "Don\u{2019}t show this again"
                </button>
            </p>
        </Show>
    }
}

/// A quiet text button in the `NudgeLine` link grammar, for a disclosure
/// toggle ("Round controls", "2 groups read alike").
///
/// A11y: a native button with `aria-expanded` and `aria-controls`.
#[component]
pub fn Disclosure(
    #[prop(into)] label: Signal<String>,
    open: RwSignal<bool>,
    #[prop(into)] controls: String,
) -> impl IntoView {
    view! {
        <Button
            variant=ButtonVariant::Quiet
            size=ButtonSize::Touch
            on_click=Callback::new(move |_| open.update(|o| *o = !*o))
            attr:aria-expanded=move || if open.get() { "true" } else { "false" }
            attr:aria-controls=controls
        >
            {move || label.get()}
        </Button>
    }
}

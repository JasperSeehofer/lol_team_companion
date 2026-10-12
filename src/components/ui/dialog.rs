//! Dialog: a modal on the native `<dialog>` element. The browser gives the
//! focus trap, Escape and an inert background; this component adds the
//! header grammar, the scrim, focus return and the confirm variant.

use leptos::prelude::*;

use super::button::{Button, ButtonVariant};

/// Width of a [`Dialog`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    /// 24rem: confirmations.
    Sm,
    /// 32rem: short forms (the default).
    #[default]
    Md,
    /// 44rem: longer forms, previews.
    Lg,
    /// The whole viewport: the phone menu.
    Sheet,
}

impl DialogSize {
    pub fn class(self) -> &'static str {
        match self {
            DialogSize::Sm => "m-auto max-h-[calc(100dvh-2rem)] w-[min(24rem,calc(100vw-2rem))]",
            DialogSize::Md => "m-auto max-h-[calc(100dvh-2rem)] w-[min(32rem,calc(100vw-2rem))]",
            DialogSize::Lg => "m-auto max-h-[calc(100dvh-2rem)] w-[min(44rem,calc(100vw-2rem))]",
            DialogSize::Sheet => "m-0 h-dvh max-h-dvh w-screen max-w-none border-0",
        }
    }
}

const DIALOG: &str = "overflow-visible border border-outline bg-ink-1 p-0 text-primary shadow-overlay backdrop:bg-scrim open:flex open:flex-col";

/// A modal dialog. Open and close it through `open`; Escape, the close
/// button and a click on the scrim set it back to `false`.
///
/// A11y: native `<dialog>` opened with `showModal()`, so focus moves inside
/// (to the first `autofocus` element, else the first focusable one), Tab
/// stays inside, the page behind is inert and Escape closes. `aria-modal`,
/// `aria-labelledby` (the `<h2>` title) and `aria-describedby` (the
/// description) are set. On close, focus returns to the element that was
/// focused before it opened. The scrim is opaque (no blur) and page scroll
/// is locked while open (`input.css`). Server and browser render the same
/// closed `<dialog>`.
#[component]
pub fn Dialog(
    /// Prefix for the DOM ids of the title and description; unique on the page.
    id: &'static str,
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] size: DialogSize,
    /// Close when the scrim is clicked (default true; false for forms
    /// that would lose input).
    #[prop(default = true)]
    dismiss_on_scrim: bool,
    /// Actions, right-aligned under the content (primary last).
    #[prop(optional, into)]
    footer: Option<ViewFn>,
    /// Content between the header and the footer (none for confirmations).
    #[prop(optional)]
    children: Option<ChildrenFn>,
) -> impl IntoView {
    let dialog_ref = NodeRef::<leptos::html::Dialog>::new();
    let title_id = format!("{id}-title");
    let desc_id = format!("{id}-desc");
    let described_by = description.is_some().then(|| desc_id.clone());

    #[cfg(feature = "hydrate")]
    let prev_focus = StoredValue::new_local(None::<web_sys::HtmlElement>);

    Effect::new(move |_| {
        let want_open = open.get();
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let Some(dialog) = dialog_ref.get() else {
                return;
            };
            if want_open && !dialog.open() {
                let active = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                    .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
                prev_focus.set_value(active);
                let _ = dialog.show_modal();
            } else if !want_open && dialog.open() {
                dialog.close();
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = (want_open, dialog_ref);
    });

    // Fired by `close()`, Escape (after `cancel`) and form method=dialog.
    let on_close = move |_| {
        open.set(false);
        #[cfg(feature = "hydrate")]
        if let Some(el) = prev_focus.get_value() {
            let _ = el.focus();
        }
    };
    // A click whose target is the <dialog> itself landed on the scrim
    // (the content wrapper covers the whole box).
    let on_click = move |ev: leptos::ev::MouseEvent| {
        if !dismiss_on_scrim {
            return;
        }
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let target = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok());
            if let (Some(t), Some(d)) = (target, dialog_ref.get()) {
                let d: &web_sys::Element = d.as_ref();
                if t == *d {
                    open.set(false);
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = ev;
    };

    let class = format!("{DIALOG} {}", size.class());
    let title_id_attr = title_id.clone();
    view! {
        <dialog
            node_ref=dialog_ref
            class=class
            aria-modal="true"
            aria-labelledby=title_id_attr
            aria-describedby=described_by
            data-dialog=id
            on:close=on_close
            on:click=on_click
        >
            <div class="flex min-h-0 flex-col">
                <div class="flex items-start gap-4 border-b border-rule px-5 pb-4 pt-5">
                    <div class="flex min-w-0 flex-1 flex-col gap-1">
                        <h2 id=title_id class="font-display text-heading font-semibold text-primary">{title}</h2>
                        {description.map(|d| view! {
                            <p id=desc_id class="text-body text-secondary">{d}</p>
                        })}
                    </div>
                    <button
                        type="button"
                        class="-mr-2 -mt-1 grid size-8 shrink-0 cursor-pointer place-items-center rounded-ctl text-muted transition-colors hover:bg-ink-3 hover:text-primary"
                        aria-label="Close"
                        on:click=move |_| open.set(false)
                    >
                        <svg class="size-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                            <path d="M4 4l8 8M12 4l-8 8" stroke-linecap="round" />
                        </svg>
                    </button>
                </div>
                {children.map(|c| view! {
                    <div class="min-h-0 overflow-y-auto px-5 py-4 text-body text-primary">{c()}</div>
                })}
                {footer.map(|f| view! {
                    <div class="flex flex-wrap items-center justify-end gap-2 border-t border-rule px-5 py-3">{f.run()}</div>
                })}
            </div>
        </dialog>
    }
}

/// Whether a confirmation destroys something.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConfirmTone {
    /// Primary confirm button, focused first.
    #[default]
    Default,
    /// Danger confirm button; Cancel is focused first so Enter is safe.
    Destructive,
}

/// A confirmation: title, one sentence about the consequence, Cancel and
/// the confirm action (named for what it does: "Delete plan").
///
/// A11y: a [`Dialog`] (size Sm). For destructive confirmations Cancel has
/// initial focus, so pressing Enter by habit does not destroy anything;
/// otherwise the confirm button has it. The consequence sentence is the
/// dialog's description.
#[component]
pub fn ConfirmDialog(
    id: &'static str,
    open: RwSignal<bool>,
    #[prop(into)] title: String,
    /// What happens if confirmed: "The plan and its notes are deleted."
    #[prop(into)]
    message: String,
    #[prop(into)] confirm_label: String,
    #[prop(default = "Cancel".to_string(), into)] cancel_label: String,
    #[prop(optional)] tone: ConfirmTone,
    on_confirm: Callback<()>,
) -> impl IntoView {
    let destructive = tone == ConfirmTone::Destructive;
    let confirm_variant = if destructive {
        ButtonVariant::Danger
    } else {
        ButtonVariant::Primary
    };
    let confirm_label = StoredValue::new(confirm_label);
    let cancel_label = StoredValue::new(cancel_label);
    let confirm = Callback::new(move |_| {
        on_confirm.run(());
        open.set(false);
    });
    let cancel = Callback::new(move |_| open.set(false));
    view! {
        <Dialog
            id=id
            open=open
            title=title
            description=message
            size=DialogSize::Sm
            footer=move || view! {
                <Button variant=ButtonVariant::Secondary on_click=cancel attr:autofocus=destructive attr:data-confirm="cancel">
                    {cancel_label.get_value()}
                </Button>
                <Button variant=confirm_variant on_click=confirm attr:autofocus=!destructive attr:data-confirm="confirm">
                    {confirm_label.get_value()}
                </Button>
            }
        />
    }
}

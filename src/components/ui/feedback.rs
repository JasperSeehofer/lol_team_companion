use leptos::callback::Callback;
use leptos::prelude::*;

/// An inline error message for page-level and resource errors.
///
/// A11y: `role="alert"`, so it is announced when it appears; the icon is
/// decorative and the text says what went wrong. Danger text is 4.5:1 on
/// every layer. (Batch 2 replaces this with `Callout`.)
#[component]
pub fn ErrorBanner(message: String) -> impl IntoView {
    view! {
        <div role="alert" class="flex items-start gap-3 border border-danger/40 bg-danger/10 rounded-ctl px-4 py-3">
            <svg class="mt-0.5 size-4 shrink-0 text-danger" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
                <circle cx="8" cy="8" r="6.25" />
                <path d="M8 4.75v3.75M8 10.75v.5" stroke-linecap="round" />
            </svg>
            <p class="text-body text-danger">{message}</p>
        </div>
    }
}

/// A success or error line, chosen by whether the text starts with "Error".
///
/// A11y: `role="status"` (polite); the tone is repeated in the words.
#[component]
pub fn StatusMessage(message: String) -> impl IntoView {
    let is_err = message.starts_with("Error");
    let cls = if is_err {
        "border border-danger/40 bg-danger/10 text-danger rounded-ctl px-4 py-3 text-body"
    } else {
        "border border-success/40 bg-success/10 text-success rounded-ctl px-4 py-3 text-body"
    };
    view! {
        <div role="status" class=cls>{message}</div>
    }
}

// ── Toast System ──────────────────────────────────────────────────────────────

/// The kind of a toast notification.
#[derive(Clone, Debug, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

/// A single toast notification entry.
#[derive(Clone, Debug)]
pub struct ToastEntry {
    pub id: u64,
    pub kind: ToastKind,
    pub message: String,
}

/// Context type injected by ToastProvider — pages call `toast.show.run(...)`.
#[derive(Clone, Copy)]
pub struct ToastContext {
    pub show: Callback<(ToastKind, String)>,
}

/// Provides toast notifications to the entire app tree.
/// Must be placed unconditionally in App (not in a cfg block) so SSR can call use_context.
///
/// A11y: success toasts are `role="status"` (polite) and leave after 4 s;
/// errors stay until dismissed and are announced assertively. Toasts never
/// take focus; the dismiss and copy buttons are at least 24px.
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::<ToastEntry>::new());
    let next_id = StoredValue::new(std::sync::atomic::AtomicU64::new(0u64));

    let show = Callback::new(move |(kind, message): (ToastKind, String)| {
        let id = next_id.with_value(|c| c.fetch_add(1, std::sync::atomic::Ordering::Relaxed));

        // Enforce max 3 visible: remove oldest success toast before pushing if at capacity
        set_toasts.update(|v| {
            if v.len() >= 3 {
                if let Some(pos) = v.iter().position(|t| t.kind == ToastKind::Success) {
                    v.remove(pos);
                }
            }
            v.push(ToastEntry {
                id,
                kind: kind.clone(),
                message,
            });
        });

        // Auto-dismiss success toasts after 4 seconds
        if kind == ToastKind::Success {
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::prelude::*;
                let cb = Closure::once(move || {
                    set_toasts.update(|v| v.retain(|t| t.id != id));
                });
                if let Some(win) = web_sys::window() {
                    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(),
                        4000,
                    );
                }
                cb.forget();
            }
        }
    });

    provide_context(ToastContext { show });

    view! {
        {children()}
        <ToastOverlay toasts=toasts set_toasts=set_toasts />
    }
}

/// Internal overlay component: renders all active toasts at the top centre.
///
/// A11y: see `ToastProvider`.
#[component]
fn ToastOverlay(
    toasts: ReadSignal<Vec<ToastEntry>>,
    set_toasts: WriteSignal<Vec<ToastEntry>>,
) -> impl IntoView {
    view! {
        <div class="fixed top-16 left-1/2 -translate-x-1/2 z-(--z-toast) flex flex-col gap-2 items-center pointer-events-none">
            {move || toasts.get().into_iter().map(|t| {
                let id = t.id;
                let is_error = t.kind == ToastKind::Error;
                let msg = t.message.clone();
                #[allow(unused_variables)]
                let msg_for_copy = t.message.clone();
                let base = if is_error {
                    "pointer-events-auto flex items-start gap-3 bg-ink-2 border border-danger/60 text-danger rounded-ctl px-4 py-3 text-body shadow-overlay min-w-64 max-w-sm"
                } else {
                    "pointer-events-auto flex items-center gap-3 bg-ink-2 border border-success/60 text-success rounded-ctl px-4 py-3 text-body shadow-overlay min-w-64 max-w-sm"
                };
                view! {
                    <div role="status" aria-live=if is_error { "assertive" } else { "polite" } class=base>
                        <span class="flex-1">{msg}</span>
                        {if is_error {
                            view! {
                                <div class="flex gap-1 shrink-0">
                                    <button
                                        class="text-danger hover:bg-danger/15 text-caption min-h-6 min-w-6 px-1.5 rounded-ctl transition-colors cursor-pointer"
                                        on:click=move |_| {
                                            #[cfg(feature = "hydrate")]
                                            {
                                                let msg = msg_for_copy.clone();
                                                if let Some(win) = web_sys::window() {
                                                    let _ = win.navigator().clipboard().write_text(&msg);
                                                }
                                            }
                                        }
                                    >"Copy"</button>
                                    <button
                                        class="text-danger hover:bg-danger/15 text-caption min-h-6 min-w-6 px-1.5 rounded-ctl transition-colors cursor-pointer"
                                        on:click=move |_| set_toasts.update(|v| v.retain(|t| t.id != id))
                                    aria-label="Dismiss">"\u{00d7}"</button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

// ── Skeleton Primitives ───────────────────────────────────────────────────────

/// A loading placeholder line. Width and height are Tailwind w-*/h-* classes.
///
/// A11y: decorative (`aria-hidden`); the surrounding `Suspense` region is
/// what announces loading. The pulse stops under reduced motion.
#[component]
pub fn SkeletonLine(
    #[prop(default = "w-full")] width: &'static str,
    #[prop(default = "h-4")] height: &'static str,
) -> impl IntoView {
    let cls =
        format!("animate-pulse motion-reduce:animate-none bg-ink-2 rounded-ctl {width} {height}");
    view! { <div class=cls aria-hidden="true"></div> }
}

/// A block-shaped loading placeholder. Height is a Tailwind h-* class.
///
/// A11y: decorative (`aria-hidden`), static under reduced motion.
#[component]
pub fn SkeletonCard(#[prop(default = "h-24")] height: &'static str) -> impl IntoView {
    let cls = format!(
        "animate-pulse motion-reduce:animate-none bg-ink-1 border border-rule {height} w-full"
    );
    view! { <div class=cls aria-hidden="true"></div> }
}

/// A grid of skeleton blocks. cols must be 2, 3, or 4 (defaults to 3).
///
/// A11y: decorative, like its blocks.
#[component]
pub fn SkeletonGrid(
    #[prop(default = 3u8)] cols: u8,
    #[prop(default = 2u8)] rows: u8,
    #[prop(default = "h-20")] card_height: &'static str,
) -> impl IntoView {
    let col_class = match cols {
        2 => "grid grid-cols-2 gap-3",
        3 => "grid grid-cols-3 gap-3",
        4 => "grid grid-cols-4 gap-3",
        _ => "grid grid-cols-1 gap-3",
    };
    let items: Vec<u8> = (0..cols * rows).collect();
    view! {
        <div class=col_class aria-hidden="true">
            {items.into_iter().map(|_| view! { <SkeletonCard height=card_height /> }).collect_view()}
        </div>
    }
}

// ── Empty States ──────────────────────────────────────────────────────────────

/// An empty state: says what will appear here and how, with an optional link.
///
/// A11y: plain text plus a real link for the next step; no decorative icon
/// or emoji. Write the message as a sentence that names the action
/// (honesty rule: never fake data instead).
#[component]
pub fn EmptyState(
    message: &'static str,
    #[prop(optional)] cta_label: Option<&'static str>,
    #[prop(optional)] cta_href: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="py-10 flex flex-col items-start gap-3 border-t border-rule">
            <p class="text-body text-secondary max-w-prose">{message}</p>
            {match (cta_label, cta_href) {
                (Some(label), Some(href)) => view! {
                    <a href=href
                       class="inline-flex h-8 items-center rounded-ctl border border-outline px-3 text-label font-medium text-primary transition-colors hover:bg-ink-3">
                        {label}
                    </a>
                }.into_any(),
                _ => ().into_any(),
            }}
        </div>
    }
}

/// Consistent "no team" state used by all team-scoped pages.
///
/// A11y: as `EmptyState`.
#[component]
pub fn NoTeamState() -> impl IntoView {
    view! {
        <EmptyState
            message="Team pages need a team. Create one or ask to join one on the roster page; your drafts, plans and reviews will appear here."
            cta_label="Go to the roster"
            cta_href="/team/roster"
        />
    }
}

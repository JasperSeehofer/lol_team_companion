use leptos::prelude::*;
use leptos::callback::Callback;

/// A styled error banner for page-level and resource errors.
#[component]
pub fn ErrorBanner(message: String) -> impl IntoView {
    view! {
        <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-4 flex items-start gap-3">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-red-400 shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
            <p class="text-red-400 text-sm">{message}</p>
        </div>
    }
}

/// A styled status message that shows success (green) or error (red) based on content.
#[component]
pub fn StatusMessage(message: String) -> impl IntoView {
    let is_err = message.starts_with("Error");
    let cls = if is_err {
        "bg-red-500/10 border border-red-500/30 text-red-400 rounded-xl px-4 py-3 text-sm"
    } else {
        "bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 rounded-xl px-4 py-3 text-sm"
    };
    view! {
        <div class=cls>{message}</div>
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
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::<ToastEntry>::new());
    let next_id = StoredValue::new(std::sync::atomic::AtomicU64::new(0u64));

    let show = Callback::new(move |(kind, message): (ToastKind, String)| {
        let id = next_id.with_value(|c| {
            c.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        });

        // Enforce max 3 visible: remove oldest success toast before pushing if at capacity
        set_toasts.update(|v| {
            if v.len() >= 3 {
                if let Some(pos) = v.iter().position(|t| t.kind == ToastKind::Success) {
                    v.remove(pos);
                }
            }
            v.push(ToastEntry { id, kind: kind.clone(), message });
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

/// Internal overlay component — renders all active toasts at top-center of screen.
#[component]
fn ToastOverlay(
    toasts: ReadSignal<Vec<ToastEntry>>,
    set_toasts: WriteSignal<Vec<ToastEntry>>,
) -> impl IntoView {
    view! {
        <div class="fixed top-16 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 items-center pointer-events-none">
            {move || toasts.get().into_iter().map(|t| {
                let id = t.id;
                let is_error = t.kind == ToastKind::Error;
                let msg = t.message.clone();
                #[allow(unused_variables)]
                let msg_for_copy = t.message.clone();
                let base = if is_error {
                    "pointer-events-auto flex items-start gap-3 bg-red-500/20 border border-red-500/40 text-red-300 rounded-xl px-4 py-3 text-sm shadow-lg min-w-64 max-w-sm"
                } else {
                    "pointer-events-auto flex items-center gap-3 bg-emerald-500/20 border border-emerald-500/40 text-emerald-300 rounded-xl px-4 py-3 text-sm shadow-lg min-w-64 max-w-sm"
                };
                view! {
                    <div class=base>
                        <span class="flex-1">{msg}</span>
                        {if is_error {
                            view! {
                                <div class="flex gap-1 shrink-0">
                                    <button
                                        class="text-red-400/70 hover:text-red-300 text-xs px-1.5 py-0.5 rounded hover:bg-red-500/20 transition-colors"
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
                                        class="text-red-400/70 hover:text-red-300 text-xs px-1.5 py-0.5 rounded hover:bg-red-500/20 transition-colors"
                                        on:click=move |_| set_toasts.update(|v| v.retain(|t| t.id != id))
                                    >"×"</button>
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

/// A single animated skeleton line. Width and height are Tailwind w-*/h-* classes.
#[component]
pub fn SkeletonLine(
    #[prop(default = "w-full")] width: &'static str,
    #[prop(default = "h-4")] height: &'static str,
) -> impl IntoView {
    let cls = format!("animate-pulse bg-elevated rounded {width} {height}");
    view! { <div class=cls></div> }
}

/// A card-shaped animated skeleton block. Height is a Tailwind h-* class.
#[component]
pub fn SkeletonCard(#[prop(default = "h-24")] height: &'static str) -> impl IntoView {
    let cls = format!("animate-pulse bg-elevated rounded-xl border border-divider/30 {height} w-full");
    view! { <div class=cls></div> }
}

/// A grid of skeleton cards. cols must be 2, 3, or 4 (defaults to 3).
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
        <div class=col_class>
            {items.into_iter().map(|_| view! { <SkeletonCard height=card_height /> }).collect_view()}
        </div>
    }
}

// ── Empty States ──────────────────────────────────────────────────────────────

/// A full-width empty state placeholder with optional icon and CTA.
/// Backward-compatible: existing `<EmptyState message="..." />` call sites continue to compile.
#[component]
pub fn EmptyState(
    message: &'static str,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] cta_label: Option<&'static str>,
    #[prop(optional)] cta_href: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="text-center py-12 flex flex-col items-center gap-3">
            {match icon {
                Some(i) => view! { <span class="text-4xl">{i}</span> }.into_any(),
                None => view! { <span></span> }.into_any(),
            }}
            <p class="text-secondary text-sm max-w-xs">{message}</p>
            {match (cta_label, cta_href) {
                (Some(label), Some(href)) => view! {
                    <a href=href
                       class="mt-1 bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors">
                        {label}
                    </a>
                }.into_any(),
                _ => view! { <span></span> }.into_any(),
            }}
        </div>
    }
}

/// Consistent "no team" state used by all team-scoped pages.
#[component]
pub fn NoTeamState() -> impl IntoView {
    view! {
        <EmptyState
            icon="👥"
            message="You need a team to use this feature. Create or join a team to get started."
            cta_label="Go to Team Roster"
            cta_href="/team/roster"
        />
    }
}

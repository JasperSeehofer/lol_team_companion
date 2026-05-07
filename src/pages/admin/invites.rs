//! Phase 17 plan 17-06 — Admin invites page (visual stub).
//!
//! D-13 utility tier + D-14 admin gate. Per UI-SPEC §"Admin invite
//! page" lines 558–567 — the visual layout lands in Phase 17;
//! Phase 19.1 wires the route guard (404-not-403 to avoid leaking
//! existence to non-admins) and the real `invite_code` table writes.
//!
//! For Phase 17 the page renders for any auth user with placeholder
//! data. The threat register (T-17-27) records the deferred mitigation.

use leptos::prelude::*;

use crate::components::ornaments::HeraldicDivider;

/// Static demo rows for the visual stub. Phase 19.1 replaces these
/// with `db::list_invite_codes()` against a real `invite_code` table.
struct InviteRow {
    code: &'static str,
    created: &'static str,
    consumed_by: Option<&'static str>,
    consumed_at: Option<&'static str>,
}

const DEMO_ROWS: &[InviteRow] = &[
    InviteRow {
        code: "STRAT-7K9X-22A4",
        created: "2026-04-30",
        consumed_by: None,
        consumed_at: None,
    },
    InviteRow {
        code: "ROOM-MN3Q-88FE",
        created: "2026-05-01",
        consumed_by: Some("midbean"),
        consumed_at: Some("2026-05-02"),
    },
    InviteRow {
        code: "FOLIO-2WJ1-04XX",
        created: "2026-05-04",
        consumed_by: None,
        consumed_at: None,
    },
];

#[component]
pub fn AdminInvitesPage() -> impl IntoView {
    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-6">
            <div class="max-w-5xl mx-auto flex flex-col gap-6">
                // Page header per UI-SPEC line 562
                <header>
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                        "Invite Management"
                    </div>
                    <div class="flex items-baseline justify-between mt-2 gap-4">
                        <h1 class="font-display italic text-[44px] leading-tight text-primary">
                            "Beta invitations"
                        </h1>
                        <button
                            type="button"
                            class="bg-accent text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                        >
                            "Mint invite code"
                        </button>
                    </div>
                    <div class="mt-3"><HeraldicDivider width=320 /></div>
                </header>

                // Phase 19.1 advisory banner — visual stub disclosure
                <div class="bg-elevated border border-outline/50 rounded-lg p-3">
                    <p class="text-xs text-muted">
                        <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-warning mr-2">
                            "Stub"
                        </span>
                        "Visual layout only \u{2014} Phase 19.1 wires real invite_code writes and admin gate."
                    </p>
                </div>

                // Form card — mint new invite (visual stub)
                <section class="bg-surface border border-divider rounded-xl p-6">
                    <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted mb-3">
                        "Mint code"
                    </div>
                    <div class="flex gap-3 items-end">
                        <div class="flex-1">
                            <label class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                "Memo (optional)"
                            </label>
                            <input
                                type="text"
                                placeholder="e.g. \"For Aria — top laner\""
                                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            />
                        </div>
                        <button
                            type="button"
                            class="bg-accent text-accent-contrast font-semibold rounded-lg px-4 py-3 text-sm hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                        >
                            "Generate"
                        </button>
                    </div>
                </section>

                // Table card — issued codes
                <section class="bg-surface border border-divider rounded-xl overflow-hidden">
                    <div class="px-6 py-4 border-b border-divider">
                        <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                            "Issued codes"
                        </div>
                        <h2 class="font-display italic text-[20px] text-primary mt-1">
                            "All invitations"
                        </h2>
                    </div>
                    <table class="w-full">
                        <thead class="bg-elevated">
                            <tr class="text-left">
                                <th class="px-6 py-3 text-xs text-muted uppercase tracking-wider font-imperial">"Code"</th>
                                <th class="px-6 py-3 text-xs text-muted uppercase tracking-wider font-imperial">"Created"</th>
                                <th class="px-6 py-3 text-xs text-muted uppercase tracking-wider font-imperial">"Consumed by"</th>
                                <th class="px-6 py-3 text-xs text-muted uppercase tracking-wider font-imperial">"Consumed at"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {DEMO_ROWS.iter().map(|row| {
                                let used = row.consumed_by.is_some();
                                let status_class = if used { "text-dimmed" } else { "text-muted" };
                                let consumed_by = row.consumed_by.unwrap_or("\u{2014}");
                                let consumed_at = row.consumed_at.unwrap_or("\u{2014}");
                                view! {
                                    <tr class="border-t border-divider/50 hover:bg-elevated/50 transition-colors">
                                        <td class="px-6 py-4">
                                            <code class="font-mono text-[12px] text-primary">{row.code}</code>
                                        </td>
                                        <td class="px-6 py-4 text-sm text-secondary tabular-nums">{row.created}</td>
                                        <td class=format!("px-6 py-4 text-sm {}", status_class)>
                                            {if used { consumed_by } else { "Unused" }}
                                        </td>
                                        <td class=format!("px-6 py-4 text-sm tabular-nums {}", status_class)>
                                            {consumed_at}
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </section>
            </div>
        </div>
    }
}

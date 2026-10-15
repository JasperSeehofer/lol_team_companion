//! `/design` gallery sections for batch 2 (interaction primitives).
//! All content is fixture text.

use leptos::prelude::*;

use crate::components::ui::toast::ToastItem;
use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, Checkbox, Command, CommandBar,
    ConfirmDialog, ConfirmTone, Dialog, DialogSize, EmptySize, EmptyState, Field, Force, Keycap,
    Menu, MenuAlign, MenuItem, Segment, SegmentedControl, Skeleton, SkeletonGroup, SkeletonShape,
    Switch, TextField, Toast, ToastContext, ToastEntry, ToastKind, Tooltip, TooltipPlacement,
};
use crate::components::ui::{LoadFailed, SaveState, SaveStatus};
use crate::pages::design::{GallerySection, Sample};

fn info_icon() -> impl IntoView {
    view! {
        <svg class="size-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6">
            <circle cx="8" cy="8" r="6.25" />
            <path d="M8 7.25v4M8 4.75v.5" stroke-linecap="round" />
        </svg>
    }
}

#[component]
pub fn DialogSection() -> impl IntoView {
    let form_open = RwSignal::new(false);
    let confirm_open = RwSignal::new(false);
    let result = RwSignal::new("Nothing confirmed yet".to_string());
    let name = RwSignal::new("Blue side plan".to_string());
    let on_confirm = Callback::new(move |_| {
        result.set("Plan deleted (fixture: nothing was deleted)".to_string())
    });
    let save = Callback::new(move |_| {
        result.set(format!(
            "Saved \u{201c}{}\u{201d} (fixture)",
            name.get_untracked()
        ));
        form_open.set(false);
    });
    let cancel = Callback::new(move |_| form_open.set(false));
    view! {
        <GallerySection id="dialog" title="Dialog" lede="Native modal dialog: focus moves in, Tab stays inside, Escape or the scrim closes, focus returns to the button. The confirm variant names the action; for destructive actions Cancel has focus first.">
            <div class="flex flex-wrap items-center gap-3" data-testid="dialog-demo">
                <Button on_click=Callback::new(move |_| form_open.set(true)) attr:data-testid="open-dialog">"Rename plan"</Button>
                <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| confirm_open.set(true)) attr:data-testid="open-confirm">"Delete plan"</Button>
                <output class="text-label text-secondary" data-testid="dialog-result" aria-live="polite">{move || result.get()}</output>
            </div>
            <Dialog
                id="gallery-dialog"
                open=form_open
                title="Rename plan"
                description="The new name shows on Match Day and in the plan list."
                size=DialogSize::Md
                footer=move || view! {
                    <Button variant=ButtonVariant::Secondary on_click=cancel>"Cancel"</Button>
                    <Button variant=ButtonVariant::Primary on_click=save>"Save name"</Button>
                }
            >
                <Field id="gallery-dialog-name" label="Plan name">
                    <TextField value=name />
                </Field>
            </Dialog>
            <ConfirmDialog
                id="gallery-confirm"
                open=confirm_open
                title="Delete this plan?"
                message="The plan and its notes are deleted for the whole team. This cannot be undone."
                confirm_label="Delete plan"
                tone=ConfirmTone::Destructive
                on_confirm=on_confirm
            />
        </GallerySection>
    }
}

#[component]
pub fn MenuSection() -> impl IntoView {
    let picked = RwSignal::new("Nothing chosen yet".to_string());
    let on_select = Callback::new(move |id: String| picked.set(format!("Chose: {id}")));
    let items = vec![
        MenuItem::new("duplicate", "Duplicate"),
        MenuItem::new("rename", "Rename").shortcut("R"),
        MenuItem::new("export", "Export as text").disabled(),
        MenuItem::link("open", "Open on Match Day", "#menu"),
        MenuItem::new("delete", "Delete plan").danger(),
    ];
    let items_forced = items.clone();
    view! {
        <GallerySection id="menu" title="Menu" lede="A button that opens a short list of actions. Down or Enter opens and focuses the first item, arrows move, a letter jumps, Escape closes and returns focus. Destructive items sit last, under a rule, and say what they do.">
            <div class="grid gap-8 sm:grid-cols-2">
                <Sample label="Default (closed)">
                    <div class="flex flex-col items-start gap-2" data-testid="menu-demo">
                        <Menu id="gallery-menu" label="Plan actions" items=items on_select=on_select />
                        <output class="text-label text-secondary" data-testid="menu-result">{move || picked.get()}</output>
                    </div>
                </Sample>
                <Sample label="Open (forced), aligned to the end">
                    <div class="flex min-h-64 w-full justify-end">
                        <Menu id="gallery-menu-open" label="Plan actions" items=items_forced align=MenuAlign::End force_open=true />
                    </div>
                </Sample>
            </div>
        </GallerySection>
    }
}

fn gallery_commands() -> Vec<Command> {
    vec![
        Command::go("Draft", "#command")
            .hint("Strategy")
            .keywords("pick ban"),
        Command::go("Tree drafter", "#command").hint("Strategy"),
        Command::go("Game plan", "#command").hint("Strategy"),
        Command::go("Opponents", "#command")
            .hint("Strategy")
            .keywords("scouting dossier"),
        Command::go("Stats", "#command").hint("History"),
        Command {
            id: "toast".to_string(),
            label: "Show a test toast".to_string(),
            group: "Commands".to_string(),
            keywords: "notification".to_string(),
            href: None,
            hint: Some("Gallery".to_string()),
        },
    ]
}

#[component]
pub fn CommandSection() -> impl IntoView {
    let open = RwSignal::new(false);
    let toast = use_context::<ToastContext>();
    let ran = RwSignal::new("No command run yet".to_string());
    let on_run = Callback::new(move |id: String| {
        ran.set(format!("Ran: {id}"));
        if let Some(t) = toast {
            t.push.run(Toast::info("Test toast from the command bar"));
        }
    });
    view! {
        <GallerySection id="command" title="Command bar" lede="Ctrl+K (Cmd+K on a Mac) opens it from any page: type to filter, arrows move, Enter goes. Today it jumps to pages; commands arrive with the draft and Match Day work. This instance opens from the button only.">
            <div class="flex flex-wrap items-center gap-3" data-testid="command-demo">
                <Button on_click=Callback::new(move |_| open.set(true)) attr:data-testid="open-command">
                    "Open the command bar"
                </Button>
                <span class="text-caption text-muted">"In the nav: " <Keycap keys="Ctrl" /> " " <Keycap keys="K" /></span>
                <output class="text-label text-secondary" data-testid="command-result">{move || ran.get()}</output>
            </div>
            <CommandBar id="gallery-command" open=open commands=gallery_commands() on_run=on_run global_shortcut=false />
        </GallerySection>
    }
}

#[component]
pub fn ToastSection() -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let undone = RwSignal::new("Note not deleted".to_string());
    let push = move |t: Toast| {
        if let Some(ctx) = toast {
            ctx.push.run(t);
        }
    };
    let success = Callback::new(move |_| push(Toast::success("Game plan saved.")));
    let error = Callback::new(move |_| {
        push(Toast::error(
            "Could not save the plan. Your changes are kept; try again.",
        ))
    });
    let undo = Callback::new(move |_| {
        undone.set("Note deleted".to_string());
        let restore = Callback::new(move |_| undone.set("Note restored by Undo".to_string()));
        push(Toast::success("Team note deleted.").with_action("Undo", restore));
    });
    let warning = Callback::new(move |_| {
        push(Toast::new(
            ToastKind::Warning,
            "Sync is slow; results may be a few minutes old.",
        ))
    });
    let noop = Callback::new(|_: u64| {});
    let samples = vec![
        ToastEntry {
            id: 9001,
            kind: ToastKind::Success,
            message: "Game plan saved.".to_string(),
            action: None,
        },
        ToastEntry {
            id: 9002,
            kind: ToastKind::Success,
            message: "Team note deleted.".to_string(),
            action: Some(crate::components::ui::ToastAction {
                label: "Undo".to_string(),
                on_action: Callback::new(|_| {}),
            }),
        },
        ToastEntry {
            id: 9003,
            kind: ToastKind::Info,
            message: "Patch 16.20 data is in.".to_string(),
            action: None,
        },
        ToastEntry {
            id: 9004,
            kind: ToastKind::Warning,
            message: "Sync is slow; results may be a few minutes old.".to_string(),
            action: None,
        },
        ToastEntry {
            id: 9005,
            kind: ToastKind::Error,
            message: "Could not save the plan. Try again.".to_string(),
            action: None,
        },
    ];
    view! {
        <GallerySection id="toast" title="Toast" lede="Something just happened. Toasts stack bottom left (three at most) in one polite live region; success and info leave after 5 s, with an action or a warning after 8 s, errors stay. Hover or focus pauses. The action slot is where Undo lives (team notes in M5).">
            <div class="grid gap-8 lg:grid-cols-2">
                <div class="flex flex-col gap-3" data-testid="toast-demo">
                    <div class="flex flex-wrap gap-2">
                        <Button size=ButtonSize::Sm on_click=success attr:data-testid="toast-success">"Show success"</Button>
                        <Button size=ButtonSize::Sm on_click=undo attr:data-testid="toast-undo">"Delete a note (with Undo)"</Button>
                        <Button size=ButtonSize::Sm on_click=warning>"Show warning"</Button>
                        <Button size=ButtonSize::Sm on_click=error>"Show error"</Button>
                    </div>
                    <output class="text-label text-secondary" data-testid="toast-result">{move || undone.get()}</output>
                </div>
                <div>
                    <p class="eyebrow mb-3">"Every kind (static preview)"</p>
                    <ul class="flex flex-col gap-2" aria-label="Toast preview">
                        {samples
                            .into_iter()
                            .map(|e| view! { <ToastItem entry=e dismiss=noop pause=noop resume=noop /> })
                            .collect_view()}
                    </ul>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn CalloutSection() -> impl IntoView {
    view! {
        <GallerySection id="callout" title="Callout" lede="Inline messages that belong to a place. Each tone has a shape as well as a colour (circle, check, triangle with \u{201c}!\u{201d}, square), and the tone is said in words to screen readers. Danger and warning interrupt; info and success wait.">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <Callout tone=CalloutTone::Info>"Scouting uses the last 20 games of each player."</Callout>
                <Callout tone=CalloutTone::Success>"Plan saved. The team sees it on Match Day."</Callout>
                <Callout tone=CalloutTone::Warning title="Two open points in the game plan">
                    "Settle the dragon setup and the level 6 timing before 19:00."
                </Callout>
                <Callout
                    tone=CalloutTone::Danger
                    title="Could not load the opponent dossier"
                    action=|| view! { <Button size=ButtonSize::Sm>"Try again"</Button> }
                >
                    "The Riot API did not answer. Nothing was lost."
                </Callout>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn StatusSection() -> impl IntoView {
    let states = [
        ("Idle (nothing to say)", SaveState::Idle),
        ("Not saved yet", SaveState::NotSaved),
        ("Unsaved changes", SaveState::Unsaved),
        ("Saving", SaveState::Saving),
        ("Saved", SaveState::Saved),
        (
            "Failed, with Retry",
            SaveState::Failed("Ahri to Match ready".into()),
        ),
    ];
    view! {
        <GallerySection id="status" title="Save state and load failure" lede="SaveStatus is the one line in a page header that says whether the last change reached the server; it says the state in words and offers Retry when a save failed. LoadFailed replaces a section that did not load, with Try again.">
            <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                <div class="flex flex-col gap-3">
                    {states.into_iter().map(|(label, state)| view! {
                        <Sample label=label>
                            <SaveStatus state=Signal::stored(state) on_retry=Callback::new(|_| ()) />
                        </Sample>
                    }).collect_view()}
                </div>
                <LoadFailed what="Your pool" on_retry=Callback::new(|_| ()) />
            </div>
        </GallerySection>
    }
}

#[component]
pub fn EmptySection() -> impl IntoView {
    view! {
        <GallerySection id="empty" title="Empty state" lede="Says what will appear here and how it gets there, with at most one action. No icons, no invented data.">
            <div class="grid grid-cols-1 gap-8 md:grid-cols-2">
                <EmptyState
                    title="No draft plans yet"
                    message="Plans you save on the draft page appear here, newest first, with their side and opponent."
                    cta_label="Open the draft page"
                    cta_href="#empty"
                />
                <div class="flex flex-col gap-6">
                    <EmptyState
                        size=EmptySize::Compact
                        message="No action items for this review. Add one from the review form."
                    />
                    <EmptyState
                        size=EmptySize::Compact
                        title="Not synced"
                        message="Link your Riot account on your profile; your last 20 games appear here after the first sync."
                        action=|| view! { <Button size=ButtonSize::Sm variant=ButtonVariant::Primary>"Link account"</Button> }
                    />
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn SkeletonSection() -> impl IntoView {
    view! {
        <GallerySection id="skeleton" title="Skeleton" lede="The only pulsing thing in the app, and only while loading. Shapes follow the content they stand in for; one status says what is loading. Under reduced motion the shapes stay still.">
            <div class="grid grid-cols-1 gap-8 md:grid-cols-3">
                <Sample label="Lines (3, last shorter)">
                    <div class="w-64 max-w-full"><Skeleton lines=3 /></div>
                </Sample>
                <Sample label="Row: avatar and text">
                    <div class="flex w-64 max-w-full items-center gap-3">
                        <Skeleton shape=SkeletonShape::Circle size="size-8" />
                        <div class="flex flex-1 flex-col gap-2">
                            <Skeleton width="w-1/2" />
                            <Skeleton width="w-3/4" size="h-3" />
                        </div>
                    </div>
                </Sample>
                <Sample label="Group with a status (blocks)">
                    <div class="w-64 max-w-full">
                        <SkeletonGroup label="Loading draft plans">
                            <Skeleton shape=SkeletonShape::Block size="h-12" />
                            <Skeleton shape=SkeletonShape::Block size="h-12" />
                        </SkeletonGroup>
                    </div>
                </Sample>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn TooltipSection() -> impl IntoView {
    view! {
        <GallerySection id="tooltip" title="Tooltip" lede="A hint on hover and on keyboard focus, after 300ms. It repeats or explains what is already there and is never the only place information lives (phones have no hover). Escape hides it.">
            <div class="flex flex-wrap items-end gap-10 pt-10">
                <Sample label="Icon button, hover or focus">
                    <Tooltip id="tip-info" text="Fit is computed from your pool and the enemy picks.">
                        <Button variant=ButtonVariant::Quiet aria_label="About fit" icon=info_icon attr:data-testid="tooltip-trigger">""</Button>
                    </Tooltip>
                </Sample>
                <Sample label="Shown (forced), above">
                    <Tooltip id="tip-forced" text="Last surveyed 3 days ago" force_open=true>
                        <Button size=ButtonSize::Sm>"Dossier"</Button>
                    </Tooltip>
                </Sample>
                <Sample label="Below">
                    <Tooltip id="tip-below" text="Ctrl+S saves" placement=TooltipPlacement::Bottom>
                        <Button size=ButtonSize::Sm>"Save"</Button>
                    </Tooltip>
                </Sample>
            </div>
            <div class="h-8"></div>
        </GallerySection>
    }
}

#[component]
pub fn TogglesSection() -> impl IntoView {
    let mode = RwSignal::new("team".to_string());
    let side = RwSignal::new("blue".to_string());
    let fixed = RwSignal::new("solo".to_string());
    let notify = RwSignal::new(true);
    let compact = RwSignal::new(false);
    let off = RwSignal::new(false);
    let on = RwSignal::new(true);
    let manual = RwSignal::new("solo".to_string());
    let item_a = RwSignal::new(true);
    let item_b = RwSignal::new(false);
    let item_off = RwSignal::new(false);
    view! {
        <GallerySection id="toggles" title="Segmented control, switch and checkbox" lede="Segmented: one of two to five short options that switch a view at once (radio semantics, arrow keys, one Tab stop; manual activation when switching has a side effect). Switch: on or off, effective immediately, label always visible; round, because it is a toggle. Checkbox: a checklist item or a yes/no in a form; square.">
            <div class="grid grid-cols-1 gap-8 md:grid-cols-2">
                <div class="flex flex-col gap-5">
                    <Sample label="Default (live)">
                        <div class="flex flex-wrap items-center gap-3" data-testid="segmented-demo">
                            <SegmentedControl id="gal-mode" label="Workspace" options=vec![Segment::new("solo", "Solo"), Segment::new("team", "Team")] value=mode />
                            <output class="text-label text-secondary" data-testid="segmented-value">{move || format!("Workspace: {}", mode.get())}</output>
                        </div>
                    </Sample>
                    <Sample label="Three options, hover forced on the second">
                        <SegmentedControl
                            id="gal-side"
                            label="Side"
                            options=vec![Segment::new("blue", "Blue"), Segment::new("red", "Red"), Segment::new("both", "Both")]
                            value=side
                            force=Force::Hover
                        />
                    </Sample>
                    <Sample label="Disabled">
                        <SegmentedControl id="gal-fixed" label="Locked" options=vec![Segment::new("solo", "Solo"), Segment::new("team", "Team")] value=fixed disabled=true />
                    </Sample>
                    <Sample label="Manual activation: arrows move focus, Enter or Space selects">
                        <div class="flex flex-wrap items-center gap-3" data-testid="segmented-manual">
                            <SegmentedControl id="gal-manual" label="Workspace (manual)" options=vec![Segment::new("solo", "Solo"), Segment::new("team", "Team")] value=manual manual=true />
                            <output class="text-label text-secondary" data-testid="segmented-manual-value">{move || format!("Workspace: {}", manual.get())}</output>
                        </div>
                    </Sample>
                </div>
                <div class="flex flex-col gap-5">
                    <Sample label="Switch (live), with state written out">
                        <div data-testid="switch-demo"><Switch label="Notify me about join requests" checked=notify show_state=true /></div>
                    </Sample>
                    <Sample label="Off, hover forced">
                        <Switch label="Compact rows" checked=compact force=Force::Hover />
                    </Sample>
                    <Sample label="Focus forced">
                        <Switch label="Show estimated paths" checked=on force=Force::Focus />
                    </Sample>
                    <Sample label="Disabled">
                        <Switch label="Share with scouts" checked=off disabled=true />
                    </Sample>
                    <Sample label="Checkbox: a checklist (done items struck through)">
                        <div class="flex flex-col gap-2" data-testid="checkbox-demo">
                            <Checkbox label="Read the opponent brief" checked=item_a strike=true />
                            <Checkbox label="Agree on the level 1 plan" checked=item_b strike=true />
                            <Checkbox label="Disabled item" checked=item_off disabled=true />
                        </div>
                    </Sample>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn KeycapSection() -> impl IntoView {
    view! {
        <GallerySection id="keycap" title="Keycap" lede="Shortcuts drawn as keys teach the keyboard layer without a tutorial. Inline in text, in buttons (the shortcut slot) and in the command bar.">
            <div class="flex flex-wrap items-center gap-x-8 gap-y-4 text-label text-secondary">
                <span class="inline-flex items-center gap-1.5"><Keycap keys="Ctrl" /><Keycap keys="K" />" command bar"</span>
                <span class="inline-flex items-center gap-1.5"><Keycap keys="P" />" pick"</span>
                <span class="inline-flex items-center gap-1.5"><Keycap keys="B" />" ban"</span>
                <span class="inline-flex items-center gap-1.5"><Keycap keys="1" />"\u{2013}"<Keycap keys="5" />" role"</span>
                <span class="inline-flex items-center gap-1.5"><Keycap keys="J" /><Keycap keys="K" /><Keycap keys="L" />" shuttle"</span>
                <Button variant=ButtonVariant::Primary shortcut="P">"Pick Rakan"</Button>
            </div>
        </GallerySection>
    }
}

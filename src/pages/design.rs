//! `/design`: the design-system gallery (docs/DESIGN.md, "Gallery").
//!
//! For developers and admins: dev builds show it to everyone, otherwise
//! only admins (`ADMIN_EMAILS`) see it and everyone else gets the 404 page
//! (the route renders with `SsrMode::Async` so the status is set). Every
//! primitive appears in every variant and state, with forced hover, focus
//! and pressed states for screenshots. All content is fixture text.

use leptos::prelude::*;

use crate::components::ui::draft::TOURNAMENT_ORDER;
use crate::components::ui::tokens::{
    contrast_by_name, css_value, TokenGroup, GALLERY_PAIRS, TOKENS,
};
use crate::components::ui::{
    AvatarSize, Badge, BadgeTone, Button, ButtonSize, ButtonType, ButtonVariant, DraftBoard, Edge,
    Field, FieldSize, Force, HeadingLevel, InputKind, Keycap, Layer, Pad, PageHeader, PageShell,
    PageWidth, PrepEmphasis, PrepSequence, PrepState, PrepStep, RowLink, SectionHeader, Select,
    SelectOption, Surface, TabItem, TabPanel, Table, Tabs, Td, Text, TextField, TextRole, Th, Tone,
    Tr, UserAvatar,
};
use crate::error_template::NotFound;
use crate::pages::design_domain::{
    AvatarSection, BarSection, CanvasSection, LegendSection, MarkersSection, ReadoutSection,
    TimelineSection,
};
use crate::pages::design_interaction::{
    CalloutSection, CommandSection, DialogSection, EmptySection, KeycapSection, MenuSection,
    SkeletonSection, ToastSection, TogglesSection, TooltipSection,
};

/// Whether the caller may see the gallery: always in dev, else admins only.
#[server]
pub async fn gallery_access() -> Result<bool, ServerFnError> {
    use crate::server::{authz, config};

    if config::get().is_dev {
        return Ok(true);
    }
    authz::require_admin().await.map(|_| true)
}

const SECTIONS: &[(&str, &str)] = &[
    ("tokens", "Colour tokens"),
    ("contrast", "Contrast"),
    ("type", "Type"),
    ("space", "Space, radius, motion"),
    ("layout", "Layout"),
    ("text", "Text"),
    ("button", "Button"),
    ("fields", "Fields"),
    ("badge", "Badge"),
    ("table", "Table"),
    ("sort", "Sortable table"),
    ("tabs", "Tabs"),
    ("toggles", "Segmented and switch"),
    ("keycap", "Keycap"),
    ("callout", "Callout"),
    ("toast", "Toast"),
    ("empty", "Empty state"),
    ("skeleton", "Skeleton"),
    ("tooltip", "Tooltip"),
    ("dialog", "Dialog"),
    ("menu", "Menu"),
    ("command", "Command bar"),
    ("avatar", "Champion avatar"),
    ("markers", "Role, side, result"),
    ("readout", "Stat readout"),
    ("bar", "Proportion bar"),
    ("draft", "Tournament draft"),
    ("prep", "Prep sequence"),
    ("legend", "Legend block"),
    ("canvas", "Canvas frame"),
    ("timeline", "Timeline scrub"),
];

#[component]
pub fn DesignGalleryPage() -> impl IntoView {
    let access = Resource::new(|| (), |_| gallery_access());
    view! {
        <Suspense fallback=|| ()>
            {move || access.get().map(|r| match r {
                Ok(true) => view! { <Gallery /> }.into_any(),
                _ => view! { <NotFound /> }.into_any(),
            })}
        </Suspense>
    }
}

/// One gallery section: anchor, `data-gallery` hook for screenshots, h2.
#[component]
pub(crate) fn GallerySection(
    id: &'static str,
    title: &'static str,
    #[prop(into)] lede: String,
    children: Children,
) -> impl IntoView {
    let heading_id = format!("{id}-title");
    let labelled_by = heading_id.clone();
    view! {
        <section id=id data-gallery=id aria-labelledby=labelled_by class="scroll-mt-28 border-t border-rule py-10">
            <SectionHeader title=title id=heading_id />
            <p class="-mt-1 mb-6 max-w-prose text-body text-secondary">{lede}</p>
            {children()}
        </section>
    }
}

/// A sample with a state label under it.
#[component]
pub(crate) fn Sample(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col items-start gap-2">
            {children()}
            <span class="text-caption text-muted">{label}</span>
        </div>
    }
}

#[component]
fn Gallery() -> impl IntoView {
    view! {
        <PageShell width=PageWidth::Wide>
            <PageHeader
                eyebrow="Developer reference"
                title="Design system"
                meta=|| view! {
                    <span>"Direction C \u{201c}Survey\u{201d}"</span>
                    <span class="text-rule" aria-hidden="true">"/"</span>
                    <span>"Tokens and primitives, batches 1 to 3"</span>
                    <span class="text-rule" aria-hidden="true">"/"</span>
                    <span>"All content on this page is fixture text"</span>
                }
                actions=|| view! { <Button href="/admin/invites" variant=ButtonVariant::Quiet>"Invites"</Button> }
            />
            <nav aria-label="Gallery sections" class="mb-2 flex flex-wrap gap-x-5 gap-y-2 text-label">
                {SECTIONS
                    .iter()
                    .map(|(id, title)| view! {
                        <a href=format!("#{id}") class="text-muted underline decoration-rule underline-offset-4 hover:text-primary hover:decoration-accent">{*title}</a>
                    })
                    .collect_view()}
            </nav>
            <TokensSection />
            <ContrastSection />
            <TypeSection />
            <SpaceSection />
            <LayoutSection />
            <TextSection />
            <ButtonSection />
            <FieldsSection />
            <BadgeSection />
            <TableSection />
            <SortSection />
            <TabsSection />
            <TogglesSection />
            <KeycapSection />
            <CalloutSection />
            <ToastSection />
            <EmptySection />
            <SkeletonSection />
            <TooltipSection />
            <DialogSection />
            <MenuSection />
            <CommandSection />
            <AvatarSection />
            <MarkersSection />
            <ReadoutSection />
            <BarSection />
            <DraftSection />
            <PrepSection />
            <LegendSection />
            <CanvasSection />
            <TimelineSection />
        </PageShell>
    }
}

#[component]
fn TokensSection() -> impl IntoView {
    let groups = [
        TokenGroup::Ground,
        TokenGroup::Line,
        TokenGroup::Text,
        TokenGroup::Accent,
        TokenGroup::Domain,
        TokenGroup::Status,
        TokenGroup::Terrain,
    ];
    view! {
        <GallerySection id="tokens" title="Colour tokens" lede="Every colour token from input.css, in OKLCH, with its role. Terrain tokens are for canvases only.">
            <div class="flex flex-col gap-8">
                {groups
                    .into_iter()
                    .map(|g| view! {
                        <div>
                            <p class="eyebrow mb-3">{g.label()}</p>
                            <ul class="grid grid-cols-1 gap-x-6 gap-y-4 sm:grid-cols-2 lg:grid-cols-4">
                                {TOKENS
                                    .iter()
                                    .filter(|t| t.group == g)
                                    .map(|t| view! {
                                        <li class="grid grid-cols-[2.25rem_minmax(0,1fr)] gap-x-3">
                                            <span
                                                class="row-span-3 size-9 border border-rule"
                                                style=format!("background: {};", css_value(t))
                                                aria-hidden="true"
                                            ></span>
                                            <span class="text-label font-semibold text-primary">{t.name}</span>
                                            <code class="figures text-caption text-muted">{css_value(t)}</code>
                                            <span class="text-caption text-secondary">{t.role}</span>
                                        </li>
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    })
                    .collect_view()}
            </div>
        </GallerySection>
    }
}

#[component]
fn ContrastSection() -> impl IntoView {
    view! {
        <GallerySection id="contrast" title="Contrast" lede="Measured WCAG 2 ratios, computed from the tokens on this page. 4.5 for text, 3.0 for control edges, focus and markers. The full enforced list is in DESIGN.md.">
            <Table caption="Contrast pairs" caption_hidden=true>
                <thead>
                    <tr>
                        <Th>"Sample"</Th>
                        <Th>"Foreground"</Th>
                        <Th>"Background"</Th>
                        <Th>"Use"</Th>
                        <Th numeric=true>"Ratio"</Th>
                        <Th numeric=true>"Minimum"</Th>
                        <Th>"Result"</Th>
                    </tr>
                </thead>
                <tbody>
                    {GALLERY_PAIRS
                        .iter()
                        .map(|p| {
                            let ratio = contrast_by_name(p.fg, p.bg).unwrap_or(0.0);
                            let fg = TOKENS.iter().find(|t| t.name == p.fg).map(css_value).unwrap_or_default();
                            let bg = TOKENS.iter().find(|t| t.name == p.bg).map(css_value).unwrap_or_default();
                            let pass = ratio + 1e-9 >= p.min;
                            view! {
                                <Tr>
                                    <Td>
                                        <span
                                            class="inline-grid h-8 w-16 place-items-center border border-rule text-body font-semibold"
                                            style=format!("color: {fg}; background: {bg};")
                                        >
                                            "Aa"
                                        </span>
                                    </Td>
                                    <Td><code class="figures text-label">{p.fg}</code></Td>
                                    <Td><code class="figures text-label">{p.bg}</code></Td>
                                    <Td muted=true>{p.what}</Td>
                                    <Td numeric=true>{format!("{ratio:.2}")}</Td>
                                    <Td numeric=true muted=true>{format!("{:.1}", p.min)}</Td>
                                    <Td>
                                        {if pass {
                                            view! { <Badge tone=BadgeTone::Success>"Pass"</Badge> }.into_any()
                                        } else {
                                            view! { <Badge tone=BadgeTone::Danger>"Fail"</Badge> }.into_any()
                                        }}
                                    </Td>
                                </Tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </Table>
        </GallerySection>
    }
}

#[component]
fn TypeSection() -> impl IntoView {
    let rows: [(TextRole, &str, &str); 12] = [
        (TextRole::Display, "Display \u{00b7} Familjen Grotesk 44/1.05", "Scrim vs Beta Dragons"),
        (TextRole::Page, "Page \u{00b7} Familjen Grotesk 32/1.1", "Blue side plan"),
        (TextRole::Title, "Title \u{00b7} Familjen Grotesk 24/1.2", "Opponent dossier"),
        (TextRole::Heading, "Heading \u{00b7} Familjen Grotesk 20/1.3", "Draft plans"),
        (TextRole::Reading, "Reading \u{00b7} Public Sans 16/1.6", "Aggressive early and poke-heavy through mid. They group at 12 for grubs, ban Yasuo and Zed, and put Azir first."),
        (TextRole::Body, "Body \u{00b7} Public Sans 14/1.5", "Game plan drafted, 2 open points, 3 open action items."),
        (TextRole::Label, "Label \u{00b7} Public Sans 13 medium", "Selecting for Blue Support"),
        (TextRole::Caption, "Caption \u{00b7} Public Sans 12/1.4", "Last surveyed 3 days ago \u{00b7} evidence from their last 5 games"),
        (TextRole::Overline, "Overline \u{00b7} JetBrains Mono 12 caps", "Opponent dossier"),
        (TextRole::Figure, "Figure \u{00b7} JetBrains Mono 14 tabular", "17:05 \u{00b7} sync 41 / 25 \u{00b7} 9,400 u \u{00b7} 120 \u{20ac}"),
        (TextRole::FigureLarge, "Figure large \u{00b7} JetBrains Mono 24", "84"),
        (TextRole::Note, "Note \u{00b7} Source Serif 4 italic (annotations only)", "Top pushes alone while 4 sit bot side; Night Owls collapse with 3."),
    ];
    view! {
        <GallerySection id="type" title="Type" lede="Three faces plus the serif for surveyor's notes. Nothing below 12px. Figures are tabular everywhere.">
            <dl class="flex flex-col">
                {rows
                    .into_iter()
                    .map(|(role, spec, sample)| view! {
                        <div class="grid grid-cols-1 gap-2 border-t border-rule py-4 sm:grid-cols-[16rem_minmax(0,1fr)] sm:gap-6">
                            <dt class="text-caption text-muted">{spec}</dt>
                            <dd><Text role=role tone=Tone::Primary>{sample}</Text></dd>
                        </div>
                    })
                    .collect_view()}
            </dl>
        </GallerySection>
    }
}

#[component]
fn SpaceSection() -> impl IntoView {
    let steps = [4u32, 8, 12, 16, 24, 32, 48, 64];
    view! {
        <GallerySection id="space" title="Space, radius, motion" lede="A 4px spacing scale; radius 0 for structure, 3px for controls, round only for avatars and toggles; motion between 100 and 300ms, ease-out, nothing loops.">
            <div class="grid gap-8 lg:grid-cols-3">
                <div>
                    <p class="eyebrow mb-3">"Spacing"</p>
                    <ul class="flex flex-col gap-2">
                        {steps
                            .into_iter()
                            .map(|s| view! {
                                <li class="grid grid-cols-[3rem_minmax(0,1fr)] items-center gap-3">
                                    <span class="figures text-caption text-muted">{format!("{s}px")}</span>
                                    <span class="h-2 bg-secondary" style=format!("width: {s}px;")></span>
                                </li>
                            })
                            .collect_view()}
                    </ul>
                </div>
                <div>
                    <p class="eyebrow mb-3">"Radius"</p>
                    <div class="flex flex-wrap items-end gap-6">
                        <Sample label="0 \u{00b7} structure"><span class="block size-14 border border-outline bg-ink-1"></span></Sample>
                        <Sample label="3px \u{00b7} controls"><span class="block size-14 rounded-ctl border border-outline bg-ink-2"></span></Sample>
                        <Sample label="round \u{00b7} avatars"><UserAvatar name="Round" size=AvatarSize::Xl /></Sample>
                    </div>
                </div>
                <div>
                    <p class="eyebrow mb-3">"Motion"</p>
                    <Table caption="Motion tokens" caption_hidden=true>
                        <tbody>
                            <Tr><Td header=true>"--dur-1"</Td><Td numeric=true>"100ms"</Td><Td muted=true>"press, colour"</Td></Tr>
                            <Tr><Td header=true>"--dur-2"</Td><Td numeric=true>"160ms"</Td><Td muted=true>"hover, toggle"</Td></Tr>
                            <Tr><Td header=true>"--dur-3"</Td><Td numeric=true>"220ms"</Td><Td muted=true>"popover, tray"</Td></Tr>
                            <Tr><Td header=true>"--dur-4"</Td><Td numeric=true>"300ms"</Td><Td muted=true>"panel, route"</Td></Tr>
                        </tbody>
                    </Table>
                    <p class="mt-3 text-caption text-muted">"Ease-out cubic-bezier(0.23, 1, 0.32, 1). Reduced motion: instant."</p>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
fn LayoutSection() -> impl IntoView {
    let layers = [
        (Layer::Page, "Page"),
        (Layer::Ink1, "Ink 1"),
        (Layer::Ink2, "Ink 2"),
        (Layer::Ink3, "Ink 3"),
        (Layer::Canvas, "Canvas"),
    ];
    let edges = [
        (Edge::None, "None"),
        (Edge::RuleTop, "Rule top"),
        (Edge::RuleY, "Rule top and bottom"),
        (Edge::Rule, "Rule box"),
        (Edge::Object, "Object (actionable)"),
    ];
    view! {
        <GallerySection id="layout" title="Layout" lede="PageShell and PageHeader frame this page (its title is the page's h1). SectionHeader gives every section a real heading. Surface draws nothing unless asked: layers by tone, edges by meaning.">
            <div class="flex flex-col gap-8">
                <div>
                    <p class="eyebrow mb-3">"SectionHeader"</p>
                    <Surface layer=Layer::Ink1 pad=Pad::Lg>
                        <SectionHeader
                            title="Opponent dossier"
                            level=HeadingLevel::H3
                            eyebrow="Level 3 with eyebrow, meta and actions"
                            meta="Last surveyed 3 days ago"
                            actions=|| view! { <Button size=ButtonSize::Sm>"Open dossier"</Button> }
                        />
                        <Text role=TextRole::Body tone=Tone::Secondary tag=crate::components::ui::TextTag::P>
                            "Section content follows the heading without a box around it."
                        </Text>
                    </Surface>
                </div>
                <div>
                    <p class="eyebrow mb-3">"Surface layers"</p>
                    <div class="grid grid-cols-2 gap-3 sm:grid-cols-5">
                        {layers
                            .into_iter()
                            .map(|(l, name)| view! {
                                <Surface layer=l edge=Edge::Rule pad=Pad::Md class="h-20">
                                    <span class="text-label text-primary">{name}</span>
                                </Surface>
                            })
                            .collect_view()}
                    </div>
                </div>
                <div>
                    <p class="eyebrow mb-3">"Surface edges"</p>
                    <div class="grid grid-cols-1 gap-3 sm:grid-cols-5">
                        {edges
                            .into_iter()
                            .map(|(e, name)| view! {
                                <Surface layer=Layer::Ink1 edge=e pad=Pad::Md class="h-20">
                                    <span class="text-label text-primary">{name}</span>
                                </Surface>
                            })
                            .collect_view()}
                    </div>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
fn TextSection() -> impl IntoView {
    let tones = [
        (Tone::Primary, "Primary"),
        (Tone::Secondary, "Secondary"),
        (Tone::Muted, "Muted"),
        (Tone::Accent, "Accent"),
        (Tone::Success, "Success"),
        (Tone::Warning, "Warning"),
        (Tone::Danger, "Danger"),
        (Tone::Info, "Info"),
    ];
    view! {
        <GallerySection id="text" title="Text" lede="Text roles come from the type scale (see Type); tones are the text colours. Every tone reads at 4.5:1 or better on every layer.">
            <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
                {tones
                    .into_iter()
                    .map(|(t, name)| view! {
                        <Surface layer=Layer::Ink2 pad=Pad::Md>
                            <Text role=TextRole::Label tone=t>{name}</Text>
                        </Surface>
                    })
                    .collect_view()}
            </div>
        </GallerySection>
    }
}

fn plus_icon() -> impl IntoView {
    view! {
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75">
            <path d="M8 3v10M3 8h10" stroke-linecap="round" />
        </svg>
    }
}

#[component]
fn ButtonSection() -> impl IntoView {
    let variants = [
        (ButtonVariant::Primary, "Primary"),
        (ButtonVariant::Secondary, "Secondary"),
        (ButtonVariant::Quiet, "Quiet"),
        (ButtonVariant::Danger, "Danger"),
    ];
    let clicks = RwSignal::new(0u32);
    let count = Callback::new(move |_| clicks.update(|c| *c += 1));
    view! {
        <GallerySection id="button" title="Button" lede="Four variants, three sizes. Primary is the one next thing to do per view. Hover, focus and pressed are forced here; loading disables and sets aria-busy.">
            <div class="flex flex-col gap-8">
                <div class="overflow-x-auto p-1">
                    <div class="grid min-w-[44rem] grid-cols-[7rem_repeat(6,minmax(0,1fr))] items-center gap-x-4 gap-y-5">
                        <span></span>
                        <span class="eyebrow">"Default"</span>
                        <span class="eyebrow">"Hover"</span>
                        <span class="eyebrow">"Focus"</span>
                        <span class="eyebrow">"Pressed"</span>
                        <span class="eyebrow">"Disabled"</span>
                        <span class="eyebrow">"Loading"</span>
                        {variants
                            .into_iter()
                            .map(|(v, name)| view! {
                                <span class="text-label text-secondary">{name}</span>
                                <div><Button variant=v>"Save plan"</Button></div>
                                <div><Button variant=v force=Force::Hover>"Save plan"</Button></div>
                                <div><Button variant=v force=Force::Focus>"Save plan"</Button></div>
                                <div><Button variant=v force=Force::Active>"Save plan"</Button></div>
                                <div><Button variant=v disabled=true>"Save plan"</Button></div>
                                <div><Button variant=v loading=true>"Saving"</Button></div>
                            })
                            .collect_view()}
                    </div>
                </div>
                <div class="flex flex-wrap items-end gap-6">
                    <Sample label="Size Sm (28px)"><Button size=ButtonSize::Sm>"Compare"</Button></Sample>
                    <Sample label="Size Md (32px)"><Button size=ButtonSize::Md>"Compare"</Button></Sample>
                    <Sample label="Size Lg (40px)"><Button size=ButtonSize::Lg variant=ButtonVariant::Primary>"Sign in"</Button></Sample>
                    <Sample label="Size Touch (44px on phones, 32px from md)"><Button size=ButtonSize::Touch>"Change password"</Button></Sample>
                    <Sample label="Icon slot"><Button icon=plus_icon>"Add plan"</Button></Sample>
                    <Sample label="Keycap hint"><Button variant=ButtonVariant::Primary shortcut="P">"Pick Rakan"</Button></Sample>
                    <Sample label="Icon only, labelled"><Button variant=ButtonVariant::Quiet aria_label="Add plan" icon=plus_icon>""</Button></Sample>
                    <Sample label="As a link"><Button href="#button">"Open brief"</Button></Sample>
                </div>
                <div class="flex flex-col gap-2">
                    <span class="eyebrow">"A button keeps its size: idle, loading, done"</span>
                    <div class="flex flex-wrap items-center gap-4" data-testid="button-size-demo">
                        <Button variant=ButtonVariant::Primary reserve=&["Sync my games", "Synced"] attr:data-testid="button-size-idle">"Sync my games"</Button>
                        <Button variant=ButtonVariant::Primary reserve=&["Sync my games", "Synced"] loading=true attr:data-testid="button-size-loading">"Sync my games"</Button>
                        <Button variant=ButtonVariant::Primary reserve=&["Sync my games", "Synced"] attr:data-testid="button-size-done">"Synced"</Button>
                    </div>
                    <p class="text-caption text-muted">"The spinner overlays the label; "<code class="figures">"reserve"</code>" lists every label, so all three share the widest box."</p>
                </div>
                <div class="flex flex-wrap items-center gap-4" data-testid="button-demo">
                    <Button variant=ButtonVariant::Secondary on_click=count kind=ButtonType::Button>"Count a click"</Button>
                    <output class="figures text-label text-secondary" data-testid="button-count" aria-live="polite">
                        {move || format!("Clicked {} times", clicks.get())}
                    </output>
                    <span class="text-caption text-muted">"Keycaps: " <Keycap keys="Ctrl" /> " " <Keycap keys="K" /></span>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
fn FieldsSection() -> impl IntoView {
    let name = RwSignal::new(String::new());
    let role = RwSignal::new("mid".to_string());
    let roles = vec![
        SelectOption::new("top", "Top"),
        SelectOption::new("jungle", "Jungle"),
        SelectOption::new("mid", "Mid"),
        SelectOption::new("bot", "Bot"),
        SelectOption::new("support", "Support"),
    ];
    let roles_a = roles.clone();
    let roles_b = roles.clone();
    let error = Signal::derive(|| Some("Choose an opponent from the list.".to_string()));
    view! {
        <GallerySection id="fields" title="Fields" lede="Field owns the label, help and error and wires them to the control. The edge is the 3:1 outline; focus is the chalk outline; errors are announced.">
            <div class="grid grid-cols-1 gap-x-8 gap-y-6 md:grid-cols-3">
                <Field id="g-default" label="Opponent" hint="The team you are preparing for.">
                    <TextField placeholder="Beta Dragons" />
                </Field>
                <Field id="g-hover" label="Hover (forced)">
                    <TextField placeholder="Beta Dragons" force=Force::Hover />
                </Field>
                <Field id="g-focus" label="Focus (forced)">
                    <TextField placeholder="Beta Dragons" force=Force::Focus />
                </Field>
                <Field id="g-disabled" label="Disabled">
                    <TextField placeholder="Locked by the coach" disabled=true />
                </Field>
                <Field id="g-error" label="Opponent" error=error required=true>
                    <TextField placeholder="Beta Dragons" />
                </Field>
                <Field id="g-small" label="Small (filters)">
                    <TextField kind=InputKind::Search size=FieldSize::Sm placeholder="Search champions" />
                </Field>
                <Field id="g-area" label="Game plan note" hint="Multi-line: rows set the height.">
                    <TextField rows=3 placeholder="Call the reset at 16:30 when grubs are down." />
                </Field>
                <Field id="g-select" label="Role">
                    <Select options=roles_a />
                </Field>
                <Field id="g-select-disabled" label="Role (disabled)">
                    <Select options=roles_b disabled=true />
                </Field>
            </div>
            <div class="mt-8 grid grid-cols-1 gap-6 border-t border-rule pt-6 md:grid-cols-2" data-testid="field-demo">
                <div class="flex flex-col gap-2">
                    <Field id="g-live-name" label="Player name (live)">
                        <TextField value=name placeholder="Type a name" />
                    </Field>
                    <output class="text-label text-secondary" data-testid="field-echo">
                        {move || if name.get().is_empty() { "Nothing typed yet".to_string() } else { format!("You typed: {}", name.get()) }}
                    </output>
                </div>
                <div class="flex flex-col gap-2">
                    <Field id="g-live-role" label="Role (live)">
                        <Select value=role options=roles />
                    </Field>
                    <output class="text-label text-secondary" data-testid="select-echo">
                        {move || format!("Selected: {}", role.get())}
                    </output>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
fn BadgeSection() -> impl IntoView {
    view! {
        <GallerySection id="badge" title="Badge" lede="Short labels. Status, side and result tones carry a shape or a letter as well as colour; side colours are never reused for results.">
            <div class="flex flex-col gap-6">
                <div class="flex flex-wrap items-center gap-3">
                    <Badge>"Draft"</Badge>
                    <Badge tone=BadgeTone::Accent>"Selecting"</Badge>
                    <Badge tone=BadgeTone::Success>"Ready"</Badge>
                    <Badge tone=BadgeTone::Warning>"2 open points"</Badge>
                    <Badge tone=BadgeTone::Danger>"Sync failed"</Badge>
                    <Badge tone=BadgeTone::Info>"Patch 16.20"</Badge>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <Badge tone=BadgeTone::SideBlue>"Blue side"</Badge>
                    <Badge tone=BadgeTone::SideRed>"Red side"</Badge>
                    <span class="text-caption text-muted">"Last 5, oldest first:"</span>
                    <Badge tone=BadgeTone::Win label="Win">"W"</Badge>
                    <Badge tone=BadgeTone::Loss label="Loss">"L"</Badge>
                    <Badge tone=BadgeTone::Win label="Win">"W"</Badge>
                    <Badge tone=BadgeTone::Win label="Win">"W"</Badge>
                    <Badge tone=BadgeTone::Loss label="Loss">"L"</Badge>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
fn TableSection() -> impl IntoView {
    let roster = [
        ("Nordwind", "Top", "Ornn, Gnar, K'Sante", 42, 57.1),
        ("Hask", "Jungle", "Jarvan IV, Vi, Sejuani", 39, 51.3),
        ("Lio", "Mid", "Orianna, Azir, Syndra", 44, 54.5),
        ("Jinxie", "Bot", "Jinx, Kai'Sa, Ezreal", 41, 48.8),
        ("Pell", "Support", "Lulu, Rakan, Nautilus", 40, 60.0),
    ];
    view! {
        <GallerySection id="table" title="Table" lede="Dense data: hairline rows, tabular figures right-aligned, a caption, sortable header state and row links. Compact by default; sticky header when the body scrolls.">
            <div class="flex flex-col gap-8">
                <Table caption="Roster: Alpha Wolves (fixture)">
                    <thead>
                        <tr>
                            <Th>"Player"</Th>
                            <Th>"Role"</Th>
                            <Th>"Pool"</Th>
                            <Th numeric=true sort=crate::components::ui::SortDir::Descending>"Games"</Th>
                            <Th numeric=true>"Win rate"</Th>
                        </tr>
                    </thead>
                    <tbody>
                        {roster
                            .into_iter()
                            .map(|(p, r, pool, games, wr)| view! {
                                <Tr>
                                    <Td header=true><RowLink href="#table">{p}</RowLink></Td>
                                    <Td muted=true>{r}</Td>
                                    <Td>{pool}</Td>
                                    <Td numeric=true>{games}</Td>
                                    <Td numeric=true>{format!("{wr:.1}%")}</Td>
                                </Tr>
                            })
                            .collect_view()}
                    </tbody>
                </Table>
                <Table caption="Last results, comfortable density (fixture)" density=crate::components::ui::Density::Comfortable sticky_header=true>
                    <thead>
                        <tr>
                            <Th>"Date"</Th>
                            <Th>"Opponent"</Th>
                            <Th>"Result"</Th>
                            <Th numeric=true>"Duration"</Th>
                        </tr>
                    </thead>
                    <tbody>
                        <Tr><Td muted=true>"8 Oct"</Td><Td>"Night Owls"</Td><Td><Badge tone=BadgeTone::Loss label="Loss">"L"</Badge></Td><Td numeric=true>"31:12"</Td></Tr>
                        <Tr selected=true><Td muted=true>"5 Oct"</Td><Td>"Beta Dragons"</Td><Td><Badge tone=BadgeTone::Win label="Win">"W"</Badge></Td><Td numeric=true>"27:40"</Td></Tr>
                        <Tr><Td muted=true>"1 Oct"</Td><Td>"Gamma Foxes"</Td><Td><Badge tone=BadgeTone::Win label="Win">"W"</Badge></Td><Td numeric=true>"34:05"</Td></Tr>
                    </tbody>
                </Table>
            </div>
        </GallerySection>
    }
}

#[component]
fn SortSection() -> impl IntoView {
    use crate::components::ui::{sort_rows, SortDir, SortState, SortStatus, SortTh};
    const LABELS: &[(&str, &str)] = &[
        ("champion", "Champion"),
        ("games", "Games"),
        ("winrate", "Win rate"),
    ];
    let rows: Vec<(&'static str, i32, f32)> = vec![
        ("Orianna", 14, 57.1),
        ("Azir", 9, 44.4),
        ("Syndra", 14, 64.3),
        ("Taliyah", 3, 66.7),
    ];
    let sort = RwSignal::new(SortState::by("games", SortDir::Descending));
    view! {
        <GallerySection id="sort" title="Sortable table" lede="SortTh: a click, Enter or Space sorts by the column; again flips it. Numbers start descending, words ascending; ties keep their order. The header names the direction with aria-sort and a chevron, a status line announces it.">
            <div data-testid="sort-demo">
                <SortStatus sort=sort labels=LABELS />
                <Table caption="Champion trends (fixture)" caption_hidden=true>
                    <thead>
                        <tr>
                            <SortTh label="Champion" key="champion" sort=sort />
                            <SortTh label="Games" key="games" sort=sort numeric=true />
                            <SortTh label="Win rate" key="winrate" sort=sort numeric=true />
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let mut list = rows.clone();
                            let st = sort.get();
                            if let Some(key) = st.key {
                                sort_rows(&mut list, st.dir, |a, b| match key {
                                    "champion" => a.0.cmp(b.0),
                                    "games" => a.1.cmp(&b.1),
                                    _ => a.2.total_cmp(&b.2),
                                });
                            }
                            list.into_iter().map(|(c, g, w)| view! {
                                <Tr>
                                    <Td header=true>{c}</Td>
                                    <Td numeric=true>{g}</Td>
                                    <Td numeric=true>{format!("{w:.1}%")}</Td>
                                </Tr>
                            }).collect_view()
                        }}
                    </tbody>
                </Table>
            </div>
        </GallerySection>
    }
}

#[component]
fn TabsSection() -> impl IntoView {
    let selected = RwSignal::new("match-day".to_string());
    let tabs = vec![
        TabItem::new("match-day", "Match Day"),
        TabItem::new("draft", "Draft + Graph").with_count(3),
        TabItem::new("sand-table", "Sand Table"),
    ];
    view! {
        <GallerySection id="tabs" title="Tabs" lede="Views of the same object. Left and Right move (wrapping), Home and End jump; only the selected tab is in the tab order.">
            <div data-testid="tabs-demo">
                <Tabs group="gallery-tabs" label="Screens" tabs=tabs selected=selected />
                <TabPanel group="gallery-tabs" tab="match-day" selected=selected>
                    <p class="text-body text-secondary">"Match Day: the brief, the draft plans, the game plan and open items, in order."</p>
                </TabPanel>
                <TabPanel group="gallery-tabs" tab="draft" selected=selected>
                    <p class="text-body text-secondary">"Draft + Graph: the board, the graph in the centre and the ranked reasons."</p>
                </TabPanel>
                <TabPanel group="gallery-tabs" tab="sand-table" selected=selected>
                    <p class="text-body text-secondary">"Sand Table: the replay map, the moments rail and the scrubber."</p>
                </TabPanel>
            </div>
        </GallerySection>
    }
}

/// The worked tournament draft: Alpha Wolves on blue, step 19 (B5) open.
fn example_draft() -> (Vec<Option<String>>, Vec<Option<String>>) {
    let picks: [(&str, Option<&str>); 20] = [
        ("Yasuo", None),
        ("Orianna", None),
        ("Zed", None),
        ("Jarvan IV", None),
        ("Akali", None),
        ("Lulu", None),
        ("Azir", Some("Mid")),
        ("Lee Sin", Some("Jungle")),
        ("Syndra", Some("Mid")),
        ("Jinx", Some("Bot")),
        ("Vi", Some("Jungle")),
        ("Camille", Some("Top")),
        ("Kalista", None),
        ("Leona", None),
        ("Viego", None),
        ("Taliyah", None),
        ("Kai'Sa", Some("Bot")),
        ("Ornn", Some("Top")),
        ("", Some("Support")),
        ("", Some("Support")),
    ];
    let champions = picks
        .iter()
        .map(|(c, _)| (!c.is_empty()).then(|| c.to_string()))
        .collect();
    let roles = picks.iter().map(|(_, r)| r.map(str::to_string)).collect();
    (champions, roles)
}

#[component]
fn DraftSection() -> impl IntoView {
    let (champions, roles) = example_draft();
    // Mid-draft: step 19 (Blue pick 5) is being decided.
    let mid: Vec<Option<String>> = champions.iter().take(18).cloned().collect();
    let mid = RwSignal::new(mid);
    let current = RwSignal::new(Some(18_usize));
    let chosen = RwSignal::new(None::<usize>);
    let roles_mid = roles.clone();
    // Complete: the same draft with blue pick 5 and red pick 5 locked in.
    let mut done = champions.clone();
    done[18] = Some("Rakan".to_string());
    done[19] = Some("Nautilus".to_string());
    let mut notes = vec![None; 20];
    notes[6] = Some("Blind-safe mid, flex".to_string());
    notes[18] = Some("Engage for Jinx".to_string());
    view! {
        <GallerySection id="draft" title="Tournament draft" lede="The compact board the draft page uses: blue left, red right, each column is that team's final draft. Five bans per side, grouped by ban rotation (3, then 2), then five picks in pick order, bracketed by pick rotation: B1 · R1 R2 · B2 B3 · R3 · R4 · B4 B5 · R5. The step being decided is the only lit slot. Fixture: Alpha Wolves on blue.">
            <div class="grid grid-cols-1 gap-10 lg:grid-cols-2">
                <div class="min-w-0">
                    <p class="eyebrow mb-3">"Mid-draft (interactive)"</p>
                    <DraftBoard
                        id="gallery-draft"
                        label="Example draft board, mid-draft"
                        champions=Signal::derive(move || mid.get())
                        roles=Signal::stored(roles_mid)
                        current=Signal::derive(move || current.get())
                        blue_team=Signal::stored(Some("Alpha Wolves".to_string()))
                        red_team=Signal::stored(Some("Beta Dragons".to_string()))
                        on_select=Callback::new(move |i: usize| {
                            current.set(Some(i));
                            chosen.set(Some(i));
                        })
                        on_clear=Callback::new(move |i: usize| mid.update(|v| {
                            if let Some(s) = v.get_mut(i) {
                                *s = None;
                            }
                        }))
                        hints=true
                    />
                    <p class="mt-3 px-5 text-caption text-muted" aria-live="polite" data-gallery-draft-chosen="">
                        {move || match chosen.get() {
                            Some(i) => format!("Chosen: {}", TOURNAMENT_ORDER[i].describe()),
                            None => "Click a slot or press Enter on one.".to_string(),
                        }}
                    </p>
                </div>
                <div class="min-w-0">
                    <p class="eyebrow mb-3">"Complete (read-only)"</p>
                    <DraftBoard
                        id="gallery-draft-done"
                        label="Example draft board, complete"
                        champions=Signal::stored(done)
                        roles=Signal::stored(roles)
                        notes=Signal::stored(notes)
                        blue_team=Signal::stored(Some("Alpha Wolves".to_string()))
                        red_team=Signal::stored(Some("Beta Dragons".to_string()))
                    />
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
fn PrepSection() -> impl IntoView {
    let steps = vec![
        PrepStep::new("Read the opponent brief", PrepState::Done)
            .detail("Beta Dragons, last surveyed 3 days ago")
            .status("Ready")
            .link("#prep", "Open the brief"),
        PrepStep::new("Finish the draft plans", PrepState::Next)
            .detail("Plan 3 (red side) is not started")
            .status("2 of 3")
            .link("#prep", "Open draft plan 3"),
        PrepStep::new("Settle the game plan", PrepState::Attention)
            .detail("Drafted; 2 open points")
            .status("2 open points")
            .link("#prep", "Open the game plan"),
        PrepStep::new("Close open action items", PrepState::Todo)
            .detail("Hask: first clear vs Lee Sin \u{00b7} Jinxie: Kai'Sa key mapping \u{00b7} Team: dragon setup 60 s earlier")
            .status("3 open")
            .link("#prep", "Open action items"),
    ];
    let quiet = steps.clone();
    view! {
        <GallerySection id="prep" title="Prep sequence" lede="The Match Day pattern: what to do next, in order. Done steps recede, the next step is emphasised and links to where it is done, steps that need attention say why. Quiet emphasis outlines the next step for views that have their own primary action (the game plan's Save plan).">
            <div class="grid max-w-5xl gap-10 lg:grid-cols-2">
                <div>
                    <p class="eyebrow mb-2">"Primary (Match Day) \u{00b7} scrim vs Beta Dragons"</p>
                    <PrepSequence label="Preparation steps" steps=steps />
                </div>
                <div>
                    <p class="eyebrow mb-2">"Quiet (inside an editor)"</p>
                    <PrepSequence label="Preparation steps, quiet" steps=quiet emphasis=PrepEmphasis::Quiet />
                </div>
            </div>
        </GallerySection>
    }
}

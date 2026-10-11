//! `/design` gallery sections for batch 3 (domain primitives and the
//! reserved canvas pieces). All content is fixture text.

use leptos::prelude::*;

use crate::components::ui::{
    Annotation, AvatarShape, AvatarSize, AvatarState, CanvasFrame, ChampionAvatar, Delta,
    DraftSide, FrameGrid, GameResult, GlyphSize, LegendBlock, LegendEntry, LineStyle, LineWeight,
    Moment, NotePlacement, ReadoutSize, ResultGlyph, Role, RoleLabel, RoleLabelStyle, ScaleBar,
    Sentiment, SideMarker, SideText, StatReadout, Swatch, SwatchInk, TimelineScrub, Trend,
    UserAvatar,
};
use crate::pages::design::{GallerySection, Sample};

#[component]
pub fn AvatarSection() -> impl IntoView {
    let sizes = [
        (AvatarSize::Xs, "Xs 20"),
        (AvatarSize::Sm, "Sm 24"),
        (AvatarSize::Md, "Md 32"),
        (AvatarSize::Lg, "Lg 40"),
        (AvatarSize::Xl, "Xl 56"),
    ];
    view! {
        <GallerySection id="avatar" title="Champion avatar" lede="Square icons from the app's own image endpoint (/img/champion/{name}/icon), never a third party. A two-letter monogram shows while the image loads and stays when it is missing. Sides add shape: blue round, red square.">
            <div class="flex flex-col gap-8">
                <div class="flex flex-wrap items-end gap-6">
                    {sizes
                        .into_iter()
                        .map(|(s, l)| view! { <Sample label=l><ChampionAvatar name="Azir" size=s /></Sample> })
                        .collect_view()}
                </div>
                <div class="flex flex-wrap items-end gap-6">
                    <Sample label="Square (lists)"><ChampionAvatar name="Orianna" size=AvatarSize::Lg /></Sample>
                    <Sample label="Round (tokens)"><ChampionAvatar name="Orianna" size=AvatarSize::Lg shape=AvatarShape::Round /></Sample>
                    <Sample label="Blue side"><ChampionAvatar name="Jinx" size=AvatarSize::Lg side=DraftSide::Blue /></Sample>
                    <Sample label="Red side"><ChampionAvatar name="Kai'Sa" size=AvatarSize::Lg side=DraftSide::Red /></Sample>
                    <Sample label="Banned"><ChampionAvatar name="Yasuo" size=AvatarSize::Lg state=AvatarState::Banned /></Sample>
                    <Sample label="Dimmed"><ChampionAvatar name="Zed" size=AvatarSize::Lg state=AvatarState::Dimmed /></Sample>
                    <Sample label="Missing image: monogram"><ChampionAvatar name="Not A Champion" size=AvatarSize::Lg /></Sample>
                    <Sample label="User avatar"><UserAvatar name="Linnea Voss" size=AvatarSize::Lg label="Linnea Voss" /></Sample>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn MarkersSection() -> impl IntoView {
    view! {
        <GallerySection id="markers" title="Role, side and result marks" lede="Roles are typographic (TOP JGL MID BOT SUP), read out as words. Sides are a blue circle and a red square in matched colours. Results are a filled teal W and a hollow L, never the side colours.">
            <div class="grid grid-cols-1 gap-8 md:grid-cols-3">
                <div class="flex flex-col gap-4">
                    <p class="eyebrow">"RoleLabel"</p>
                    <div class="flex flex-wrap items-center gap-4">
                        {Role::ALL.into_iter().map(|r| view! { <RoleLabel role=r /> }).collect_view()}
                    </div>
                    <div class="flex flex-wrap items-center gap-2">
                        {Role::ALL.into_iter().map(|r| view! { <RoleLabel role=r style=RoleLabelStyle::Boxed /> }).collect_view()}
                    </div>
                    <div class="flex items-center gap-2 text-label text-secondary">
                        <RoleLabel role=Role::Support style=RoleLabelStyle::Active />
                        "Selecting for blue support"
                    </div>
                </div>
                <div class="flex flex-col gap-4">
                    <p class="eyebrow">"SideMarker"</p>
                    <div class="flex flex-wrap items-center gap-5">
                        <SideMarker side=DraftSide::Blue />
                        <SideMarker side=DraftSide::Red />
                        <SideMarker side=DraftSide::Blue text=SideText::Short />
                        <SideMarker side=DraftSide::Red text=SideText::Short />
                    </div>
                    <div class="flex flex-wrap items-center gap-5">
                        <SideMarker side=DraftSide::Blue text=SideText::Long />
                        <SideMarker side=DraftSide::Red text=SideText::Long />
                    </div>
                </div>
                <div class="flex flex-col gap-4">
                    <p class="eyebrow">"ResultGlyph"</p>
                    <div class="flex flex-wrap items-center gap-2">
                        <span class="text-caption text-muted">"Last 5, oldest first:"</span>
                        <ResultGlyph result=GameResult::Win />
                        <ResultGlyph result=GameResult::Loss />
                        <ResultGlyph result=GameResult::Win />
                        <ResultGlyph result=GameResult::Win />
                        <ResultGlyph result=GameResult::Loss />
                    </div>
                    <div class="flex flex-wrap items-center gap-2">
                        <ResultGlyph result=GameResult::Win size=GlyphSize::Sm />
                        <ResultGlyph result=GameResult::Loss size=GlyphSize::Sm />
                        <ResultGlyph result=GameResult::Win word=true />
                        <ResultGlyph result=GameResult::Loss word=true />
                    </div>
                </div>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn ReadoutSection() -> impl IntoView {
    view! {
        <GallerySection id="readout" title="Stat readout" lede="A number that is the point, in a fixed slot so a row lines up. Changes are written out with their direction; colour follows good or bad (fewer deaths is good), never the sides.">
            <div class="flex flex-col gap-8">
                <div class="grid grid-cols-2 gap-6 sm:grid-cols-4">
                    <StatReadout label="Win rate" value="57.1" unit="%" slots=4 delta=Delta::new("+4.2", Trend::Up, Sentiment::Good).against("vs last 10") />
                    <StatReadout label="Deaths" value="3.4" unit="per game" slots=4 delta=Delta::new("-0.8", Trend::Down, Sentiment::Good) />
                    <StatReadout label="Gold at 15" value="6,120" slots=5 delta=Delta::new("-310", Trend::Down, Sentiment::Bad) />
                    <StatReadout label="Vision" value="1.1" unit="per min" slots=4 delta=Delta::new("0.0", Trend::Flat, Sentiment::Neutral) />
                </div>
                <div class="grid grid-cols-2 gap-6 sm:grid-cols-4">
                    <StatReadout label="Fit" value="84" size=ReadoutSize::Lg />
                    <StatReadout label="Games" value="42" size=ReadoutSize::Sm />
                    <StatReadout label="KDA" value="" empty_reason="No games synced yet" />
                </div>
            </div>
        </GallerySection>
    }
}

fn graph_legend() -> Vec<LegendEntry> {
    vec![
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Solid,
                weight: LineWeight::Medium,
                ink: SwatchInk::Primary,
                arrow: false,
            },
            "Synergy \u{00b7} solid",
        ),
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Dashed,
                weight: LineWeight::Medium,
                ink: SwatchInk::Primary,
                arrow: true,
            },
            "Counters \u{00b7} dashed, arrow to the target",
        ),
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Dotted,
                weight: LineWeight::Medium,
                ink: SwatchInk::Primary,
                arrow: false,
            },
            "Weak against \u{00b7} dotted",
        ),
        LegendEntry::new(Swatch::Widths(SwatchInk::Muted), "Line width = strength"),
        LegendEntry::new(Swatch::Contours, "Contours every 10 fit, from 50"),
        LegendEntry::new(Swatch::Sides, "Blue circle \u{00b7} red square"),
    ]
}

fn map_legend() -> Vec<LegendEntry> {
    vec![
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Solid,
                weight: LineWeight::Thick,
                ink: SwatchInk::SideBlue,
                arrow: false,
            },
            "Known path (blue)",
        ),
        LegendEntry::new(
            Swatch::Line {
                style: LineStyle::Dashed,
                weight: LineWeight::Thick,
                ink: SwatchInk::SideRed,
                arrow: false,
            },
            "Estimated path (red)",
        ),
        LegendEntry::new(
            Swatch::Anchor(SwatchInk::SideBlue),
            "Exact event: solid anchor",
        ),
        LegendEntry::new(
            Swatch::Area {
                soft: true,
                ink: SwatchInk::SideRed,
            },
            "Approximate area: soft edge",
        ),
        LegendEntry::new(
            Swatch::Area {
                soft: false,
                ink: SwatchInk::SideBlue,
            },
            "Known area: solid edge",
        ),
        LegendEntry::new(Swatch::Side(DraftSide::Red), "Red player token"),
    ]
}

#[component]
pub fn LegendSection() -> impl IntoView {
    view! {
        <GallerySection id="legend" title="Legend block" lede="Every canvas that encodes colour, shape, line style or width explains it next to the drawing. Swatches differ by shape and line style, not only by hue, and every entry is words.">
            <div class="grid grid-cols-1 gap-10 md:grid-cols-[18rem_minmax(0,1fr)]">
                <LegendBlock id="gal-graph" title="Draft Graph legend" entries=graph_legend() />
                <LegendBlock id="gal-map" title="Sand Table legend (columns)" entries=map_legend() columns=true />
            </div>
        </GallerySection>
    }
}

/// A champion token placed on a canvas at percent coordinates.
fn token(name: &'static str, side: Option<DraftSide>, x: f32, y: f32) -> impl IntoView {
    view! {
        <div class="absolute -translate-x-1/2 -translate-y-1/2" style=format!("left: {x}%; top: {y}%;")>
            {match side {
                Some(s) => view! { <ChampionAvatar name=name size=AvatarSize::Md side=s /> }.into_any(),
                None => view! { <ChampionAvatar name=name size=AvatarSize::Md shape=AvatarShape::Round /> }.into_any(),
            }}
        </div>
    }
}

#[component]
pub fn CanvasSection() -> impl IntoView {
    view! {
        <GallerySection id="canvas" title="Canvas frame and annotation" lede="Reserved for the Draft Graph and the Sand Table (M4). A coordinate frame with numbered edges or A\u{2013}H by 1\u{2013}5 grid references, a faint grid only inside, a caption, at most one scale bar, and surveyor's notes in serif italic on leader lines. Fixture drawings, no behaviour.">
            <div class="grid grid-cols-1 gap-10 xl:grid-cols-2">
                <CanvasFrame caption="Draft Graph (fixture): support candidates for blue, grid references A\u{2013}H by 1\u{2013}5. Text twin: the ranked list." grid=FrameGrid::Reference { cols: 8, rows: 5 } texture=true>
                    <svg class="absolute inset-0 size-full" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                        <line x1="12" y1="30" x2="44" y2="55" class="stroke-primary" stroke-width="2.5" vector-effect="non-scaling-stroke" />
                        <line x1="12" y1="70" x2="44" y2="55" class="stroke-primary" stroke-width="1.5" vector-effect="non-scaling-stroke" />
                        <line x1="44" y1="55" x2="86" y2="40" class="stroke-secondary" stroke-width="2" stroke-dasharray="7 5" vector-effect="non-scaling-stroke" />
                    </svg>
                    {token("Jinx", Some(DraftSide::Blue), 12.0, 30.0)}
                    {token("Azir", Some(DraftSide::Blue), 12.0, 70.0)}
                    {token("Rakan", None, 44.0, 55.0)}
                    {token("Nautilus", Some(DraftSide::Red), 86.0, 40.0)}
                    <Annotation x=44.0 y=55.0 title="Fit 84" at="D3" placement=NotePlacement::NorthEast>
                        "Engage follows Azir's shuffle; three team notes agree."
                    </Annotation>
                </CanvasFrame>
                <CanvasFrame
                    caption="Sand Table (fixture): map coordinates 0\u{2013}1000 at 17:05. Text twin: readouts and moments."
                    grid=FrameGrid::Numeric { max: 1000, step: 250 }
                    aspect="aspect-square"
                    texture=true
                    scale=|| view! { <ScaleBar label="200 units" /> }
                >
                    {token("Ornn", Some(DraftSide::Blue), 22.0, 18.0)}
                    {token("Camille", Some(DraftSide::Red), 30.0, 12.0)}
                    {token("Lee Sin", Some(DraftSide::Red), 34.0, 26.0)}
                    {token("Jinx", Some(DraftSide::Blue), 70.0, 72.0)}
                    <Annotation x=22.0 y=18.0 title="17:00 Pick-off" at="220, 820" placement=NotePlacement::SouthEast>
                        "Top pushes alone while four sit bot side; call the reset at 16:30."
                    </Annotation>
                </CanvasFrame>
            </div>
        </GallerySection>
    }
}

#[component]
pub fn TimelineSection() -> impl IntoView {
    let value = RwSignal::new(1025u32);
    let moments = vec![
        Moment::new(360, "First roam"),
        Moment::new(560, "Dragon setup"),
        Moment::new(1020, "Pick-off"),
        Moment::new(1450, "Baron fight"),
    ];
    view! {
        <GallerySection id="timeline" title="Timeline scrub" lede="Reserved for the Sand Table (M4): one clock for map, moments and readouts. Static data for now and no playback, so no play button. Focus the ruler: arrows step a second (Shift: ten), Page Up and Down a minute, J and L jump between moments.">
            <div data-testid="timeline-demo">
                <TimelineScrub label="Replay position (fixture)" duration=1800 value=value moments=moments />
            </div>
        </GallerySection>
    }
}

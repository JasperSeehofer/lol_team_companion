//! `/design` gallery section for the drawn Rift (M152): the two levels of
//! `RiftSketch` as a zoomable preview (1x draws the sketch, 2x and 4x the
//! survey), the Match Day inset and the share strip. The product's
//! zoomable map belongs to a later round (M149); this preview is for review
//! only. All content is fixture text.

use leptos::prelude::*;

use crate::components::ui::{
    RiftAnchor, RiftCrop, RiftLabel, RiftLevel, RiftSketch, RiftZoom, Segment, SegmentedControl,
};
use crate::pages::design::{GallerySection, Sample};

const CAPTION_SKETCH: &str = "Sketch: lanes, the river with a bridge gap at mid, the two bases, Baron and Dragon, ten bush clusters, and a surveyed label in chalk.";
const CAPTION_X2: &str = "Survey at 2x: exact wall outlines with 45-degree hatch, bushes as outlined stipple, towers, inhibitors, nexuses, pits and camps at their positions. Scroll the frame to move around.";
const CAPTION_X4: &str = "Survey at 4x: the same drawing with the hatch held at a 6 px pitch and the marks at their screen size. Scroll the frame to move around.";

#[component]
pub fn RiftSection() -> impl IntoView {
    let stop = RwSignal::new("1".to_string());
    let options = vec![
        Segment::new("1", "1x sketch"),
        Segment::new("2", "2x survey"),
        Segment::new("4", "4x survey"),
    ];
    let caption = move || match stop.get().as_str() {
        "2" => CAPTION_X2,
        "4" => CAPTION_X4,
        _ => CAPTION_SKETCH,
    };
    view! {
        <GallerySection id="rift" title="Drawn Rift" lede="An original schematic of the map, drawn from measured coordinates (docs/rift-schematic.md), at two levels of detail. Decorative: the drawing is hidden from screen readers and the page says in words what it shows.">
            <div class="flex flex-col gap-8">
                <div class="flex flex-col gap-3" data-testid="rift-zoom">
                    <SegmentedControl id="rift-zoom" label="Zoom" options=options value=stop manual=true />
                    <div
                        tabindex="0"
                        role="region"
                        aria-label="Rift preview (scrolls when zoomed in)"
                        class="h-[560px] w-[560px] max-w-full overflow-auto border border-rule bg-page"
                        data-testid="rift-frame"
                    >
                        {move || match stop.get().as_str() {
                            "2" => view! { <RiftSketch level=RiftLevel::Survey zoom=RiftZoom::X2 class="w-[1120px]" /> }.into_any(),
                            "4" => view! { <RiftSketch level=RiftLevel::Survey zoom=RiftZoom::X4 class="w-[2240px]" /> }.into_any(),
                            _ => view! {
                                <div class="w-[560px] p-6">
                                    <RiftSketch label=RiftLabel::new("Your team", RiftAnchor::BlueBase) class="w-full" />
                                </div>
                            }.into_any(),
                        }}
                    </div>
                    <p class="max-w-prose text-caption text-muted" aria-live="polite">{caption}</p>
                </div>
                <div class="flex flex-wrap items-start gap-8">
                    <Sample label="Match Day inset, 88 px">
                        <div class="bg-ink-1 p-3">
                            <RiftSketch crop=RiftCrop::Inset class="w-[88px]" />
                        </div>
                    </Sample>
                    <Sample label="Share strip at the bridge (no hatch, no label)">
                        <RiftSketch crop=RiftCrop::ShareStrip class="w-[358px] max-w-full md:w-[640px]" />
                    </Sample>
                    <Sample label="Share strip at the side the plan was drafted on">
                        <RiftSketch crop=RiftCrop::ShareStripAt(RiftAnchor::BlueBase) label=RiftLabel::new("Ours", RiftAnchor::BlueBase) class="w-[358px] max-w-full md:w-[640px]" />
                    </Sample>
                </div>
            </div>
        </GallerySection>
    }
}

//! The design-system primitives (docs/DESIGN.md, "Primitives").
//!
//! Batch 1: layout (PageShell, PageHeader, SectionHeader, Surface), Text,
//! Button (+ Keycap), Field, TextField, Select, Badge, Table, Tabs, and the
//! domain primitives DraftBoard (the tournament draft) and PrepSequence.
//! Batch 2 (interaction): Dialog/ConfirmDialog, Menu, CommandBar, Toast,
//! Callout, EmptyState, Skeleton, Tooltip, SegmentedControl, Switch, Checkbox.
//! Batch 3 (domain): ChampionAvatar/UserAvatar, RoleLabel, SideMarker,
//! ResultGlyph, StatReadout, LegendBlock, CanvasFrame/ScaleBar,
//! Annotation, TimelineScrub; ProportionBar (M7).
//! M141: SaveStatus / SaveTracker / LoadFailed (status.rs).
//!
//! Rules for every component here: typed variants (enums, no strings),
//! class names written out in full (Tailwind only sees literals), identical
//! markup on server and browser, and a doc comment that states the
//! accessibility contract on a line starting with `A11y:` (a test checks).

pub mod a11y;
pub mod avatar;
pub mod badge;
pub mod bar;
pub mod button;
pub mod callout;
pub mod canvas;
pub mod command;
pub mod dialog;
pub mod draft;
pub mod empty;
pub mod field;
pub mod layout;
pub mod markers;
pub mod menu;
pub mod prep;
pub mod readout;
pub mod skeleton;
pub mod status;
pub mod table;
pub mod tabs;
pub mod text;
pub mod timeline;
pub mod toast;
pub mod toggle;
pub mod tokens;
pub mod tooltip;

pub use avatar::{AvatarShape, AvatarSize, AvatarState, ChampionAvatar, UserAvatar};
pub use badge::{Badge, BadgeTone};
pub use bar::{bar_widths, BarInk, BarSegment, BarSize, ProportionBar};
pub use button::{Button, ButtonSize, ButtonType, ButtonVariant, Force, Keycap};
pub use callout::{Callout, CalloutTone};
pub use canvas::{
    Annotation, CanvasFrame, FrameGrid, LegendBlock, LegendEntry, LineStyle, LineWeight,
    NotePlacement, ScaleBar, Swatch, SwatchInk,
};
pub use command::{Command, CommandBar};
pub use dialog::{ConfirmDialog, ConfirmTone, Dialog, DialogSize};
pub use draft::{
    ban_slots, pick_slots, slot_dom_id, step_status, DraftAction, DraftBoard, DraftFormat,
    DraftPhase, DraftSide, DraftStep, TOURNAMENT_ORDER,
};
pub use empty::{EmptySize, EmptyState, NoTeamState};
pub use field::{
    ctrl_enter_submits, enter_submits, Combobox, Field, FieldContext, FieldSize, InputKind, Select,
    SelectOption, TextField,
};
pub use layout::{
    page_title, Density, Edge, HeadingLevel, Layer, Pad, PageHeader, PageShell, PageWidth,
    SectionHeader, Surface, SurfaceTag,
};
pub use markers::{
    GameResult, GlyphSize, ResultGlyph, Role, RoleLabel, RoleLabelStyle, SideMarker, SideText,
};
pub use menu::{Menu, MenuAlign, MenuItem};
pub use prep::{PrepEmphasis, PrepSequence, PrepState, PrepStep};
pub use readout::{Delta, ReadoutSize, Sentiment, StatReadout, Trend};
pub use skeleton::{Skeleton, SkeletonGroup, SkeletonShape};
pub use status::{use_save_state, LoadFailed, SaveState, SaveStatus, SaveTracker};
pub use table::{
    sort_rows, Align, RowLink, SortDir, SortState, SortStatus, SortTh, Table, Td, Th, Tr,
};
pub use tabs::{TabItem, TabPanel, Tabs};
pub use text::{Text, TextRole, TextTag, Tone};
pub use timeline::{Moment, TimelineScrub};
pub use toast::{Toast, ToastAction, ToastContext, ToastEntry, ToastKind, ToastProvider};
pub use toggle::{Checkbox, Segment, SegmentSize, SegmentedControl, Switch};
pub use tooltip::{Tooltip, TooltipPlacement};

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// Every `#[component]` in this module has a doc comment with an
    /// `A11y:` line (the accessibility contract).
    #[test]
    fn every_component_documents_its_a11y_contract() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/components/ui");
        let mut missing = Vec::new();
        let mut count = 0;
        for entry in std::fs::read_dir(&dir).expect("ui dir") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let src = std::fs::read_to_string(&path).expect("read source");
            let lines: Vec<&str> = src.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if line.trim() != "#[component]" {
                    continue;
                }
                count += 1;
                // Walk up through the doc comment directly above.
                let mut j = i;
                let mut has_a11y = false;
                while j > 0 {
                    j -= 1;
                    let l = lines[j].trim();
                    if l.starts_with("///") {
                        if l.trim_start_matches('/').trim().starts_with("A11y:") {
                            has_a11y = true;
                        }
                    } else if !l.starts_with("#[") {
                        break;
                    }
                }
                let name = lines
                    .get(i + 1)
                    .map(|l| l.trim().to_string())
                    .unwrap_or_default();
                if !has_a11y {
                    missing.push(format!("{}: {name}", path.display()));
                }
            }
        }
        assert!(count >= 20, "found only {count} components");
        assert!(
            missing.is_empty(),
            "components without an `A11y:` doc line: {missing:#?}"
        );
    }
}

//! The design-system primitives (docs/DESIGN.md, "Primitives").
//!
//! Batch 1: layout (PageShell, PageHeader, SectionHeader, Surface), Text,
//! Button (+ Keycap), Field, TextField, Select, Badge, Table, Tabs, and the
//! domain primitives DraftSlot/DraftSequence and PrepSequence.
//! `feedback` holds the older ErrorBanner, StatusMessage, toasts, skeletons
//! and empty states, restyled onto the tokens until batch 2 replaces them.
//!
//! Rules for every component here: typed variants (enums, no strings),
//! class names written out in full (Tailwind only sees literals), identical
//! markup on server and browser, and a doc comment that states the
//! accessibility contract on a line starting with `A11y:` (a test checks).

pub mod badge;
pub mod button;
pub mod draft;
pub mod feedback;
pub mod field;
pub mod layout;
pub mod prep;
pub mod table;
pub mod tabs;
pub mod text;
pub mod tokens;

pub use badge::{Badge, BadgeTone};
pub use button::{Button, ButtonSize, ButtonType, ButtonVariant, Force, Keycap};
pub use draft::{
    DraftAction, DraftPhase, DraftSequence, DraftSide, DraftSlot, DraftStep, SlotState,
    TOURNAMENT_ORDER,
};
pub use feedback::{
    EmptyState, ErrorBanner, NoTeamState, SkeletonCard, SkeletonGrid, SkeletonLine, StatusMessage,
    ToastContext, ToastEntry, ToastKind, ToastProvider,
};
pub use field::{Field, FieldContext, FieldSize, InputKind, Select, SelectOption, TextField};
pub use layout::{
    Density, Edge, HeadingLevel, Layer, Pad, PageHeader, PageShell, PageWidth, SectionHeader,
    Surface, SurfaceTag,
};
pub use prep::{PrepSequence, PrepState, PrepStep};
pub use table::{Align, RowLink, SortDir, Table, Td, Th, Tr};
pub use tabs::{TabItem, TabPanel, Tabs};
pub use text::{Text, TextRole, TextTag, Tone};

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

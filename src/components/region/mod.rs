//! Compatibility shims for pages that predate the design system.
//!
//! The names and props pages still call (`Card`, `SectionHead`, `Btn`,
//! `Badge`, `ModeToggle`, `Eyebrow`, `ChampTile`, `ChampPortrait`,
//! `RankBadge`, `LPProgress`) render batch 1 primitives or token-only
//! markup. There is one theme, so nothing here branches. New code uses
//! `components::ui` directly; M3 deletes this module when the last page
//! has moved.

pub mod chrome;
pub mod controls;
pub mod layout;
pub mod solo;

pub use chrome::*;
pub use controls::*;
pub use layout::*;
pub use solo::*;

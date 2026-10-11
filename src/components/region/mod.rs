//! Compatibility shims for pages that predate the design system.
//!
//! The names and props the solo dashboard and match detail still call
//! (`Card`, `SectionHead`, `ModeToggle`, `Eyebrow`, `RankBadge`,
//! `LPProgress`) render batch 1 primitives or token-only markup. There is
//! one theme, so nothing here branches. New code uses `components::ui`
//! directly; M5 deletes this module when those two pages move.

pub mod controls;
pub mod layout;
pub mod solo;

pub use controls::*;
pub use layout::*;
pub use solo::*;

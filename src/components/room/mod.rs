//! Immersive "room" UI primitives: parallax atmosphere layering and the
//! summonable quick-nav command palette. See the Solo Journey room
//! (`src/pages/rooms/solo_journey.rs`) for the first consumer.

pub mod parallax;
pub mod quick_nav;
pub mod room_surface;

pub use parallax::{AtmosphereRoot, GrowthOverlay, Plane};
pub use quick_nav::QuickNav;
pub use room_surface::{PanelFn, RoomSurface, SurfaceTab};

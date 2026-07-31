//! Colour token system.
//!
//! Layered design:
//! - [`primitive`] — raw named colours, the single source of truth.
//! - [`base`], [`roles`] — shared semantic tokens for the whole UI.
//! - [`markdown`], [`json`], [`sidebar`] — view/component semantic tokens.

pub mod base;
pub mod json;
pub mod markdown;
pub mod primitive;
pub mod roles;
pub mod sidebar;

pub use base::BaseColors;
pub use json::JsonColors;
pub use markdown::MarkdownColors;
pub use roles::RoleColors;
pub use sidebar::SidebarColors;

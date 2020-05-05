//! Otway is a Rust GUI toolkit built on Reclutch and Sinq.
//!
//! Its primary goals are;
//! - To give precise and open-ended control over as much as possible (`ui::Layout`, `theme`).
//! - To have a batteries-included solution where appropriate (`ui::view`, `app`).
//!
//! Although these goals may seem contradicting, the latter simply means to tie up the intentional loose ends left the former for those who need it.
//!
//! # Modules
//!
//! - `ui`; Defines the core interface and is the primary module.
//!     - `ui::view`; Modern and simple interface to compose a UI.
//! - `theme`; Defines the theme interface.
//!     - `theme::flat`; An implementation of the theme interface for a simple, dark, flat-style theme. Feature `themes` required.
//! - `kit`; Toolkit of widgets. Feature `kit` required.
//! - `app`; Application helper utility. Quick one-liner way to load a window and run a UI using Glutin/Winit and Skia, however offers minimal control in return.

#[cfg(feature = "app")]
pub mod app;
#[cfg(feature = "kit")]
pub mod kit;
pub mod theme;
pub mod ui;

pub mod prelude {
    pub use crate::{
        theme::{Theme, TypedPainter},
        ui::{AnyElement, Element, Id, Layout, WidgetChildren},
    };

    #[cfg(feature = "kit")]
    pub use crate::kit::ViewMixin;
}

pub use reclutch;

#[macro_use]
extern crate derivative;

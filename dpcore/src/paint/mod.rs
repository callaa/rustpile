pub mod annotation;
pub mod editlayer;
pub mod rasterop;
pub mod rectiter;
pub mod tile;
pub mod tileiter;
pub mod color;

pub type UserID = u8;
pub type LayerID = i32;

// Re-export types most commonly used from the outside
mod blendmode;
mod brushmask;
mod layer;
mod layerstack;
mod rect;

pub use blendmode::Blendmode;
pub use brushmask::{BrushMask, ClassicBrushCache};
pub use color::{Pixel, Color};
pub use layer::Layer;
pub use layerstack::LayerStack;
pub use rect::Rectangle;

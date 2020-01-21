pub mod annotation;
pub mod color;
pub mod editlayer;
pub mod rasterop;
pub mod rectiter;
pub mod tile;
pub mod tileiter;
pub mod layerstack;

pub type UserID = u8;
pub type LayerID = i32;

// Re-export types most commonly used from the outside
mod blendmode;
mod brushmask;
mod layer;
mod rect;

pub use blendmode::Blendmode;
pub use brushmask::{BrushMask, ClassicBrushCache};
pub use color::{Color, Pixel};
pub use layer::Layer;
pub use layerstack::LayerStack;
pub use rect::Rectangle;

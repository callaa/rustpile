pub mod editlayer;
pub mod rasterop;
pub mod rectiter;
pub mod tile;

pub type UserID = u8;

// Re-export types most commonly used from the outside
mod blendmode;
mod brushmask;
mod color;
mod layer;
mod rect;

pub use blendmode::Blendmode;
pub use brushmask::BrushMask;
pub use color::Color;
pub use layer::Layer;
pub use rect::Rectangle;

// This file is part of Drawpile.
// Copyright (C) 2020 Calle Laakkonen
//
// Drawpile is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// As additional permission under section 7, you are allowed to distribute
// the software through an app store, even if that store has restrictive
// terms and conditions that are incompatible with the GPL, provided that
// the source is also available under the GPL with or without this permission
// through a channel without those restrictive terms and conditions.
//
// Drawpile is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Drawpile.  If not, see <https://www.gnu.org/licenses/>.

pub mod annotation;
pub mod aoe;
pub mod color;
pub mod editlayer;
pub mod layerstack;
pub mod rasterop;
pub mod rectiter;
pub mod tile;
pub mod tileiter;

pub type UserID = u8;
pub type LayerID = i32;

// Re-export types most commonly used from the outside
mod blendmode;
mod brushmask;
mod layer;
mod rect;

pub use aoe::AoE;
pub use blendmode::Blendmode;
pub use brushmask::{BrushMask, ClassicBrushCache};
pub use color::{Color, Pixel};
pub use layer::Layer;
pub use layerstack::LayerStack;
pub use rect::Rectangle;

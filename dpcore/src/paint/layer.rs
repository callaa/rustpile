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

use std::convert::TryFrom;
use std::rc::Rc;

use super::aoe::{AoE, TileMap};
use super::blendmode::Blendmode;
use super::brushmask::BrushMask;
use super::color::{Color, Pixel};
use super::rect::Rectangle;
use super::rectiter::RectIterator;
use super::tile::{Tile, TileData, TILE_SIZE, TILE_SIZEI};
use super::tileiter::MutableTileIterator;
use super::LayerID;

/// A tiled image layer.
///
/// When modifying the layer, the functions in editlayer module should be used.
/// These functions return an Area Of Effect value that can be used to notify
/// layer observers of the changes.
///
#[derive(Clone)]
pub struct Layer {
    pub id: LayerID,
    pub title: String,
    pub opacity: f32,
    pub hidden: bool,
    pub censored: bool,
    pub fixed: bool,
    pub blendmode: Blendmode,
    width: u32,
    height: u32,
    tiles: Rc<Vec<Tile>>,
    sublayers: Vec<Rc<Layer>>,
}

impl Layer {
    /// Construct a new layer filled with the given color
    pub fn new(id: i32, width: u32, height: u32, fill: &Color) -> Layer {
        Layer {
            id,
            title: String::new(),
            opacity: 1.0,
            hidden: false,
            censored: false,
            fixed: false,
            blendmode: Blendmode::Normal,
            width,
            height,
            tiles: Rc::new(vec![
                Tile::new(&fill, 0);
                (Tile::div_up(width) * Tile::div_up(height)) as usize
            ]),
            sublayers: vec![],
        }
    }

    /// Build a layer from raw pixel data
    /// This is typically used for scratch layers as a part of some
    /// larger image manipulation process.
    pub fn from_image(pixels: &[Pixel], width: u32, height: u32) -> Layer {
        let xtiles = Tile::div_up(width);
        let ytiles = Tile::div_up(height);

        let mut layer = Layer::new(0, width, height, &Color::TRANSPARENT);

        let imagerect = Rectangle::new(0, 0, width as i32, height as i32);

        let tilevec = Rc::make_mut(&mut layer.tiles);
        for ty in 0..ytiles {
            for tx in 0..xtiles {
                let srcrect = Rectangle::new(
                    (tx * TILE_SIZE) as i32,
                    (ty * TILE_SIZE) as i32,
                    TILE_SIZE as i32,
                    TILE_SIZE as i32,
                )
                .intersected(&imagerect)
                .unwrap();
                let destrect =
                    srcrect.offset(-((tx * TILE_SIZE) as i32), -((ty * TILE_SIZE) as i32));

                let tile = tilevec[(ty * xtiles + tx) as usize].rect_iter_mut(0, &destrect, true);
                let src = RectIterator::from_rectangle(pixels, width as usize, &srcrect);

                for (destrow, srcrow) in tile.zip(src) {
                    destrow.clone_from_slice(srcrow);
                }
            }
        }

        layer.optimize(&AoE::Everything);
        layer
    }

    /// Get mutable access to a sublayer with the given ID
    ///
    /// If the sublayer does not exist already, it will be created and added to the list of sublayers.
    ///
    /// By convention, ID 0 is not used. Sublayers with positive IDs are used for indirect
    /// drawing (matching user IDs) and sublayers with negative IDs are for local previews.
    pub fn get_or_create_sublayer(&mut self, id: LayerID) -> &mut Layer {
        assert!(id != 0, "Sublayer ID 0 is not allowed");

        if let Some(i) = self.sublayers.iter().position(|sl| sl.id == id) {
            return Rc::make_mut(&mut self.sublayers[i]);
        }
        self.sublayers.push(Rc::new(Layer::new(
            id,
            self.width,
            self.height,
            &Color::TRANSPARENT,
        )));
        let last = self.sublayers.len() - 1;
        Rc::make_mut(&mut self.sublayers[last])
    }

    /// Find and remove a sublayer with the given ID (if it exists)
    ///
    /// Note: you should not typically need to call this directly.
    /// Instead, use `merge_sublayer` or `remove_sublayer` from `editlayer` module
    pub fn take_sublayer(&mut self, id: LayerID) -> Option<Rc<Layer>> {
        if let Some(i) = self.sublayers.iter().position(|sl| sl.id == id) {
            Some(self.sublayers.remove(i))
        } else {
            None
        }
    }

    /// Check if a sublayer with the given ID exists
    pub fn has_sublayer(&self, id: LayerID) -> bool {
        self.sublayers.iter().any(|sl| sl.id == id)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// A layer is visible when it's not explicitly hidden and it's opacity is greater than zero
    ///
    /// The `hidden` flag is typically used as a local-only flag that allows a layer to be
    /// hidden just for the current user.
    pub fn is_visible(&self) -> bool {
        !self.hidden && self.opacity > 0.0
    }

    /// Return the pixel at the given coordinates
    pub fn pixel_at(&self, x: u32, y: u32) -> Pixel {
        let ti = x / TILE_SIZE;
        let tj = y / TILE_SIZE;
        let tx = x - ti * TILE_SIZE;
        let ty = y - tj * TILE_SIZE;

        self.tile(ti, tj).pixel_at(tx, ty)
    }

    /// Get a weighted average of the color under the dab mask
    pub fn sample_dab_color(&self, x: i32, y: i32, dab: &BrushMask) -> Color {
        let sample_rect = match Rectangle::new(x, y, dab.diameter as i32, dab.diameter as i32)
            .cropped(self.width, self.height)
        {
            Some(r) => r,
            None => return Color::TRANSPARENT,
        };

        let tx0 = sample_rect.x / TILE_SIZEI;
        let ty0 = sample_rect.y / TILE_SIZEI;
        let tx1 = sample_rect.right() / TILE_SIZEI;
        let ty1 = sample_rect.bottom() / TILE_SIZEI;

        let mut sum_weight = 0.0;
        let mut sum_color = Color::TRANSPARENT;

        for tx in tx0..=tx1 {
            for ty in ty0..=ty1 {
                let tile = self.tile(tx as u32, ty as u32);
                let tile_rect = Rectangle::tile(tx, ty, TILE_SIZEI);
                let subrect = sample_rect.intersected(&tile_rect).unwrap();
                let rect_in_tile = subrect.offset(-tile_rect.x, -tile_rect.y);
                let rect_in_dab = subrect.offset(-sample_rect.x, -sample_rect.y);
                for (pix, mask) in tile
                    .rect_iter(&rect_in_tile)
                    .flatten()
                    .zip(dab.rect_iter(&rect_in_dab).flatten())
                {
                    let m = *mask as f32 / 255.0;
                    let c = Color::from_pixel(*pix);
                    sum_weight += m;
                    sum_color.r += c.r * m;
                    sum_color.g += c.g * m;
                    sum_color.b += c.b * m;
                    sum_color.a += c.a * m;
                }
            }
        }

        sum_color.r /= sum_weight;
        sum_color.g /= sum_weight;
        sum_color.b /= sum_weight;
        sum_color.a /= sum_weight;

        sum_color
    }

    /// Check if every tile of this layer is the same
    pub fn same_tile(&self) -> Option<Tile> {
        if self.tiles.is_empty() {
            return None;
        }
        let first = &self.tiles[0];
        if self.tiles[1..].iter().any(|t| t != first) {
            return None;
        }

        Some(first.clone())
    }

    /// Check if this entire layer is filled with a solid color
    pub fn solid_color(&self) -> Option<Color> {
        if self.tiles.is_empty() {
            return Some(Color::TRANSPARENT);
        }
        let c = self.tiles[0].solid_color();
        if c.is_none() || self.tiles[1..].iter().any(|t| t.solid_color() != c) {
            None
        } else {
            c
        }
    }

    /// Return a bitmap of non-blank tiles
    pub fn nonblank_tilemap(&self) -> TileMap {
        TileMap {
            tiles: self.tiles.iter().map(|t| *t != Tile::Blank).collect(),
            w: Tile::div_up(self.width),
            h: Tile::div_up(self.height),
        }
    }

    /// Return the tile at the given index
    pub fn tile(&self, i: u32, j: u32) -> &Tile {
        debug_assert!(i * TILE_SIZE < self.width);
        debug_assert!(j * TILE_SIZE < self.height);
        let xtiles = Tile::div_up(self.width);
        &self.tiles[(j * xtiles + i) as usize]
    }

    /// Return a mutable reference to a tile at the given index
    pub fn tile_mut(&mut self, i: u32, j: u32) -> &mut Tile {
        debug_assert!(i * TILE_SIZE < self.width);
        debug_assert!(j * TILE_SIZE < self.height);
        let xtiles = Tile::div_up(self.width);
        let v = Rc::make_mut(&mut self.tiles);
        &mut v[(j * xtiles + i) as usize]
    }

    /// Get direct access to the tile vector.
    /// You normally shouldn't use this directly.
    pub fn tilevec(&self) -> &Vec<Tile> {
        &self.tiles
    }

    /// Get direct mutable access to the tile vector.
    /// You normally shouldn't use this directly. Instead, use the
    /// functions in `editlayer` module.
    pub fn tilevec_mut(&mut self) -> &mut Vec<Tile> {
        Rc::make_mut(&mut self.tiles)
    }

    /// Return a mutable iterator to the tiles that intersect the given
    /// rectangle (in normal pixel coordinates)
    pub fn tile_rect_mut(&mut self, r: &Rectangle) -> MutableTileIterator<Tile> {
        assert!(
            r.x >= 0
                && r.y >= 0
                && r.right() < self.width as i32
                && r.bottom() < self.height as i32
        );
        let tx0 = (r.x / TILE_SIZEI) as usize;
        let tx1 = (r.right() / TILE_SIZEI) as usize;
        let ty0 = (r.y / TILE_SIZEI) as usize;
        let ty1 = (r.bottom() / TILE_SIZEI) as usize;
        let stride = Tile::div_up(self.width) as usize;

        MutableTileIterator::new(
            self.tilevec_mut(),
            stride,
            tx0,
            ty0,
            tx1 - tx0 + 1,
            ty1 - ty0 + 1,
        )
    }

    pub fn flatten_tile(&self, destination: &mut TileData, i: u32, j: u32) {
        if !self.is_visible() {
            return;
        }

        // TODO censor
        if self.sublayers.is_empty() {
            // No sublayers: just composite this one as is
            destination.merge_tile(self.tile(i, j), self.opacity, self.blendmode);
        } else {
            // Sublayers present: compositing needed
            let mut tmp = self.tile(i, j).clone_data();
            for sublayer in self.sublayers.iter() {
                if sublayer.is_visible() {
                    tmp.merge_tile(sublayer.tile(i, j), sublayer.opacity, sublayer.blendmode);
                }
            }

            // TODO tint, highlight and onionskin

            destination.merge_data(&tmp, self.opacity, self.blendmode);
        }
    }

    /// Call optimize on every tile in the given area.
    /// This will release memory and speed up rendering, as blank
    /// tiles can be skipped.
    pub fn optimize(&mut self, area: &AoE) {
        use AoE::*;
        match area {
            Nothing => (),
            Bounds(r) => self.tile_rect_mut(r).for_each(|(_, _, t)| t.optimize()),
            // Note: when using a bitmap, we need to iterate through the
            // entire vector anyway and an optimize call is cheap enough
            // that there's no point to the extra check.
            _ => {
                Rc::make_mut(&mut self.tiles)
                    .iter_mut()
                    .for_each(|t| t.optimize());
            }
        }
    }

    /// Do a shallow comparison between these layers and return the difference
    pub fn compare(&self, other: &Layer) -> AoE {
        if Rc::ptr_eq(&self.tiles, &other.tiles) {
            return AoE::Nothing;
        }
        if self.width != other.width || self.height != other.height {
            return AoE::Resize(0, 0);
        }

        TileMap {
            tiles: self
                .tiles
                .iter()
                .zip(other.tiles.iter())
                .map(|(a, b)| !a.ptr_eq(b))
                .collect(),
            w: Tile::div_up(self.width),
            h: Tile::div_up(self.height),
        }
        .into()
    }

    /// Return a new layer with the size adjusted by the given values
    ///
    /// The new layer will contain the same content as this one, but offset and possibly cropped.
    /// If the layer is filled with a solid color, the resized layer will also be fully filled
    /// with that color. Otherwise, the expanded areas will be filled with transparency.
    pub fn resized(&self, top: i32, right: i32, bottom: i32, left: i32) -> Layer {
        let new_width = u32::try_from(left + self.width as i32 + right).unwrap();
        let new_height = u32::try_from(top + self.height as i32 + bottom).unwrap();

        let new_tiles = if let Some(c) = self.solid_color() {
            // The fastest case: this layer is filled with solid color
            Rc::new(vec![
                Tile::new(&c, 0);
                (Tile::div_up(new_width) * Tile::div_up(new_height))
                    as usize
            ])
        } else if (left % TILE_SIZEI) == 0 && (top % TILE_SIZEI) == 0 {
            // Tile aligned resize. Existing tiles can be reused.
            self.resized_fast(left, top, new_width, new_height)
        } else {
            // Uh oh, top/left change not evenly divisible by tile size
            // means we have to rebuild all the tiles
            self.resized_slow(left, top, new_width, new_height)
        };

        Layer {
            title: self.title.clone(),
            width: new_width,
            height: new_height,
            tiles: new_tiles,
            sublayers: self
                .sublayers
                .iter()
                .map(|sl| Rc::new(sl.resized(top, right, bottom, left)))
                .collect(),
            ..*self
        }
    }

    fn resized_slow(&self, offx: i32, offy: i32, w: u32, h: u32) -> Rc<Vec<Tile>> {
        let oldxtiles = Tile::div_up(self.width) as i32;
        let newxtiles = Tile::div_up(w) as i32;
        let newytiles = Tile::div_up(h) as i32;
        let mut new_vec = Rc::new(vec![Tile::Blank; (newxtiles * newytiles) as usize]);
        let tiles = Rc::make_mut(&mut new_vec);

        // Iterate through the original image. Most of the time, the canvas
        // is being expanded so this is the set of tiles we'd iterate over anyway.
        for (index, tile) in self.tiles.iter().enumerate() {
            if *tile == Tile::Blank {
                continue;
            }

            let source_tile_geometry = Rectangle::tile(
                index as i32 % oldxtiles,
                index as i32 / oldxtiles,
                TILE_SIZEI,
            );

            // Where the pixel data should be moved.
            // It might be out of bounds if the layer is being contracted.
            let target_rect = source_tile_geometry.offset(offx, offy);

            if let Some(cropped) = target_rect.cropped(w, h) {
                // It appears to be in bounds.

                // Depending on the offset, the source tile will overlap
                // with either 2 or 4 destination tiles
                let dx0 = (cropped.x / TILE_SIZEI) as usize;
                let dx1 = (cropped.right() / TILE_SIZEI) as usize;
                let dy0 = (cropped.y / TILE_SIZEI) as usize;
                let dy1 = (cropped.bottom() / TILE_SIZEI) as usize;

                for (i, j, dest_tile) in MutableTileIterator::new(
                    tiles,
                    newxtiles as usize,
                    dx0,
                    dy0,
                    dx1 - dx0 + 1,
                    dy1 - dy0 + 1,
                ) {
                    let destination_tile_geometry = Rectangle::tile(i, j, TILE_SIZEI);

                    // The half or quarter destination tile
                    let subrect = destination_tile_geometry.intersected(&target_rect).unwrap();

                    // Destination rectangle inside the destination tile
                    let dest_tile_rect =
                        subrect.offset(-destination_tile_geometry.x, -destination_tile_geometry.y);

                    // Source rectangle inside the source tile
                    let source_tile_rect = subrect.offset(-target_rect.x, -target_rect.y);

                    dest_tile
                        .rect_iter_mut(tile.last_touched_by(), &dest_tile_rect, true)
                        .zip(tile.rect_iter(&source_tile_rect))
                        .for_each(|(d, s)| d.clone_from_slice(s));
                }
            }
        }

        new_vec
    }

    fn resized_fast(&self, offx: i32, offy: i32, w: u32, h: u32) -> Rc<Vec<Tile>> {
        debug_assert!(offx % TILE_SIZEI == 0);
        debug_assert!(offy % TILE_SIZEI == 0);
        let oldxtiles = Tile::div_up(self.width) as i32;
        let oldytiles = Tile::div_up(self.height) as i32;
        let newxtiles = Tile::div_up(w) as i32;
        let newytiles = Tile::div_up(h) as i32;
        let mut new_vec = Rc::new(vec![Tile::Blank; (newxtiles * newytiles) as usize]);

        let xt_off = offx / TILE_SIZEI;
        let yt_off = offy / TILE_SIZEI;

        let tiles = Rc::make_mut(&mut new_vec);

        for y in yt_off.max(0)..newytiles.min(oldytiles + yt_off) {
            let sy = y - yt_off;
            for x in xt_off.max(0)..newxtiles.min(oldxtiles + xt_off) {
                let sx = x - xt_off;
                tiles[(y * newxtiles + x) as usize] = self.tile(sx as u32, sy as u32).clone();
            }
        }

        new_vec
    }

    #[cfg(test)]
    fn refcount(&self) -> usize {
        Rc::strong_count(&self.tiles) + Rc::weak_count(&self.tiles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::color::{WHITE_PIXEL, ZERO_PIXEL};
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_tilevector_cow() {
        let mut layer = Layer::new(
            0,
            100,
            64,
            &Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        let layer2 = layer.clone();
        assert_eq!(layer.refcount(), 2);

        // Tile vectors are shared at this point
        assert_eq!(layer.tiles[0].refcount(), 2);
        assert_eq!(layer.compare(&layer2), AoE::Nothing);

        // Changing a tile makes the vectors unique:
        // there are now more references to the same tile data
        *layer.tile_mut(0, 0) = Tile::Blank;
        assert_eq!(layer.refcount(), 1);
        assert_eq!(layer2.tiles[0].refcount(), 3);

        assert_eq!(
            layer.compare(&layer2),
            AoE::Bitmap(TileMap {
                tiles: bitvec![1, 0],
                w: 2,
                h: 1,
            })
        );
    }

    #[test]
    fn test_from_small_image() {
        #[rustfmt::skip]
        let image = [
            [0, 0, 0, 1], [0, 0, 0, 2], [0, 0, 0, 3],
            [0, 0, 1, 1], [0, 0, 1, 2], [0, 0, 1, 3],
        ];
        let layer = Layer::from_image(&image, 3, 2);
        assert_eq!(layer.pixel_at(0, 0), [0, 0, 0, 1]);
        assert_eq!(layer.pixel_at(1, 0), [0, 0, 0, 2]);
        assert_eq!(layer.pixel_at(2, 0), [0, 0, 0, 3]);
        assert_eq!(layer.pixel_at(3, 0), [0, 0, 0, 0]);
        assert_eq!(layer.pixel_at(0, 1), [0, 0, 1, 1]);
        assert_eq!(layer.pixel_at(1, 1), [0, 0, 1, 2]);
        assert_eq!(layer.pixel_at(2, 1), [0, 0, 1, 3]);
        assert_eq!(layer.pixel_at(3, 1), [0, 0, 0, 0]);
        assert_eq!(layer.pixel_at(0, 2), [0, 0, 0, 0]);
    }

    #[test]
    fn test_solid_expand() {
        let r = Color::rgb8(255, 0, 0);
        let layer = Layer::new(0, TILE_SIZE, TILE_SIZE, &r);
        let layer2 = layer.resized(10, 10, 0, 0);
        for t in layer2.tiles.iter() {
            assert_eq!(t.solid_color(), Some(r));
        }
    }

    #[test]
    fn test_fast_expand() {
        let t = Color::TRANSPARENT;

        let mut layer = Layer::new(0, TILE_SIZE, TILE_SIZE, &t);
        // Change a pixel so the whole layer won't be of uniform color
        layer
            .tile_mut(0, 0)
            .rect_iter_mut(0, &Rectangle::new(1, 1, 1, 1), false)
            .next()
            .unwrap()[0] = WHITE_PIXEL;

        let layer2 = layer.resized(TILE_SIZEI, TILE_SIZEI, 2 * TILE_SIZEI, TILE_SIZEI);

        // Should look like:
        // 000
        // 0X0
        // 000
        // 000

        assert_eq!(layer2.width(), TILE_SIZE * 3);
        assert_eq!(layer2.height(), TILE_SIZE * 4);
        assert_eq!(layer2.tile(0, 0).solid_color(), Some(t));
        assert_eq!(layer2.tile(1, 0).solid_color(), Some(t));
        assert_eq!(layer2.tile(2, 0).solid_color(), Some(t));

        assert_eq!(layer2.tile(0, 1).solid_color(), Some(t));
        assert_eq!(layer2.tile(1, 1).solid_color(), None);
        assert_eq!(layer2.tile(2, 1).solid_color(), Some(t));

        assert_eq!(layer2.tile(0, 2).solid_color(), Some(t));
        assert_eq!(layer2.tile(1, 2).solid_color(), Some(t));
        assert_eq!(layer2.tile(2, 2).solid_color(), Some(t));

        assert_eq!(layer2.tile(0, 3).solid_color(), Some(t));
        assert_eq!(layer2.tile(1, 3).solid_color(), Some(t));
        assert_eq!(layer2.tile(2, 3).solid_color(), Some(t));
    }

    #[test]
    fn test_fast_contract() {
        let t = Color::TRANSPARENT;

        let mut layer = Layer::new(0, TILE_SIZE * 3, TILE_SIZE * 3, &t);
        // Change a pixel so the whole layer won't be of uniform color
        // and so we can distinguish the tiles from the new fully transparent ones.
        for y in 0..3 {
            for x in 0..3 {
                layer
                    .tile_mut(x, y)
                    .rect_iter_mut(0, &Rectangle::new(0, 0, 1, 1), false)
                    .next()
                    .unwrap()[0] = WHITE_PIXEL;
            }
        }

        let layer2 = layer.resized(-TILE_SIZEI, -TILE_SIZEI * 2, TILE_SIZEI, 0);

        // Should look like:
        // X
        // X
        // O

        assert_eq!(layer2.width(), TILE_SIZE);
        assert_eq!(layer2.height(), TILE_SIZE * 3);
        assert_eq!(layer2.tile(0, 0).solid_color(), None);
        assert_eq!(layer2.tile(0, 1).solid_color(), None);
        assert_eq!(layer2.tile(0, 2).solid_color(), Some(t));
    }

    #[test]
    fn test_slow_expand() {
        let mut layer = Layer::new(0, TILE_SIZE, TILE_SIZE, &Color::rgb8(0, 0, 0));
        layer
            .tile_mut(0, 0)
            .rect_iter_mut(0, &Rectangle::new(0, 0, 1, 1), false)
            .next()
            .unwrap()[0] = WHITE_PIXEL;

        let layer = layer.resized(10, 0, 0, 5);

        assert_eq!(layer.width, TILE_SIZE + 5);
        assert_eq!(layer.height, TILE_SIZE + 10);

        assert_eq!(layer.pixel_at(0, 0), ZERO_PIXEL);
        assert_eq!(layer.pixel_at(5, 10), WHITE_PIXEL);
    }

    #[test]
    fn test_slow_contract() {
        let mut layer = Layer::new(0, TILE_SIZE, TILE_SIZE, &Color::TRANSPARENT);
        layer
            .tile_mut(0, 0)
            .rect_iter_mut(0, &Rectangle::new(5, 10, 1, 1), false)
            .next()
            .unwrap()[0] = WHITE_PIXEL;

        let layer = layer.resized(-10, 0, 0, -5);

        assert_eq!(layer.width, TILE_SIZE - 5);
        assert_eq!(layer.height, TILE_SIZE - 10);

        assert_eq!(layer.pixel_at(5, 10), ZERO_PIXEL);
        assert_eq!(layer.pixel_at(0, 0), WHITE_PIXEL);
    }

    #[test]
    fn test_sample_dab() {
        let mut layer = Layer::new(0, TILE_SIZE * 2, TILE_SIZE * 2, &Color::TRANSPARENT);
        layer.tile_mut(0, 0).fill(&Color::rgb8(255, 0, 0), 0);
        layer.tile_mut(1, 0).fill(&Color::rgb8(0, 255, 0), 0);
        layer.tile_mut(0, 1).fill(&Color::rgb8(0, 0, 255), 0);

        let dab_mask = BrushMask::new_round_pixel(32, 1.0);
        let sampled = layer.sample_dab_color(64 - 16, 64 - 16, &dab_mask);

        assert_eq!(
            sampled,
            Color {
                r: 0.25,
                g: 0.25,
                b: 0.25,
                a: 0.75
            }
        );
    }
}

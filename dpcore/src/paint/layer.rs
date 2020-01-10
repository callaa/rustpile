use std::convert::TryFrom;
use std::rc::Rc;

use super::blendmode::Blendmode;
use super::color::Color;
use super::rect::Rectangle;
use super::rectiter::RectIterator;
use super::tile::{Tile, TileData, TILE_SIZE, TILE_SIZEI};
use super::tileiter::MutableTileIterator;

/// A tiled image layer.
///
/// When modifying the layer, the functions in editlayer module should be used.
/// These functions return an Area Of Effect value that can be used to notify
/// layer observers of the changes.
///
#[derive(Clone)]
pub struct Layer {
    pub id: i32,
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
    pub fn from_image(pixels: &[u32], width: u32, height: u32) -> Layer {
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

                let tile = tilevec[(ty * xtiles + tx) as usize].rect_iter_mut(0, &destrect);
                let src = RectIterator::from_rectangle(pixels, width as usize, &srcrect);

                for (destrow, srcrow) in tile.zip(src) {
                    destrow.clone_from_slice(srcrow);
                }
            }
        }

        layer
    }

    /// Get mutable access to a sublayer with the given ID
    ///
    /// If the sublayer does not exist already, it will be created and added to the list of sublayers.
    ///
    /// By convention, ID 0 is not used. Sublayers with positive IDs are used for indirect
    /// drawing (matching user IDs) and sublayers with negative IDs are for local previews.
    pub fn get_or_create_sublayer(&mut self, id: i32) -> &mut Layer {
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
    pub fn take_sublayer(&mut self, id: i32) -> Option<Rc<Layer>> {
        if let Some(i) = self.sublayers.iter().position(|sl| sl.id == id) {
            Some(self.sublayers.remove(i))
        } else {
            None
        }
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
    pub fn pixel_at(&self, x: u32, y: u32) -> u32 {
        let ti = x / TILE_SIZE;
        let tj = y / TILE_SIZE;
        let tx = x - ti * TILE_SIZE;
        let ty = y - tj * TILE_SIZE;

        self.tile(ti, tj).pixel_at(tx, ty)
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

            destination.merge_data(&mut tmp, self.opacity, self.blendmode);
        }
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
            self.resized_fast(left, top, new_width as u32, new_height as u32)
        } else {
            // Uh oh, top/left change not evenly divisible by tile size
            // means we have to rebuild all the tiles
            self.resized_slow()
        };

        Layer {
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

    fn resized_slow(&self) -> Rc<Vec<Tile>> {
        todo!();
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
    use super::*;

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

        // Changing a tile makes the vectors unique:
        // there are now more references to the same tile data
        *layer.tile_mut(0, 0) = Tile::Blank;
        assert_eq!(layer.refcount(), 1);
        assert_eq!(layer2.tiles[0].refcount(), 3);
    }

    #[test]
    fn test_from_small_image() {
        let image = [1, 2, 3, 11, 12, 13];
        let layer = Layer::from_image(&image, 3, 2);
        assert_eq!(layer.pixel_at(0, 0), 1);
        assert_eq!(layer.pixel_at(1, 0), 2);
        assert_eq!(layer.pixel_at(2, 0), 3);
        assert_eq!(layer.pixel_at(3, 0), 0);
        assert_eq!(layer.pixel_at(0, 1), 11);
        assert_eq!(layer.pixel_at(1, 1), 12);
        assert_eq!(layer.pixel_at(2, 1), 13);
        assert_eq!(layer.pixel_at(3, 1), 0);
        assert_eq!(layer.pixel_at(0, 2), 0);
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
            .rect_iter_mut(0, &Rectangle::new(1, 1, 1, 1))
            .next()
            .unwrap()[0] = 0xff_ffffff;

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
                    .rect_iter_mut(0, &Rectangle::new(0, 0, 1, 1))
                    .next()
                    .unwrap()[0] = 0xff_ffffff;
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
}

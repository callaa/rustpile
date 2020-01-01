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

    /// Return the tile at the given index
    pub fn tile(&self, i: u32, j: u32) -> &Tile {
        let xtiles = Tile::div_up(self.width);
        &self.tiles[(j * xtiles + i) as usize]
    }

    /// Return a mutable reference to a tile at the given index
    pub fn tile_mut(&mut self, i: u32, j: u32) -> &mut Tile {
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
}

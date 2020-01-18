use std::fmt;
use std::rc::Rc;

use super::color::*;
use super::rectiter::{MutableRectIterator, RectIterator};
use super::{rasterop, Blendmode, Rectangle, UserID};

pub const TILE_SIZE: u32 = 64;
pub const TILE_SIZEI: i32 = TILE_SIZE as i32;
const TILE_LENGTH: usize = (TILE_SIZE * TILE_SIZE) as usize;

#[derive(Clone)]
pub struct TileData {
    pub pixels: [Pixel; TILE_LENGTH],
    pub last_touched_by: UserID,
}

#[derive(Clone)]
pub enum Tile {
    Bitmap(Rc<TileData>),
    Blank,
}

static TRANSPARENT_DATA: TileData = TileData {
    pixels: [ZERO_PIXEL; TILE_LENGTH],
    last_touched_by: 0,
};

impl TileData {
    pub fn new(pixel: Pixel, user: UserID) -> TileData {
        TileData {
            pixels: [pixel; TILE_LENGTH],
            last_touched_by: user,
        }
    }

    pub fn merge_data(&mut self, other: &TileData, opacity: f32, mode: Blendmode) {
        rasterop::pixel_blend(
            &mut self.pixels,
            &other.pixels,
            (opacity * 255.0) as u8,
            mode,
        );
    }

    pub fn merge_tile(&mut self, other: &Tile, opacity: f32, mode: Blendmode) {
        match other {
            Tile::Bitmap(td) => self.merge_data(td, opacity, mode),
            Tile::Blank => (),
        }
    }
}

impl Tile {
    // Construct a new tile filled with the given color.
    // If the color is transparent, a Blank tile is returned.
    pub fn new(color: &Color, user: UserID) -> Tile {
        let p = color.as_pixel();
        if p[ALPHA_CHANNEL] == 0 {
            Tile::Blank
        } else {
            Tile::Bitmap(Rc::new(TileData::new(p, user)))
        }
    }

    // Construct a new tile filled with the given color.
    // A bitmap tile is constructed even if the color is transparent.
    pub fn new_solid(color: &Color, user: UserID) -> Tile {
        Tile::Bitmap(Rc::new(TileData::new(color.as_pixel(), user)))
    }

    pub fn div_up(x: u32) -> u32 {
        (x + TILE_SIZE - 1) / TILE_SIZE
    }

    // Check if every pixel of this tile is the same
    pub fn solid_color(&self) -> Option<Color> {
        match self {
            Tile::Bitmap(td) => {
                let pix = td.pixels[0];
                match td.pixels.into_iter().all(|&p| p == pix) {
                    true => Some(Color::from_pixel(pix)),
                    false => None,
                }
            }
            Tile::Blank => Some(Color::TRANSPARENT),
        }
    }

    pub fn clone_data(&self) -> TileData {
        match self {
            Tile::Bitmap(td) => TileData::clone(td),
            Tile::Blank => TileData::new(ZERO_PIXEL, 0),
        }
    }

    /// Check if all pixels of this tile are fully transparent
    pub fn is_blank(&self) -> bool {
        match self {
            Tile::Bitmap(td) => td.pixels.iter().all(|&p| p[ALPHA_CHANNEL] == 0),
            Tile::Blank => true,
        }
    }

    /// Get the last touched by user ID tag
    pub fn last_touched_by(&self) -> UserID {
        match self {
            Tile::Bitmap(td) => td.last_touched_by,
            Tile::Blank => 0,
        }
    }

    // Fill this tile with a solid color
    pub fn fill(&mut self, color: &Color, user: UserID) -> () {
        if color.a == 0.0 {
            *self = Tile::Blank
        } else {
            match self {
                Tile::Bitmap(td) => {
                    let pixel = color.as_pixel();
                    let data = Rc::make_mut(td);
                    data.last_touched_by = user;
                    for i in data.pixels.iter_mut() {
                        *i = pixel;
                    }
                }
                Tile::Blank => {
                    *self = Tile::new_solid(color, user);
                }
            }
        }
    }

    /// Merge this tile with the other tile
    pub fn merge(&mut self, other: &Tile, opacity: f32, mode: Blendmode) {
        if let Tile::Bitmap(o) = other {
            match self {
                Tile::Bitmap(td) => Rc::make_mut(td).merge_data(o, opacity, mode),
                Tile::Blank => {
                    // TODO optimization: in certain cases we can just replace this tile with other
                    if mode.can_increase_opacity() {
                        *self = Tile::new_solid(&Color::TRANSPARENT, other.last_touched_by());
                        self.merge(other, opacity, mode);
                    }
                }
            }
        }
    }

    // Return a rect iterator to this tile's content
    // Note: you may want to check if this is a Blank tile first for optimization purposes
    pub fn rect_iter(&self, r: &Rectangle) -> RectIterator<Pixel> {
        debug_assert!(r.x >= 0 && r.y >= 0);
        debug_assert!(r.right() < TILE_SIZEI && r.bottom() < TILE_SIZEI);

        match self {
            Tile::Bitmap(d) => RectIterator::from_rectangle(&d.pixels, TILE_SIZE as usize, r),
            Tile::Blank => {
                RectIterator::from_rectangle(&TRANSPARENT_DATA.pixels, TILE_SIZE as usize, r)
            }
        }
    }

    // Return a mutable iterator to this tile's content
    // If this is a Blank tile, it is converted to a fully transparent Bitmap tile first.
    pub fn rect_iter_mut(&mut self, user: UserID, r: &Rectangle) -> MutableRectIterator<Pixel> {
        debug_assert!(r.x >= 0 && r.y >= 0);
        debug_assert!(r.right() < TILE_SIZEI && r.bottom() < TILE_SIZEI);

        match self {
            Tile::Bitmap(td) => {
                let data = Rc::make_mut(td);
                MutableRectIterator::from_rectangle(&mut data.pixels, TILE_SIZE as usize, r)
            }
            Tile::Blank => {
                *self = Tile::new_solid(&Color::TRANSPARENT, user);
                self.rect_iter_mut(user, r)
            }
        }
    }

    pub fn pixel_at(&self, x: u32, y: u32) -> Pixel {
        debug_assert!(x < TILE_SIZE);
        debug_assert!(y < TILE_SIZE);
        match self {
            Tile::Bitmap(td) => td.pixels[(y * TILE_SIZE + x) as usize],
            Tile::Blank => ZERO_PIXEL,
        }
    }

    #[cfg(test)]
    pub fn refcount(&self) -> usize {
        match self {
            Tile::Bitmap(d) => Rc::strong_count(&d) + Rc::weak_count(&d),
            Tile::Blank => 0,
        }
    }

    #[cfg(debug_assertions)]
    pub fn to_ascii_art(&self) -> String {
        let mut art = String::new();
        match self {
            Tile::Bitmap(td) => {
                for y in 0..TILE_SIZE {
                    for x in 0..TILE_SIZE {
                        art.push(if td.pixels[(y * TILE_SIZE + x) as usize][0] == 0 {
                            '.'
                        } else {
                            'X'
                        });
                    }
                    art.push('\n');
                }
            }
            Tile::Blank => {
                art = String::from("[BLANK TILE]");
            }
        }
        art
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        match self {
            Tile::Bitmap(td) => match other {
                Tile::Bitmap(otd) => td.pixels[..] == otd.pixels[..],
                Tile::Blank => self.is_blank(),
            },
            Tile::Blank => other.is_blank(),
        }
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Bitmap(d) => write!(
                f,
                "Tile(pixels=[{:?}...{:?}], user={}, refs={})",
                d.pixels[0],
                d.pixels[TILE_LENGTH - 1],
                d.last_touched_by,
                Rc::strong_count(&d)
            ),
            Tile::Blank => write!(f, "Tile(blank)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow() {
        let mut tile = Tile::new_solid(&Color::TRANSPARENT, 0);
        let tile2 = tile.clone();
        let tile3 = tile2.clone();

        assert_eq!(tile.refcount(), 3);
        assert_eq!(tile2.refcount(), 3);
        assert_eq!(tile3.refcount(), 3);

        tile.fill(
            &Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            1,
        );

        assert_eq!(tile.refcount(), 1);
        assert_eq!(tile2.refcount(), 2);
        assert_eq!(tile3.refcount(), 2);
    }

    #[test]
    fn test_cow_vec() {
        let mut vec1 = vec![Tile::new_solid(&Color::TRANSPARENT, 0); 3];
        let vec2 = vec1.clone();

        assert_eq!(vec1[0].refcount(), 6);
        vec1[0] = Tile::Blank;
        assert_eq!(vec2[0].refcount(), 5);
    }

    #[test]
    fn test_solid() {
        let mut tile = Tile::Blank;
        assert_eq!(tile.solid_color(), Some(Color::TRANSPARENT));
        assert!(tile.is_blank());

        let red = Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        tile.fill(&red, 1);
        assert_eq!(tile.solid_color(), Some(red));
        assert!(!tile.is_blank());

        tile.rect_iter_mut(1, &Rectangle::new(0, 0, 3, 3))
            .flatten()
            .for_each(|p| *p = WHITE_PIXEL);
        assert_eq!(tile.solid_color(), None);
        assert!(!tile.is_blank());
    }

    #[test]
    fn test_merge() {
        let mut btm = Tile::new_solid(&Color::rgb8(0, 0, 0), 0);
        let top = Tile::new_solid(&Color::rgb8(255, 255, 255), 0);
        btm.merge(&top, 0.5, Blendmode::Normal);

        assert_eq!(
            btm.solid_color(),
            Some(Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0
            })
        );
    }

    #[test]
    fn test_merge_blank() {
        let mut btm = Tile::Blank;
        let top = Tile::new_solid(&Color::rgb8(255, 255, 255), 0);
        btm.merge(&top, 0.5, Blendmode::Normal);

        assert_eq!(
            btm.solid_color(),
            Some(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.5
            })
        );
    }
}

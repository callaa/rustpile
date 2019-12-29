use super::rectiter::RectIterator;
use super::tile::{Tile, TILE_SIZE, TILE_SIZEI};
use super::{rasterop, Blendmode, BrushMask, Color, Layer, Rectangle, UserID};

/// A layer editing operation's area of effect.
///
/// This is used inform layer observers which part
/// of the layer stack should be refreshed.
///
/// Note that the AoE can be `Nothing` even if some pixels were changed
/// if the layer is not visible.
pub enum AoE {
    Rectangle(Rectangle),
    Everything,
    Nothing,
}

/// Fills a rectangle with a solid color using the given blending mode
///
/// # Arguments
///
/// * `layer` - The target layer
/// * `user` - User ID tag to attach to the changed tiles
/// * `color` - Fill color
/// * `mode` - Fill blending mode
/// * `rect` - Rectangle to fill
pub fn fill_rect(
    layer: &mut Layer,
    user: UserID,
    color: &Color,
    mode: Blendmode,
    rect: &Rectangle,
) -> AoE {
    let r = match rect.cropped(layer.width(), layer.height()) {
        Some(r) => r,
        None => return AoE::Nothing,
    };

    let tx0 = r.x / TILE_SIZEI;
    let tx1 = r.right() / TILE_SIZEI;
    let ty0 = r.y / TILE_SIZEI;
    let ty1 = r.bottom() / TILE_SIZEI;

    let xtiles = Tile::div_up(layer.width()) as i32;

    let tiles = layer.tilevec_mut();

    let pixels = [color.as_pixel(); TILE_SIZE as usize];

    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            let tile = &mut tiles[(ty * xtiles + tx) as usize];
            // TODO if this is a Blank tile and blendmode does not have the "can increase opacity"
            // flag, then this tile can be skipped.

            let x0 = tx * TILE_SIZEI;
            let y0 = ty * TILE_SIZEI;
            let subrect = Rectangle::new(x0, y0, TILE_SIZEI, TILE_SIZEI)
                .intersected(&r)
                .unwrap()
                .offset(-x0, -y0);
            for row in tile.rect_iter_mut(user, &subrect) {
                rasterop::pixel_blend(row, &pixels, 0xff, mode);
            }
        }
    }

    AoE::Rectangle(r)
}

/// Draw a brush mask onto the layer
///
/// Note: When drawing in "indirect" mode, the brush dabs should be
/// drawn to to a sublayer. The sublayer is then merged at the end of the stroke.
///
/// # Arguments
///
/// * `layer` - The target layer
/// * `user` - User ID tag to attach to the changed tiles
/// * `x` - Left edge of the mask
/// * `y` - Top edge of the mask
/// * `mask` - The brush mask
/// * `color` - The brush color
/// * `mode` - Brush blending mode
pub fn draw_brush_dab(
    layer: &mut Layer,
    user: UserID,
    x: i32,
    y: i32,
    mask: &BrushMask,
    color: &Color,
    mode: Blendmode,
) -> AoE {
    let d = mask.diameter as i32;
    let rect = match Rectangle::new(x, y, d, d).cropped(layer.width(), layer.height()) {
        Some(r) => r,
        None => return AoE::Nothing,
    };

    let tx0 = rect.x / TILE_SIZEI;
    let tx1 = rect.right() / TILE_SIZEI;
    let ty0 = rect.y / TILE_SIZEI;
    let ty1 = rect.bottom() / TILE_SIZEI;

    let xtiles = Tile::div_up(layer.width()) as i32;
    let colorpix = color.as_pixel();
    let tiles = layer.tilevec_mut();

    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            let tile = &mut tiles[(ty * xtiles + tx) as usize];
            // TODO if this is a Blank tile and blendmode does not have the "can increase opacity"
            // flag, then this tile can be skipped.

            let x0 = tx * TILE_SIZEI;
            let y0 = ty * TILE_SIZEI;
            let subrect = Rectangle::new(x0, y0, TILE_SIZEI, TILE_SIZEI)
                .intersected(&rect)
                .unwrap();
            let tilerect = subrect.offset(-x0, -y0);
            let maskrect = rect.intersected(&subrect).unwrap().offset(-x, -y);

            for (destrow, maskrow) in
                tile.rect_iter_mut(user, &tilerect)
                    .zip(RectIterator::from_rectangle(
                        &mask.mask,
                        mask.diameter as usize,
                        &maskrect,
                    ))
            {
                rasterop::mask_blend(destrow, colorpix, maskrow, mode);
            }
        }
    }

    AoE::Rectangle(rect)
}

/// Merge another layer to this one
///
/// The other layer's opacity and blending mode are used.
///
/// The returned area of effect contains all the visible tiles of the source layer
pub fn merge(target_layer: &mut Layer, source_layer: &Layer) -> AoE {
    assert_eq!(target_layer.size(), source_layer.size());

    let target_tiles = target_layer.tilevec_mut();
    let source_tiles = source_layer.tilevec();

    // TODO this is parallelizable
    target_tiles
        .into_iter()
        .zip(source_tiles.into_iter())
        .for_each(|(d, s)| d.merge(s, source_layer.opacity, source_layer.blendmode));

    if source_layer.is_visible() {
        // TODO visible tiles only
        AoE::Everything
    } else {
        AoE::Nothing
    }
}

/// Merge a sublayer
pub fn merge_sublayer(layer: &mut Layer, sublayer_id: i32) -> AoE {
    if let Some(sublayer) = layer.take_sublayer(sublayer_id) {
        merge(layer, &sublayer)
    } else {
        AoE::Nothing
    }
}

/// Remove a sublayer without merging it
pub fn remove_sublayer(layer: &mut Layer, sublayer_id: i32) -> AoE {
    if let Some(_sublayer) = layer.take_sublayer(sublayer_id) {
        // TODO visible tiles only
        AoE::Everything
    } else {
        AoE::Nothing
    }
}

#[cfg(test)]
mod tests {
    use super::super::BrushMask;
    use super::*;

    #[test]
    fn test_fill_rect() {
        let mut layer = Layer::new(0, 200, 200, &Color::from_pixel(0));
        let pix = 0xffff0000;

        fill_rect(
            &mut layer,
            0,
            &Color::from_pixel(pix),
            Blendmode::Normal,
            &Rectangle::new(1, 1, 198, 198),
        );

        assert_eq!(layer.pixel_at(100, 0), 0);

        assert_eq!(layer.pixel_at(0, 1), 0);
        assert_eq!(layer.pixel_at(1, 1), pix);
        assert_eq!(layer.pixel_at(198, 1), pix);
        assert_eq!(layer.pixel_at(199, 1), 0);

        assert_eq!(layer.pixel_at(0, 198), 0);
        assert_eq!(layer.pixel_at(1, 198), pix);
        assert_eq!(layer.pixel_at(198, 198), pix);
        assert_eq!(layer.pixel_at(199, 198), 0);
        assert_eq!(layer.pixel_at(1, 199), 0);
    }

    #[test]
    fn test_draw_brush_dab() {
        let mut layer = Layer::new(0, 128, 128, &Color::TRANSPARENT);
        let brush = BrushMask::new_round_pixel(4, 1.0);
        // Shape should look like this:
        // 0110
        // 1111
        // 1111
        // 0110

        // Dab right at the intersection of four tiles
        draw_brush_dab(
            &mut layer,
            0,
            62,
            62,
            &brush,
            &Color::rgb8(255, 255, 255),
            Blendmode::Normal,
        );

        let pix = 0xffffffffu32;
        assert_eq!(layer.pixel_at(62, 62), 0);
        assert_eq!(layer.pixel_at(63, 62), pix);
        assert_eq!(layer.pixel_at(64, 62), pix);
        assert_eq!(layer.pixel_at(65, 62), 0);

        assert_eq!(layer.pixel_at(62, 63), pix);
        assert_eq!(layer.pixel_at(63, 63), pix);
        assert_eq!(layer.pixel_at(64, 63), pix);
        assert_eq!(layer.pixel_at(65, 63), pix);

        assert_eq!(layer.pixel_at(62, 64), pix);
        assert_eq!(layer.pixel_at(63, 64), pix);
        assert_eq!(layer.pixel_at(64, 64), pix);
        assert_eq!(layer.pixel_at(65, 64), pix);

        assert_eq!(layer.pixel_at(62, 65), 0);
        assert_eq!(layer.pixel_at(63, 65), pix);
        assert_eq!(layer.pixel_at(64, 65), pix);
        assert_eq!(layer.pixel_at(65, 65), 0);
    }

    #[test]
    fn test_layer_merge() {
        let mut btm = Layer::new(0, 128, 128, &Color::rgb8(0, 0, 0));
        let mut top = Layer::new(0, 128, 128, &Color::rgb8(255, 0, 0));
        top.opacity = 0.5;

        merge(&mut btm, &top);

        assert_eq!(btm.pixel_at(0, 0), Color::rgb8(127, 0, 0).as_pixel());
    }
}

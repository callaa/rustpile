use super::aoe::{AoE, TileMap};
use super::rectiter::RectIterator;
use super::tile::{Tile, TILE_SIZE, TILE_SIZEI};
use super::{rasterop, Blendmode, BrushMask, Color, Layer, LayerID, Pixel, Rectangle, UserID};

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

    let pixels = [color.as_pixel(); TILE_SIZE as usize];
    let can_paint = mode.can_increase_opacity();
    let can_erase = mode.can_decrease_opacity();

    for (i, j, tile) in layer.tile_rect_mut(&r) {
        if !can_paint && *tile == Tile::Blank {
            continue;
        }

        let x = i * TILE_SIZEI;
        let y = j * TILE_SIZEI;
        let subrect = Rectangle::new(x, y, TILE_SIZEI, TILE_SIZEI)
            .intersected(&r)
            .unwrap()
            .offset(-x, -y);
        for row in tile.rect_iter_mut(user, &subrect, can_erase) {
            rasterop::pixel_blend(row, &pixels, 0xff, mode);
        }
    }

    r.into()
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

    let colorpix = color.as_pixel();
    let can_paint = mode.can_increase_opacity();
    let can_erase = mode.can_decrease_opacity();

    for (i, j, tile) in layer.tile_rect_mut(&rect) {
        if !can_paint && *tile == Tile::Blank {
            continue;
        }

        let x0 = i * TILE_SIZEI;
        let y0 = j * TILE_SIZEI;
        let subrect = Rectangle::new(x0, y0, TILE_SIZEI, TILE_SIZEI)
            .intersected(&rect)
            .unwrap();
        let tilerect = subrect.offset(-x0, -y0);
        let maskrect = rect.intersected(&subrect).unwrap().offset(-x, -y);

        for (destrow, maskrow) in
            tile.rect_iter_mut(user, &tilerect, can_erase)
                .zip(RectIterator::from_rectangle(
                    &mask.mask,
                    mask.diameter as usize,
                    &maskrect,
                ))
        {
            rasterop::mask_blend(destrow, colorpix, maskrow, mode);
        }
    }

    rect.into()
}

/// Draw an image onto the layer
///
/// The given image must be in premultiplied ARGB format.
/// The image will be drawn onto the given rectangle. The width and height
/// of the rectangle must match the image dimensions. The rectangle may be
/// outside the layer boundaries; it will be cropped as needed.
pub fn draw_image(
    layer: &mut Layer,
    user: UserID,
    image: &[Pixel],
    rect: &Rectangle,
    opacity: f32,
    blendmode: Blendmode,
) -> AoE {
    assert_eq!(image.len(), (rect.w * rect.h) as usize);
    let destrect = match rect.cropped(layer.width(), layer.height()) {
        Some(r) => r,
        None => return AoE::Nothing,
    };

    let o = (opacity * 255.0) as u8;
    let can_paint = blendmode.can_increase_opacity();
    let can_erase = blendmode.can_decrease_opacity();

    for (i, j, tile) in layer.tile_rect_mut(&destrect) {
        if !can_paint && *tile == Tile::Blank {
            continue;
        }

        let x0 = i * TILE_SIZEI;
        let y0 = j * TILE_SIZEI;
        let subrect = Rectangle::new(x0, y0, TILE_SIZEI, TILE_SIZEI)
            .intersected(&destrect)
            .unwrap();
        let tilerect = subrect.offset(-x0, -y0);
        let srcrect = rect.intersected(&subrect).unwrap().offset(-rect.x, -rect.y);

        for (destrow, imagerow) in
            tile.rect_iter_mut(user, &tilerect, can_erase)
                .zip(RectIterator::from_rectangle(
                    &image,
                    rect.w as usize,
                    &srcrect,
                ))
        {
            rasterop::pixel_blend(destrow, imagerow, o, blendmode);
        }
    }

    destrect.into()
}

/// Replace a tile or a stretch of tiles.
/// This is typically used to set the initial canvas content
/// at the start of a session.
pub fn put_tile(
    layer: &mut Layer,
    sublayer: LayerID,
    col: u32,
    row: u32,
    repeat: u32,
    tile: &Tile,
) -> AoE {
    if sublayer != 0 {
        return put_tile(
            layer.get_or_create_sublayer(sublayer),
            0,
            col,
            row,
            repeat,
            &tile,
        );
    }

    let xtiles = Tile::div_up(layer.width());
    let tilevec = layer.tilevec_mut();
    if tilevec.is_empty() {
        return AoE::Nothing;
    }

    let start = (row * xtiles + col) as usize;

    if start >= tilevec.len() {
        return AoE::Nothing;
    }

    let end = (tilevec.len() - 1).min(start + repeat as usize);
    for t in &mut tilevec[start..=end] {
        *t = tile.clone();
    }

    if end > start {
        let mut aoe = TileMap::new(xtiles, Tile::div_up(layer.height()));
        aoe.tiles[start..=end].set_all(true);
        aoe.into()
    } else {
        Rectangle::tile(col as i32, row as i32, xtiles as i32).into()
    }
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
        .iter_mut()
        .zip(source_tiles.iter())
        .for_each(|(d, s)| d.merge(s, source_layer.opacity, source_layer.blendmode));

    if source_layer.is_visible() {
        source_layer.nonblank_tilemap().into()
    } else {
        AoE::Nothing
    }
}

/// Merge a sublayer
pub fn merge_sublayer(layer: &mut Layer, sublayer_id: LayerID) -> AoE {
    if let Some(sublayer) = layer.take_sublayer(sublayer_id) {
        merge(layer, &sublayer)
    } else {
        AoE::Nothing
    }
}

/// Remove a sublayer without merging it
pub fn remove_sublayer(layer: &mut Layer, sublayer_id: LayerID) -> AoE {
    if let Some(sublayer) = layer.take_sublayer(sublayer_id) {
        sublayer.nonblank_tilemap().into()
    } else {
        AoE::Nothing
    }
}

pub fn change_attributes(
    layer: &mut Layer,
    sublayer: LayerID,
    opacity: f32,
    blend: Blendmode,
    censored: bool,
    fixed: bool,
) -> AoE {
    if sublayer != 0 {
        let sl = layer.get_or_create_sublayer(sublayer);
        sl.blendmode = blend;
        sl.opacity = opacity;

        sl.nonblank_tilemap().into()
    } else {
        layer.blendmode = blend;
        layer.opacity = opacity;
        layer.censored = censored;
        layer.fixed = fixed;

        layer.nonblank_tilemap().into()
    }
}

#[cfg(test)]
mod tests {
    use super::super::color::{WHITE_PIXEL, ZERO_PIXEL};
    use super::super::BrushMask;
    use super::*;

    #[test]
    fn test_fill_rect() {
        let mut layer = Layer::new(0, 200, 200, &Color::TRANSPARENT);

        fill_rect(
            &mut layer,
            0,
            &Color::from_pixel(WHITE_PIXEL),
            Blendmode::Normal,
            &Rectangle::new(1, 1, 198, 198),
        );

        assert_eq!(layer.pixel_at(100, 0), ZERO_PIXEL);

        assert_eq!(layer.pixel_at(0, 1), ZERO_PIXEL);
        assert_eq!(layer.pixel_at(1, 1), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(198, 1), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(199, 1), ZERO_PIXEL);

        assert_eq!(layer.pixel_at(0, 198), ZERO_PIXEL);
        assert_eq!(layer.pixel_at(1, 198), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(198, 198), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(199, 198), ZERO_PIXEL);
        assert_eq!(layer.pixel_at(1, 199), ZERO_PIXEL);
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

        assert_eq!(layer.pixel_at(62, 62), ZERO_PIXEL);
        assert_eq!(layer.pixel_at(63, 62), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(64, 62), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(65, 62), ZERO_PIXEL);

        assert_eq!(layer.pixel_at(62, 63), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(63, 63), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(64, 63), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(65, 63), WHITE_PIXEL);

        assert_eq!(layer.pixel_at(62, 64), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(63, 64), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(64, 64), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(65, 64), WHITE_PIXEL);

        assert_eq!(layer.pixel_at(62, 65), ZERO_PIXEL);
        assert_eq!(layer.pixel_at(63, 65), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(64, 65), WHITE_PIXEL);
        assert_eq!(layer.pixel_at(65, 65), ZERO_PIXEL);
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

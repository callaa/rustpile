use crate::paint::{
    editlayer, AoE, Blendmode, BrushMask, ClassicBrushCache, Color, Layer, LayerID, UserID,
};
use crate::protocol::message::{DrawDabsClassicMessage, DrawDabsPixelMessage};

use std::convert::TryFrom;

pub fn drawdabs_classic(
    layer: &mut Layer,
    user: UserID,
    dabs: &DrawDabsClassicMessage,
    cache: &mut ClassicBrushCache,
) -> AoE {
    let mode = Blendmode::try_from(dabs.mode).unwrap_or(Blendmode::Normal);
    let mut color = Color::from_argb32(dabs.color);

    let aoe = if color.a > 0.0 {
        // If alpha is given, these dabs will be drawn in indirect mode
        let sublayer = layer.get_or_create_sublayer(user as LayerID);
        sublayer.opacity = color.a;
        sublayer.blendmode = mode;
        color.a = 1.0;
        drawdabs_classic_draw(sublayer, user, color, Blendmode::Normal, &dabs, cache)
    } else {
        color.a = 1.0; // needed because as_pixel returns premultiplied pixel values
        drawdabs_classic_draw(layer, user, color, mode, &dabs, cache)
    };

    if mode.can_decrease_opacity() {
        layer.optimize(&aoe);
    }

    aoe
}

fn drawdabs_classic_draw(
    layer: &mut Layer,
    user: UserID,
    color: Color,
    mode: Blendmode,
    dabs: &DrawDabsClassicMessage,
    cache: &mut ClassicBrushCache,
) -> AoE {
    let mut last_x = dabs.x;
    let mut last_y = dabs.y;
    let mut aoe = AoE::Nothing;
    for dab in dabs.dabs.iter() {
        let x = last_x + dab.x as i32;
        let y = last_y + dab.y as i32;

        let (mx, my, mask) = BrushMask::new_gimp_style_v2(
            x as f32 / 4.0,
            y as f32 / 4.0,
            dab.size as f32 / 256.0,
            dab.hardness as f32 / 255.0,
            dab.opacity as f32 / 255.0,
            cache,
        );
        aoe = aoe.merge(editlayer::draw_brush_dab(
            layer, user, mx, my, &mask, &color, mode,
        ));

        last_x = x;
        last_y = y;
    }

    aoe
}

pub fn drawdabs_pixel(
    layer: &mut Layer,
    user: UserID,
    dabs: &DrawDabsPixelMessage,
    square: bool,
) -> AoE {
    let mode = Blendmode::try_from(dabs.mode).unwrap_or(Blendmode::Normal);
    let mut color = Color::from_argb32(dabs.color);

    let aoe = if color.a > 0.0 {
        // If alpha is given, these dabs will be drawn in indirect mode
        let sublayer = layer.get_or_create_sublayer(user as LayerID);
        sublayer.opacity = color.a;
        sublayer.blendmode = mode;
        color.a = 1.0;
        drawdabs_pixel_draw(sublayer, user, color, Blendmode::Normal, &dabs, square)
    } else {
        color.a = 1.0; // needed because as_pixel returns premultiplied pixel values
        drawdabs_pixel_draw(layer, user, color, mode, &dabs, square)
    };

    if mode.can_decrease_opacity() {
        layer.optimize(&aoe);
    }

    aoe
}

fn drawdabs_pixel_draw(
    layer: &mut Layer,
    user: UserID,
    color: Color,
    mode: Blendmode,
    dabs: &DrawDabsPixelMessage,
    square: bool,
) -> AoE {
    let mut mask = BrushMask {
        diameter: 0,
        mask: Vec::new(),
    };

    let mut last_x = dabs.x;
    let mut last_y = dabs.y;
    let mut last_size = 0;
    let mut last_opacity = 0;
    let mut aoe = AoE::Nothing;

    for dab in dabs.dabs.iter() {
        let x = last_x + dab.x as i32;
        let y = last_y + dab.y as i32;

        if dab.size != last_size || dab.opacity != last_opacity {
            last_size = dab.size;
            last_opacity = dab.opacity;
            mask = if square {
                BrushMask::new_square_pixel(dab.size as u32, dab.opacity as f32 / 255.0)
            } else {
                BrushMask::new_round_pixel(dab.size as u32, dab.opacity as f32 / 255.0)
            };
        }

        let offset = dab.size as i32 / 2;
        aoe = aoe.merge(editlayer::draw_brush_dab(
            layer,
            user,
            x - offset,
            y - offset,
            &mask,
            &color,
            mode,
        ));

        last_x = x;
        last_y = y;
    }

    aoe
}

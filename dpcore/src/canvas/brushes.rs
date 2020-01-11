use crate::paint::{editlayer, Blendmode, BrushMask, Color, Layer, LayerID, UserID};
use crate::protocol::message::DrawDabsClassicMessage;

use std::convert::TryFrom;

pub fn drawdabs_classic(layer: &mut Layer, user: UserID, dabs: &DrawDabsClassicMessage) {
    let mode = Blendmode::try_from(dabs.mode).unwrap_or(Blendmode::Normal);
    let mut color = Color::from_argb32(dabs.color);

    if color.a > 0.0 {
        // If alpha is given, these dabs will be drawn in indirect mode
        let sublayer = layer.get_or_create_sublayer(user as LayerID);
        sublayer.opacity = color.a;
        sublayer.blendmode = mode;
        color.a = 1.0;
        drawdabs_classic_draw(sublayer, user, color, Blendmode::Normal, &dabs);
    } else {
        color.a = 1.0; // needed because as_pixel returns premultiplied pixel values
        drawdabs_classic_draw(layer, user, color, mode, &dabs);
    }
    // TODO return AoE
}

fn drawdabs_classic_draw(
    layer: &mut Layer,
    user: UserID,
    color: Color,
    mode: Blendmode,
    dabs: &DrawDabsClassicMessage,
) {
    let mut last_x = dabs.x;
    let mut last_y = dabs.y;
    for dab in dabs.dabs.iter() {
        let x = last_x + dab.x as i32;
        let y = last_y + dab.y as i32;

        let (mx, my, mask) = BrushMask::new_gimp_style(
            x as f32 / 4.0,
            y as f32 / 4.0,
            dab.size as f32 / 256.0,
            dab.hardness as f32 / 255.0,
            dab.opacity as f32 / 255.0,
        );
        editlayer::draw_brush_dab(layer, user, mx, my, &mask, &color, mode);

        last_x = x;
        last_y = y;
    }
}

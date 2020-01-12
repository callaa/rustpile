use dpcore::paint::{editlayer, Blendmode, BrushMask, Color, Layer, Rectangle};

mod utils;

fn main() {
    let mut layer = Layer::new(0, 256, 256, &Color::TRANSPARENT);

    // Draw a nice background
    let colors = [
        0xff845ec2, 0xffd65db1, 0xffff6f91, 0xffff9671, 0xffffc75f, 0xfff9f871,
    ];

    for (i, &c) in colors.iter().enumerate() {
        editlayer::fill_rect(
            &mut layer,
            0,
            &Color::from_argb32(c),
            Blendmode::Normal,
            &Rectangle::new(i as i32 * 40, 0, 40, 256),
        );
    }

    // Draw dabs using all the blend modes
    let modes = [
        Blendmode::Erase,
        Blendmode::Normal,
        Blendmode::Multiply,
        Blendmode::Divide,
        Blendmode::Burn,
        Blendmode::Dodge,
        Blendmode::Darken,
        Blendmode::Lighten,
        Blendmode::Subtract,
        Blendmode::Add,
        Blendmode::Recolor,
        Blendmode::Behind,
        Blendmode::ColorErase,
        // Blendmode::Replace,
    ];

    let brush = BrushMask::new_round_pixel(10, 1.0);
    let dabcolor = Color::rgb8(255, 0, 0);

    for (i, &mode) in modes.iter().enumerate() {
        println!("Mode: {:?}", mode);
        for x in (10..250).step_by(8) {
            editlayer::draw_brush_dab(
                &mut layer,
                0,
                x as i32,
                10 + i as i32 * 15,
                &brush,
                &dabcolor,
                mode,
            );
        }
    }
    utils::save_layer(&layer, "example_layer_blend_dabs.png");
}

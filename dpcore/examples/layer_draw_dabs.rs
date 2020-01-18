use dpcore::paint::{editlayer, Blendmode, BrushMask, ClassicBrushCache, Color, Layer};

mod utils;

fn main() {
    let mut layer = Layer::new(0, 256, 256, &Color::rgb8(255, 255, 255));
    let black = Color::rgb8(0, 0, 0);

    let mut x = 10;
    let mut w = 1i32;

    let mut cache = ClassicBrushCache::new();

    while x < layer.width() as i32 - w {
        let brush = BrushMask::new_square_pixel(w as u32, 1.0);
        editlayer::draw_brush_dab(
            &mut layer,
            0,
            x - w / 2,
            30 - w / 2,
            &brush,
            &black,
            Blendmode::Normal,
        );

        let brush = BrushMask::new_round_pixel(w as u32, 1.0);
        editlayer::draw_brush_dab(
            &mut layer,
            0,
            x - w / 2,
            60 - w / 2 as i32,
            &brush,
            &black,
            Blendmode::Normal,
        );

        let (bx, by, brush) =
            BrushMask::new_gimp_style(x as f32, 90.0, w as f32, 0.0, 1.0, &mut cache);
        editlayer::draw_brush_dab(&mut layer, 0, bx, by, &brush, &black, Blendmode::Normal);

        let (bx, by, brush) =
            BrushMask::new_gimp_style(x as f32, 120.0, w as f32, 1.0, 1.0, &mut cache);
        editlayer::draw_brush_dab(&mut layer, 0, bx, by, &brush, &black, Blendmode::Normal);

        w += 1;
        x += w + 2;
    }

    utils::save_layer(&layer, "example_layer_draw_dabs.png");
}

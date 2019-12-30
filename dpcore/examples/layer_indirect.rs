use dpcore::paint::{editlayer, Blendmode, BrushMask, Color, Layer};

mod utils;

fn brush_stroke(layer: &mut Layer, y: i32) {
    let black = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    for x in (10..246).step_by(5) {
        let w = 16 + ((x as f32 / 40.0 * 3.14).sin() * 15.0) as i32;
        let brush = BrushMask::new_round_pixel(w as u32, 0.4);
        editlayer::draw_brush_dab(
            layer,
            0,
            x - w / 2,
            y - w / 2,
            &brush,
            &black,
            Blendmode::Normal,
        );
    }
}

fn main() {
    let mut layer = Layer::new(0, 256, 256, &Color::rgb8(255, 255, 255));

    // The brush stroke drawn in direct mode
    brush_stroke(&mut layer, 60);

    // Indirect mode using a sublayer
    let mut sublayer = layer.get_or_create_sublayer(1);
    sublayer.opacity = 0.5;
    brush_stroke(&mut sublayer, 120);
    editlayer::merge_sublayer(&mut layer, 1);

    utils::save_layer(&layer, "example_layer_indirect.png");
}

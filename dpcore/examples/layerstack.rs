use dpcore::paint::tile::Tile;
use dpcore::paint::layerstack::{LayerStack, LayerInsertion, LayerFill};
use dpcore::paint::{editlayer, Blendmode, BrushMask, Color, Layer};

mod utils;

fn brush_stroke(layer: &mut Layer, y: i32, color: &Color) {
    for x in (10..246).step_by(5) {
        let w = 16 + ((x as f32 / 40.0 * 3.14).sin() * 15.0) as i32;
        let brush = BrushMask::new_round_pixel(w as u32, 0.4);
        editlayer::draw_brush_dab(
            layer,
            0,
            x - w / 2,
            y - w / 2,
            &brush,
            &color,
            Blendmode::Normal,
        );
    }
}

fn main() {
    let mut layerstack = LayerStack::new(256, 256);
    layerstack.background = Tile::new_solid(&Color::rgb8(255, 255, 255), 0);
    layerstack.add_layer(1, LayerFill::Solid(Color::TRANSPARENT), LayerInsertion::Top);
    layerstack.add_layer(2, LayerFill::Solid(Color::TRANSPARENT), LayerInsertion::Top);

    layerstack.get_layer_mut(2).unwrap().opacity = 0.5;

    brush_stroke(
        layerstack.get_layer_mut(1).unwrap(),
        60,
        &Color::rgb8(255, 0, 0),
    );
    brush_stroke(
        layerstack.get_layer_mut(2).unwrap(),
        80,
        &Color::rgb8(0, 255, 0),
    );

    utils::save_layerstack(&layerstack, "example_layerstack.png");
}

use dpcore::paint::{editlayer, Blendmode, BrushMask, ClassicBrushCache, Color, Layer};

mod utils;

fn main() {
    let mut cache = ClassicBrushCache::new();
    let mut layer = Layer::new(0, 512, 256, &Color::rgb8(255, 255, 255));

    draw_stroke(&mut layer, 10.5, 10.0, 500.5, 12.0, 1.0, 1.0, &mut cache);
    draw_stroke(&mut layer, 10.5, 13.0, 500.5, 15.0, 1.0, 0.0, &mut cache);

    draw_stroke(&mut layer, 10.5, 30.0, 500.5, 32.0, 2.0, 1.0, &mut cache);
    draw_stroke(&mut layer, 10.5, 35.0, 500.5, 37.0, 2.0, 0.0, &mut cache);

    draw_stroke(&mut layer, 10.5, 50.0, 500.5, 52.0, 4.0, 1.0, &mut cache);
    draw_stroke(&mut layer, 10.5, 60.0, 500.5, 67.0, 4.0, 0.0, &mut cache);

    for d in (1..10).step_by(2) {
        let dia = d as f32;
        let offset = dia * 5.0;
        for xoff in 0..=4 {
            draw_dab(
                &mut layer,
                10.0 + xoff as f32 / 4.0 + (dia * 2.0 * xoff as f32),
                100.5 + offset,
                dia,
                1.0,
                &mut cache,
            );
        }
        for yoff in 0..=4 {
            draw_dab(
                &mut layer,
                10.5 + (dia * 2.0 * (6 + yoff) as f32),
                100.0 + offset + yoff as f32 / 4.0,
                dia,
                1.0,
                &mut cache,
            );
        }
    }

    utils::save_layer(&layer, "example_gimp_style_brush.png");
}

fn draw_stroke(
    layer: &mut Layer,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    dia: f32,
    hardness: f32,
    cache: &mut ClassicBrushCache,
) {
    let mut x = x1;
    let mut y = y1;
    let dist = (x2 - x1).hypot(y2 - y1);
    let dx = (x2 - x1) / dist;
    let dy = (y2 - y1) / dist;
    let spacing = (dia * 0.15).max(1.0);

    let mut i = 0.0;
    while i <= dist {
        draw_dab(layer, x, y, dia, hardness, cache);

        x += dx * spacing;
        y += dy * spacing;
        i += spacing;
    }
}

fn draw_dab(
    layer: &mut Layer,
    x: f32,
    y: f32,
    dia: f32,
    hardness: f32,
    cache: &mut ClassicBrushCache,
) {
    let (bx, by, brush) = BrushMask::new_gimp_style_v2(x, y, dia, hardness, 1.0, cache);
    editlayer::draw_brush_dab(
        layer,
        0,
        bx,
        by,
        &brush,
        &Color::rgb8(0, 0, 0),
        Blendmode::Normal,
    );
}

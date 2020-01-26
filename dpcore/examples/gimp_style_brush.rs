use dpcore::paint::{editlayer, Blendmode, BrushMask, ClassicBrushCache, Color, Layer};

mod utils;

fn main() {
    let mut cache = ClassicBrushCache::new();
    let mut layer = Layer::new(0, 512, 256, &Color::rgb8(255, 255, 255));

    draw_stroke(&mut layer, 10.0, 10.0, 500.0, 12.0, 1.0, &mut cache);
    draw_stroke(&mut layer, 10.0, 30.0, 500.0, 32.0, 2.0, &mut cache);
    draw_stroke(&mut layer, 10.0, 50.0, 500.0, 52.0, 4.0, &mut cache);

    for d in (1..10).step_by(2) {
        let dia = d as f32;
        let offset = dia * 5.0;
        draw_dab(&mut layer, 10.0, 100.5 + offset, dia, &mut cache);
        draw_dab(
            &mut layer,
            10.25 + dia * 2.0,
            100.5 + offset,
            dia,
            &mut cache,
        );
        draw_dab(
            &mut layer,
            10.5 + dia * 4.0,
            100.5 + offset,
            dia,
            &mut cache,
        );
        draw_dab(
            &mut layer,
            10.75 + dia * 6.0,
            100.5 + offset,
            dia,
            &mut cache,
        );

        draw_dab(
            &mut layer,
            10.5 + dia * 10.0,
            100.0 + offset,
            dia,
            &mut cache,
        );
        draw_dab(
            &mut layer,
            10.5 + dia * 12.0,
            100.25 + offset,
            dia,
            &mut cache,
        );
        draw_dab(
            &mut layer,
            10.5 + dia * 14.0,
            100.5 + offset,
            dia,
            &mut cache,
        );
        draw_dab(
            &mut layer,
            10.5 + dia * 16.0,
            100.75 + offset,
            dia,
            &mut cache,
        );
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
        draw_dab(layer, x, y, dia, cache);

        x += dx * spacing;
        y += dy * spacing;
        i += spacing;
    }
}

fn draw_dab(layer: &mut Layer, x: f32, y: f32, dia: f32, cache: &mut ClassicBrushCache) {
    let (bx, by, brush) = BrushMask::new_gimp_style(x, y, dia, 0.9, 1.0, cache);
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

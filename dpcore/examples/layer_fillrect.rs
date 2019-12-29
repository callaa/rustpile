use dpcore::paint::{editlayer, Blendmode, Color, Layer, Rectangle};

mod utils;

fn main() {
    let colors = [
        0xff845ec2, 0xffd65db1, 0xffff6f91, 0xffff9671, 0xffffc75f, 0xfff9f871,
    ];

    let mut layer = Layer::new(0, 256, 256, &Color::rgb8(255, 255, 255));

    // Draw some vertical bars
    for (i, &c) in colors.iter().enumerate() {
        editlayer::fill_rect(
            &mut layer,
            0,
            &Color::from_pixel(c),
            Blendmode::Normal,
            &Rectangle::new(10 + (i as i32 * 40), 10, 30, 236),
        );
    }

    // Draw horizontal bars with different blending modes
    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.25,
        }, // Erase mode only uses the alpha channel
        Blendmode::Erase,
        &Rectangle::new(5, 15, 246, 10),
    );

    utils::save_layer(&layer, "example_layer_fillrect.png");
}

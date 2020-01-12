use dpcore::paint::{editlayer, Blendmode, Color, Layer, Rectangle};

mod utils;

fn main() {
    let colors = [
        0xff845ec2, 0xffd65db1, 0xffff6f91, 0xffff9671, 0xffffc75f, 0xfff9f871,
    ];

    let mut layer = Layer::new(0, 256, 256, &Color::TRANSPARENT);

    // A background almost that leaves a transparent skirt
    editlayer::fill_rect(
        &mut layer,
        0,
        &Color::rgb8(255, 255, 255),
        Blendmode::Normal,
        &Rectangle::new(0, 0, 256, 210),
    );

    // Draw some vertical bars
    for (i, &c) in colors.iter().enumerate() {
        editlayer::fill_rect(
            &mut layer,
            0,
            &Color::from_argb32(c),
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

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 0.5,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Multiply,
        &Rectangle::new(5, 35, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 0.0,
            g: 0.5,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Divide,
        &Rectangle::new(5, 55, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 0.5,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Darken,
        &Rectangle::new(5, 75, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Lighten,
        &Rectangle::new(5, 95, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Dodge,
        &Rectangle::new(5, 115, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Burn,
        &Rectangle::new(5, 135, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Add,
        &Rectangle::new(5, 155, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Subtract,
        &Rectangle::new(5, 175, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::ColorErase,
        &Rectangle::new(5, 195, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Recolor,
        &Rectangle::new(5, 215, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 0.8,
        },
        Blendmode::Behind,
        &Rectangle::new(5, 235, 246, 10),
    );

    editlayer::fill_rect(
        &mut layer,
        0,
        &Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        },
        Blendmode::Replace,
        &Rectangle::new(5, 250, 246, 5),
    );

    utils::save_layer(&layer, "example_layer_fillrect.png");
}

use dpcore::paint::{editlayer, Blendmode, Color, Layer, Rectangle};

mod utils;

fn main() {
    let mut layer = Layer::new(0, 256, 256, &Color::rgb8(255, 255, 255));

    let (image, w, h) = utils::load_image("testdata/logo.png");

    editlayer::draw_image(
        &mut layer,
        0,
        &image,
        &Rectangle::new(256 - w * 2 / 3, 256 - h * 2 / 3, w, h),
        1.0,
        Blendmode::Replace,
    );

    editlayer::draw_image(
        &mut layer,
        0,
        &image,
        &Rectangle::new(256 / 2 - w / 2, 256 / 2 - h / 2, w, h),
        0.5,
        Blendmode::Normal,
    );

    editlayer::draw_image(
        &mut layer,
        0,
        &image,
        &Rectangle::new(-w / 2, -h / 2, w, h),
        1.0,
        Blendmode::Normal,
    );

    utils::save_layer(&layer, "example_layer_drawimage.png");
}

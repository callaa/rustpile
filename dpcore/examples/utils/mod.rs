use dpcore::paint::tile::{Tile, TileData, TILE_SIZE};
use dpcore::paint::{Layer, LayerStack};
use dpcore::paint::color::*;
use image;
use image::{ImageBuffer, RgbaImage};

fn copy_tile_to(dest: &mut Vec<u8>, stride: u32, tile: &TileData, tx: u32, ty: u32) {
    let mut dest_offset = ty * TILE_SIZE * stride * 4 + tx * TILE_SIZE * 4;

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let px = tile.pixels[(y * TILE_SIZE + x) as usize];
            dest[(dest_offset + x * 4 + 0) as usize] = px[RED_CHANNEL];
            dest[(dest_offset + x * 4 + 1) as usize] = px[GREEN_CHANNEL];
            dest[(dest_offset + x * 4 + 2) as usize] = px[BLUE_CHANNEL];
            dest[(dest_offset + x * 4 + 3) as usize] = px[ALPHA_CHANNEL];
        }
        dest_offset += stride * 4;
    }
}

fn u8_mult(a: u8, b: u8) -> u8 {
    let c = a as u32 * b as u32 + 0x80u32;
    (((c >> 8) + c) >> 8) as u8
}

#[allow(dead_code)]
pub fn load_image(filename: &str) -> (Vec<Pixel>, i32, i32) {
    let img = image::open(filename).expect("couldn't load image");
    let rgba = img.as_rgba8().unwrap();

    let mut argb_data = Vec::<Pixel>::with_capacity((rgba.width() * rgba.height()) as usize);

    for p in rgba.pixels() {
        argb_data.push([
            p.0[3],
            u8_mult(p.0[3], p.0[0]),
            u8_mult(p.0[3], p.0[1]),
            u8_mult(p.0[3], p.0[2]),
        ]);
    }

    (argb_data, rgba.width() as i32, rgba.height() as i32)
}

#[allow(dead_code)]
pub fn save_layer(layer: &Layer, filename: &str) {
    // This is just to make copy_tile_to simpler:
    assert!(layer.width() % TILE_SIZE == 0);
    assert!(layer.height() % TILE_SIZE == 0);

    let mut rgba = vec![0u8; (layer.width() * layer.height() * 4) as usize];

    for ty in 0..(layer.height() / TILE_SIZE) {
        for tx in 0..(layer.width() / TILE_SIZE) {
            if let Tile::Bitmap(td) = layer.tile(tx, ty) {
                copy_tile_to(&mut rgba, layer.width(), td, tx, ty);
            }
        }
    }

    let img: RgbaImage = ImageBuffer::from_vec(layer.width(), layer.height(), rgba).unwrap();

    println!("Writing {}", filename);
    img.save(filename).unwrap();
}

#[allow(dead_code)]
pub fn save_layerstack(layerstack: &LayerStack, filename: &str) {
    // This is just to make copy_tile_to simpler:
    assert!(layerstack.width() % TILE_SIZE == 0);
    assert!(layerstack.height() % TILE_SIZE == 0);

    let mut rgba = vec![0u8; (layerstack.width() * layerstack.height() * 4) as usize];

    for ty in 0..(layerstack.height() / TILE_SIZE) {
        for tx in 0..(layerstack.width() / TILE_SIZE) {
            let td = layerstack.flatten_tile(tx, ty);
            copy_tile_to(&mut rgba, layerstack.width(), &td, tx, ty);
        }
    }

    let img: RgbaImage =
        ImageBuffer::from_vec(layerstack.width(), layerstack.height(), rgba).unwrap();

    println!("Writing {}", filename);
    img.save(filename).unwrap();
}

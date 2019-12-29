use dpcore::paint::tile::{Tile, TileData, TILE_SIZE};
use dpcore::paint::Layer;
use image::{ImageBuffer, RgbaImage};

fn copy_tile_to(dest: &mut Vec<u8>, stride: u32, tile: &TileData, tx: u32, ty: u32) {
    let mut dest_offset = ty * TILE_SIZE * stride * 4 + tx * TILE_SIZE * 4;

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let px = tile.pixels[(y * TILE_SIZE + x) as usize];
            dest[(dest_offset + x * 4 + 0) as usize] = ((px & 0x00ff0000) >> 16) as u8;
            dest[(dest_offset + x * 4 + 1) as usize] = ((px & 0x0000ff00) >> 8) as u8;
            dest[(dest_offset + x * 4 + 2) as usize] = ((px & 0x000000ff) >> 0) as u8;
            dest[(dest_offset + x * 4 + 3) as usize] = ((px & 0xff000000) >> 24) as u8;
        }
        dest_offset += stride * 4;
    }
}

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

use criterion::{criterion_group, criterion_main, Criterion};
use dpcore::paint::{rasterop, Blendmode, BrushMask};

fn mask_blend(mask: &[u8], mode: Blendmode) {
    let mut base = [0x80_808080_u32;64*64];
    rasterop::mask_blend(&mut base, 0xff_ffffff, mask, mode);
}

fn pixel_blend(over: &[u32], mode: Blendmode) {
    let mut base = [0x80_808080_u32;64*64];
    rasterop::pixel_blend(&mut base, over, 128, mode);
}

fn mask_blend_benchmark(c: &mut Criterion) {
    let mask = BrushMask::new_round_pixel(64, 0.5);

    c.bench_function("mask normal", |b| b.iter(|| mask_blend(&mask.mask, Blendmode::Normal)));
    c.bench_function("mask erase", |b| b.iter(|| mask_blend(&mask.mask, Blendmode::Erase)));
    c.bench_function("mask multiply", |b| b.iter(|| mask_blend(&mask.mask, Blendmode::Multiply)));
    c.bench_function("mask behind", |b| b.iter(|| mask_blend(&mask.mask, Blendmode::Behind)));
    c.bench_function("mask colorerase", |b| b.iter(|| mask_blend(&mask.mask, Blendmode::ColorErase)));
}

fn pixel_blend_benchmark(c: &mut Criterion) {
    let over = vec![0xff_ffffffu32;64*64];

    c.bench_function("pixel normal", |b| b.iter(|| pixel_blend(&over, Blendmode::Normal)));
    c.bench_function("pixel erase", |b| b.iter(|| pixel_blend(&over, Blendmode::Erase)));
    c.bench_function("pixel multiply", |b| b.iter(|| pixel_blend(&over, Blendmode::Multiply)));
    c.bench_function("pixel behind", |b| b.iter(|| pixel_blend(&over, Blendmode::Behind)));
    c.bench_function("pixel colorerase", |b| b.iter(|| pixel_blend(&over, Blendmode::ColorErase)));
}

criterion_group!(benches, mask_blend_benchmark, pixel_blend_benchmark);
criterion_main!(benches);

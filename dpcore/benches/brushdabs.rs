use criterion::{criterion_group, criterion_main, Criterion};
use dpcore::paint::{BrushMask, ClassicBrushCache};

fn gimp_style_dab_benchmark(c: &mut Criterion) {
    c.bench_function("small GIMP style dab", |b| {
        let mut cache = ClassicBrushCache::new(); // note: this produces a single outlier when the LUT is generated
        b.iter(|| {
            BrushMask::new_gimp_style(0.0, 0.0, 15.0, 0.5, 1.0, &mut cache);
        })
    });

    c.bench_function("big GIMP style dab", |b| {
        let mut cache = ClassicBrushCache::new(); // note: this produces a single outlier when the LUT is generated
        b.iter(|| {
            BrushMask::new_gimp_style(0.0, 0.0, 30.0, 0.5, 1.0, &mut cache);
        })
    });
}

fn round_pixel_dab_benchmark(c: &mut Criterion) {
    c.bench_function("round pixel dab", |b| {
        b.iter(|| {
            BrushMask::new_round_pixel(15, 1.0);
        })
    });
}

criterion_group!(benches, gimp_style_dab_benchmark, round_pixel_dab_benchmark);
criterion_main!(benches);

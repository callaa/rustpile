use super::{Blendmode, Color};

pub fn pixel_blend(base: &mut [u32], over: &[u32], opacity: u8, mode: Blendmode) {
    match mode {
        Blendmode::Normal => alpha_pixel_blend(base, over, opacity),
        Blendmode::Erase => alpha_pixel_erase(base, over, opacity),
        Blendmode::Multiply => pixel_composite(comp_op_multiply, base, over, opacity),
        Blendmode::Divide => pixel_composite(comp_op_divide, base, over, opacity),
        Blendmode::Darken => pixel_composite(comp_op_darken, base, over, opacity),
        Blendmode::Lighten => pixel_composite(comp_op_lighten, base, over, opacity),
        Blendmode::Dodge => pixel_composite(comp_op_dodge, base, over, opacity),
        Blendmode::Burn => pixel_composite(comp_op_burn, base, over, opacity),
        Blendmode::Add => pixel_composite(comp_op_add, base, over, opacity),
        Blendmode::Subtract => pixel_composite(comp_op_subtract, base, over, opacity),
        Blendmode::Recolor => pixel_composite(comp_op_recolor, base, over, opacity),
        Blendmode::Behind => alpha_pixel_under(base, over, opacity),
        Blendmode::ColorErase => pixel_color_erase(base, over, opacity),
        Blendmode::Replace => pixel_replace(base, over, opacity),
    }
}

pub fn mask_blend(base: &mut [u32], color: u32, mask: &[u8], mode: Blendmode) {
    match mode {
        Blendmode::Normal => alpha_mask_blend(base, color, mask),
        Blendmode::Erase => alpha_mask_erase(base, mask),
        Blendmode::Multiply => mask_composite(comp_op_multiply, base, color, mask),
        Blendmode::Divide => mask_composite(comp_op_divide, base, color, mask),
        Blendmode::Darken => mask_composite(comp_op_darken, base, color, mask),
        Blendmode::Lighten => mask_composite(comp_op_lighten, base, color, mask),
        Blendmode::Dodge => mask_composite(comp_op_dodge, base, color, mask),
        Blendmode::Burn => mask_composite(comp_op_burn, base, color, mask),
        Blendmode::Add => mask_composite(comp_op_add, base, color, mask),
        Blendmode::Subtract => mask_composite(comp_op_subtract, base, color, mask),
        Blendmode::Recolor => mask_composite(comp_op_recolor, base, color, mask),
        Blendmode::Behind => alpha_mask_under(base, color, mask),
        Blendmode::ColorErase => mask_color_erase(base, color, mask),
        m => panic!("TODO unimplemented mask blend mode {:?}", m),
    }
}

fn split_channels(c: u32) -> (u32, u32, u32, u32) {
    (
        c >> 24,
        (c & 0x00ff0000) >> 16,
        (c & 0x0000ff00) >> 8,
        c & 0x000000ff,
    )
}

fn get_alpha(c: u32) -> u32 {
    c >> 24
}

fn combine_channels(a: u32, r: u32, g: u32, b: u32) -> u32 {
    debug_assert!(a <= 0xff);
    debug_assert!(r <= 0xff);
    debug_assert!(g <= 0xff);
    debug_assert!(b <= 0xff);
    (a << 24) | (r << 16) | (g << 8) | b
}

fn u8_mult(a: u32, b: u32) -> u32 {
    let c = a * b + 0x80;
    ((c >> 8) + c) >> 8
}

/// Perform a premultiplied alpha blend operation on a slice of 32 bit ARGB pixels
fn alpha_pixel_blend(base: &mut [u32], over: &[u32], opacity: u8) {
    let o = opacity as u32;

    for (dp, sp) in base.into_iter().zip(over.into_iter()) {
        let (mut da, mut dr, mut dg, mut db) = split_channels(*dp);
        let (sa, sr, sg, sb) = split_channels(*sp);
        let a_s = 255 - u8_mult(sa, o);

        da = u8_mult(sa, o) + u8_mult(da, a_s);
        dr = u8_mult(sr, o) + u8_mult(dr, a_s);
        dg = u8_mult(sg, o) + u8_mult(dg, a_s);
        db = u8_mult(sb, o) + u8_mult(db, a_s);

        *dp = combine_channels(da, dr, dg, db);
    }
}

/// Perform a premultiplied alpha blend operation on a slice of 32 bit ARGB pixels
fn alpha_pixel_under(base: &mut [u32], over: &[u32], opacity: u8) {
    let o = opacity as u32;

    for (dp, sp) in base.into_iter().zip(over.into_iter()) {
        let (mut da, mut dr, mut dg, mut db) = split_channels(*dp);
        let (sa, sr, sg, sb) = split_channels(*sp);
        let a_s = u8_mult(255 - da, u8_mult(sa, o));

        da = u8_mult(sa, a_s) + da;
        dr = u8_mult(sr, a_s) + dr;
        dg = u8_mult(sg, a_s) + dg;
        db = u8_mult(sb, a_s) + db;

        *dp = combine_channels(da, dr, dg, db);
    }
}

/// Perform a premultiplied alpha blend on a slice of 32 bit ARGB pixels
/// and a color + alpha mask vector.
fn alpha_mask_under(base: &mut [u32], color: u32, mask: &[u8]) {
    debug_assert!(base.len() == mask.len());
    let (_, cr, cg, cb) = split_channels(color);

    for (dp, &mask) in base.into_iter().zip(mask.into_iter()) {
        let (mut da, mut dr, mut dg, mut db) = split_channels(*dp);
        let m = mask as u32;
        let a = u8_mult(255 - da, m);

        da = da + a;
        dr = u8_mult(cr, a) + dr;
        dg = u8_mult(cg, a) + dg;
        db = u8_mult(cb, a) + db;

        *dp = combine_channels(da, dr, dg, db);
    }
}

fn color_erase(dest: &mut Color, color: &Color) {
    // This algorithm was found from the Gimp's source code:
    // for each channel:
    // a_c = (dest_c - color_c) / (1 - color_c) if dest_c > color_c
    //     (color_c - dest_c) / (color_c) if dest_c < color_c
    //     dest_c if color_c = 0
    //     0 if dest_c = color_c
    // a_a = dest_a
    //
    // dest_a = (1-color_a) + (max(a_r, a_g, a_b) * color_a)
    // dest_c = (dest_c - color_c) / dest_a + color_r
    // dest_a *= a_a

    fn ac(d: f32, c: f32) -> f32 {
        if c < (1.0 / 256.0) {
            d
        } else if d > c {
            (d - c) / (1.0 - c)
        } else if d < c {
            (c - d) / c
        } else {
            0.0
        }
    }

    let a = Color {
        r: ac(dest.r, color.r),
        g: ac(dest.g, color.g),
        b: ac(dest.b, color.b),
        a: dest.a,
    };

    dest.a = (1.0 - color.a) + a.r.max(a.b).max(a.g) * color.a;
    dest.r = (dest.r - color.r) / dest.a + color.r;
    dest.g = (dest.g - color.g) / dest.a + color.g;
    dest.b = (dest.b - color.b) / dest.a + color.b;
    dest.a *= a.a;
}

/// Perform per pixel color erase
fn pixel_color_erase(base: &mut [u32], over: &[u32], opacity: u8) {
    let o = opacity as f32 / 255.0;

    for (dp, sp) in base.into_iter().zip(over.into_iter()) {
        // TODO optimize this?
        let mut dc = Color::from_pixel(*dp);
        let mut sc = Color::from_pixel(*sp);
        sc.a *= o;
        color_erase(&mut dc, &sc);
        *dp = dc.as_pixel();
    }
}

fn mask_color_erase(base: &mut [u32], color: u32, mask: &[u8]) {
    let mut c = Color::from_pixel(color);
    for (dp, &mp) in base.into_iter().zip(mask.into_iter()) {
        // TODO optimize this?
        let mut dc = Color::from_pixel(*dp);
        c.a = mp as f32 / 255.0;
        color_erase(&mut dc, &c);
        *dp = dc.as_pixel();
    }
}

fn pixel_replace(base: &mut [u32], over: &[u32], opacity: u8) {
    let o = opacity as u32;

    for (dp, sp) in base.into_iter().zip(over.into_iter()) {
        let (sa, sr, sg, sb) = split_channels(*sp);

        let da = u8_mult(sa, o);
        let dr = u8_mult(sr, o);
        let dg = u8_mult(sg, o);
        let db = u8_mult(sb, o);

        *dp = combine_channels(da, dr, dg, db);
    }
}

/// Perform a premultiplied alpha blend on a slice of 32 bit ARGB pixels
/// and a color + alpha mask vector.
fn alpha_mask_blend(base: &mut [u32], color: u32, mask: &[u8]) {
    debug_assert!(base.len() == mask.len());
    let (_, cr, cg, cb) = split_channels(color);

    for (dp, &mask) in base.into_iter().zip(mask.into_iter()) {
        let (mut da, mut dr, mut dg, mut db) = split_channels(*dp);
        let m = mask as u32;
        let a = 255 - m;

        da = m + u8_mult(da, a);
        dr = u8_mult(cr, m) + u8_mult(dr, a);
        dg = u8_mult(cg, m) + u8_mult(dg, a);
        db = u8_mult(cb, m) + u8_mult(db, a);

        *dp = combine_channels(da, dr, dg, db);
    }
}

/// Erase alpha channel
fn alpha_mask_erase(base: &mut [u32], mask: &[u8]) {
    debug_assert!(base.len() == mask.len());

    for (dp, &mask) in base.into_iter().zip(mask.into_iter()) {
        let (mut da, mut dr, mut dg, mut db) = split_channels(*dp);
        let m = mask as u32;
        let a = 255 - m;

        da = u8_mult(da, a);
        dr = u8_mult(dr, a);
        dg = u8_mult(dg, a);
        db = u8_mult(db, a);
        *dp = combine_channels(da, dr, dg, db);
    }
}

/// Erase using alpha channel of source pixels
fn alpha_pixel_erase(base: &mut [u32], over: &[u32], opacity: u8) {
    let o = opacity as u32;

    for (dp, sp) in base.into_iter().zip(over.into_iter()) {
        let (mut da, mut dr, mut dg, mut db) = split_channels(*dp);
        let a = 255 - u8_mult(get_alpha(*sp), o);

        da = u8_mult(da, a);
        dr = u8_mult(dr, a);
        dg = u8_mult(dg, a);
        db = u8_mult(db, a);

        *dp = combine_channels(da, dr, dg, db);
    }
}

fn blend(a: f32, b: f32, alpha: f32) -> f32 {
    (a - b) * alpha + b
}

fn comp_op_multiply(a: f32, b: f32) -> f32 {
    a * b
}

fn comp_op_divide(a: f32, b: f32) -> f32 {
    1.0f32.min(a / ((1.0 / 256.0) + b))
}

fn comp_op_darken(a: f32, b: f32) -> f32 {
    return a.min(b);
}

fn comp_op_lighten(a: f32, b: f32) -> f32 {
    return a.max(b);
}

fn comp_op_dodge(a: f32, b: f32) -> f32 {
    return 1.0f32.min(a / (1.001 - b));
}

fn comp_op_burn(a: f32, b: f32) -> f32 {
    return 0.0f32.max(1.0f32.min(1.0 - ((1.0 - a) / (b + 0.001))));
}

fn comp_op_add(a: f32, b: f32) -> f32 {
    return 1.0f32.min(a + b);
}

fn comp_op_subtract(a: f32, b: f32) -> f32 {
    return 0.0f32.max(a - b);
}

fn comp_op_recolor(_: f32, b: f32) -> f32 {
    return b;
}

/// Generic alpha-preserving compositing operations
fn pixel_composite(comp_op: fn(f32, f32) -> f32, base: &mut [u32], over: &[u32], opacity: u8) {
    let of = opacity as f32 / 255.0;
    for (dp, sp) in base.into_iter().zip(over.into_iter()) {
        // TODO optimize this. These operations need non-premultiplied color
        let mut dc = Color::from_pixel(*dp);
        let sc = Color::from_pixel(*sp);

        let alpha = sc.a * of;

        dc.r = blend(comp_op(dc.r, sc.r), dc.r, alpha);
        dc.g = blend(comp_op(dc.g, sc.g), dc.g, alpha);
        dc.b = blend(comp_op(dc.b, sc.b), dc.b, alpha);

        *dp = dc.as_pixel();
    }
}

fn mask_composite(comp_op: fn(f32, f32) -> f32, base: &mut [u32], color: u32, mask: &[u8]) {
    debug_assert!(base.len() == mask.len());
    let c = Color::from_pixel(color);
    for (dp, &mask) in base.into_iter().zip(mask.into_iter()) {
        let mut d = Color::from_pixel(*dp);
        let m = mask as f32 * 255.0;

        d.r = blend(comp_op(d.r, c.r), d.r, m);
        d.g = blend(comp_op(d.g, c.g), d.g, m);
        d.b = blend(comp_op(d.b, c.b), d.b, m);

        *dp = d.as_pixel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_pixel_blend() {
        let mut base = [0xff_ff0000];
        let over = [0x80_008000];

        alpha_pixel_blend(&mut base, &over, 0xff);
        assert_eq!(base, [0xff7f8000]);

        let mut base = [0xff_ff0000];
        alpha_pixel_blend(&mut base, &over, 0x80);
        assert_eq!(base, [0xffbf4000]);
    }

    #[test]
    fn test_alpha_mask_blend() {
        let mut base = [0xff_ff0000, 0xff_ff0000, 0xff_ff0000];
        let mask = [0xff, 0x80, 0x40];

        alpha_mask_blend(&mut base, 0x0000ff00, &mask);
        assert_eq!(base, [0xff_00ff00, 0xff_7f8000, 0xff_bf4000]);
    }

    #[test]
    fn test_alpha_pixel_erase() {
        let mut base = [0xff_ffffff, 0xff_ffffff, 0xff_ffffff];
        let over = [0xff_123456, 0x80_123456, 0x00_123456];

        alpha_pixel_erase(&mut base, &over, 0xff);
        assert_eq!(base, [0x00_000000, 0x7f_7f7f7f, 0xff_ffffff]);
    }
    #[test]
    fn test_alpha_mask_erase() {
        let mut base = [0xff_ffffff, 0xff_ffffff, 0xff_ffffff];
        let mask = [0xff, 0x80, 0x00];

        alpha_mask_erase(&mut base, &mask);
        assert_eq!(base, [0x00_000000, 0x7f_7f7f7f, 0xff_ffffff]);
    }
}

use super::blendmode::Blendmode;

pub fn pixel_blend(base: &mut [u32], over: &[u32], opacity: u8, mode: Blendmode) {
    match mode {
        Blendmode::Normal => alpha_pixel_blend(base, over, opacity),
        Blendmode::Erase => alpha_pixel_erase(base, over, opacity),
        m => panic!("TODO unimplemented pixel blend mode {:?}", m),
    }
}

pub fn mask_blend(base: &mut [u32], color: u32, mask: &[u8], mode: Blendmode) {
    match mode {
        Blendmode::Normal => alpha_mask_blend(base, color, mask),
        Blendmode::Erase => alpha_mask_erase(base, mask),
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

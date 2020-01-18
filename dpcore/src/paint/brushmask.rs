use std::iter::FromIterator;

pub struct BrushMask {
    /// Brush diameter (mask shape is a square)
    pub diameter: u32,

    /// Brush mask (length is diameter^2)
    pub mask: Vec<u8>,
}

pub struct ClassicBrushCache {
    lut: Vec<Vec<f32>>,
}

impl ClassicBrushCache {
    pub fn new() -> ClassicBrushCache {
        ClassicBrushCache {
            lut: vec![Vec::new(); 101],
        }
    }

    fn get_cached_lut(&mut self, hardness: f32) -> &[f32] {
        let h = (hardness * 100.0) as usize;
        if self.lut[h].is_empty() {
            self.lut[h] = make_gimp_style_brush_lut(hardness);
        }
        &self.lut[h]
    }
}

fn square(v: f32) -> f32 {
    v * v
}

impl BrushMask {
    pub fn new_round_pixel(diameter: u32, opacity: f32) -> BrushMask {
        debug_assert!(opacity >= 0.0 && opacity <= 1.0);
        let radius = diameter as f32 / 2.0;
        let rr = square(radius);
        let offset = 0.5_f32;

        let mut mask = vec![0u8; (diameter * diameter) as usize];
        let mut i = 0;
        let op_u8 = (opacity * 255.0) as u8;

        for y in 0..diameter {
            let yy = square(y as f32 - radius + offset);
            for x in 0..diameter {
                let xx = square(x as f32 - radius + offset);
                if (yy + xx) < rr {
                    mask[i] = op_u8;
                }
                i += 1;
            }
        }
        BrushMask { diameter, mask }
    }

    pub fn new_square_pixel(diameter: u32, opacity: f32) -> BrushMask {
        BrushMask {
            diameter,
            mask: vec![(opacity * 255.0) as u8; (diameter * diameter) as usize],
        }
    }

    /// Make a Gimp style subpixel resolution brush mask
    /// Return value include the integer coordinates where the brush mask should be dabbed
    /// TODO this whole thing needs a rewrite, but that will be a protocol change
    pub fn new_gimp_style(
        x: f32,
        y: f32,
        diameter: f32,
        hardness: f32,
        opacity: f32,
        cache: &mut ClassicBrushCache,
    ) -> (i32, i32, BrushMask) {
        let (offset, mask) = if diameter < 16.0 {
            BrushMask::new_gimp_style_highres(diameter / 2.0, hardness, opacity, cache)
        } else {
            BrushMask::new_gimp_style_lowres(diameter / 2.0, hardness, opacity, cache)
        };

        let fx = x.floor();
        let fy = y.floor();

        // this doesn't work right with negative values, but fixing it is a protocol change
        let mut xfrac = x - fx;
        let mut yfrac = y - fy;

        let ix = if xfrac < 0.5 {
            xfrac += 0.5;
            (fx - 1.0 - offset) as i32
        } else {
            xfrac -= 0.5;
            (fx - offset) as i32
        };

        let iy = if yfrac < 0.5 {
            yfrac += 0.5;
            (fy - 1.0 - offset) as i32
        } else {
            yfrac -= 0.5;
            (fy - offset) as i32
        };

        (ix, iy, mask.offset(xfrac, yfrac))
    }

    // Make a high-res Gimp style mask useful for small brushes. Return values includes x/y offset to get the top-left corner
    fn new_gimp_style_highres(
        radius: f32,
        hardness: f32,
        opacity: f32,
        cache: &mut ClassicBrushCache,
    ) -> (f32, BrushMask) {
        let op = opacity * (255.0 / 4.0); // opacity per subsample
        let mut diameter = (radius * 2.0).ceil() as u32 + 2;
        let mut offset = (radius.ceil() - radius) / -2.0;

        if diameter % 2 == 0 {
            diameter += 1;
            offset -= 2.5;
        } else {
            offset -= 1.5;
        }

        let r2 = radius * 2.0;
        let lut = cache.get_cached_lut(hardness);
        let lut_scale = square((LUT_RADIUS - 1.0) / r2);

        let mut mask = vec![0u8; (diameter * diameter) as usize];
        let mut i = 0;

        for y in 0..diameter {
            let yy0 = square(y as f32 * 2.0 - r2 + offset);
            let yy1 = square(y as f32 * 2.0 + 1.0 - r2 + offset);

            for x in 0..diameter {
                let xx0 = square(x as f32 * 2.0 - r2 + offset);
                let xx1 = square(x as f32 * 2.0 + 1.0 - r2 + offset);

                let dist0 = ((xx0 + yy0) * lut_scale) as usize;
                let dist1 = ((xx0 + yy1) * lut_scale) as usize;
                let dist2 = ((xx1 + yy0) * lut_scale) as usize;
                let dist3 = ((xx1 + yy1) * lut_scale) as usize;

                mask[i] = (if dist0 < lut.len() {
                    (lut[dist0] * op) as u8
                } else {
                    0u8
                }) + (if dist1 < lut.len() {
                    (lut[dist1] * op) as u8
                } else {
                    0u8
                }) + (if dist2 < lut.len() {
                    (lut[dist2] * op) as u8
                } else {
                    0u8
                }) + (if dist3 < lut.len() {
                    (lut[dist3] * op) as u8
                } else {
                    0u8
                });
                i += 1;
            }
        }
        (
            diameter as f32 / 2.0,
            BrushMask {
                diameter: diameter as u32,
                mask,
            },
        )
    }

    // Make a low-res Gimp style mask useful for large brushes. Return values includes x/y offset to get the top-left corner
    fn new_gimp_style_lowres(
        radius: f32,
        hardness: f32,
        opacity: f32,
        cache: &mut ClassicBrushCache,
    ) -> (f32, BrushMask) {
        let op = opacity * 255.0;

        let lut = cache.get_cached_lut(hardness);
        let lut_scale = square((LUT_RADIUS - 1.0) / radius);
        let offset;
        let mut fudge = 1.0;
        let mut diameter = ((radius * 2.0).ceil() + 2.0) as i32;
        if diameter % 2 == 0 {
            diameter += 1;
            offset = -1.0;
            if radius < 8.0 {
                fudge = 0.9;
            }
        } else {
            offset = -0.5;
        }

        // empirically determined fudge factors to make small brushes look nice
        if radius < 4.0 {
            fudge = 0.8;
        }

        let mut mask = vec![0u8; (diameter * diameter) as usize];
        let mut i = 0;

        for y in 0..diameter {
            let yy = square(y as f32 - radius + offset);
            for x in 0..diameter {
                let xx = square(x as f32 - radius + offset);
                let dist = ((xx + yy) * fudge * lut_scale) as usize;
                mask[i] = if dist < lut.len() {
                    (lut[dist] * op) as u8
                } else {
                    0u8
                };
                i += 1;
            }
        }
        (
            diameter as f32 / 2.0,
            BrushMask {
                diameter: diameter as u32,
                mask,
            },
        )
    }

    fn offset(&self, x: f32, y: f32) -> BrushMask {
        debug_assert!(x >= 0.0 && x <= 1.0);
        debug_assert!(y >= 0.0 && y <= 1.0);
        let kernel = [x * y, (1.0 - x) * y, x * (1.0 - y), (1.0 - x) * (1.0 - y)];
        let dia = self.diameter as usize;
        let mut newmask = vec![0; dia * dia];
        let mut i = 1usize;
        newmask[0] = (self.mask[0] as f32 * kernel[3]) as u8;
        for x in 0..dia - 1 {
            newmask[i] =
                (self.mask[x] as f32 * kernel[2] + self.mask[x + 1] as f32 * kernel[3]) as u8;
            i += 1;
        }
        for y in 0..dia - 1 as usize {
            let yd = y * dia;
            newmask[i] =
                (self.mask[yd] as f32 * kernel[1] + self.mask[yd + dia] as f32 * kernel[3]) as u8;
            i += 1;
            for x in 0..dia - 1 as usize {
                newmask[i] = (self.mask[yd + x] as f32 * kernel[0]
                    + self.mask[yd + x + 1] as f32 * kernel[1]
                    + self.mask[yd + dia + x] as f32 * kernel[2]
                    + self.mask[yd + dia + x + 1] as f32 * kernel[3])
                    as u8;
                i += 1;
            }
        }
        BrushMask {
            diameter: self.diameter,
            mask: newmask,
        }
    }

    #[cfg(debug_assertions)]
    pub fn to_ascii_art(&self) -> String {
        let mut art = String::new();
        for y in 0..self.diameter {
            for x in 0..self.diameter {
                art.push(if self.mask[(y * self.diameter + x) as usize] == 0 {
                    '.'
                } else {
                    'X'
                });
            }
            art.push('\n');
        }
        art
    }
}

const LUT_RADIUS: f32 = 128.0;

// Generate a lookup table for Gimp style exponential brush shape
// The value at r² (where r is distance from brush center, scaled to LUT_RADIUS) is
// the opaqueness of the pixel.
fn make_gimp_style_brush_lut(hardness: f32) -> Vec<f32> {
    let exponent = if (1.0 - hardness) < 0.000_000_4 {
        1_000_000.0f32
    } else {
        0.4 / (1.0 - hardness)
    };

    let lut_size = (LUT_RADIUS * LUT_RADIUS) as usize;
    Vec::<f32>::from_iter(
        (0..lut_size).map(|i| 1.0 - ((i as f32).sqrt() / LUT_RADIUS).powf(exponent)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_pixel() {
        let mask = BrushMask::new_round_pixel(4, 1.0 / 255.0);
        let expected: [u8; 4 * 4] = [0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0];
        assert_eq!(mask.mask, expected);
    }

    #[test]
    fn test_square_pixel() {
        let mask = BrushMask::new_square_pixel(2, 1.0 / 255.0);
        let expected: [u8; 4] = [1, 1, 1, 1];
        assert_eq!(mask.mask, expected);
    }
}

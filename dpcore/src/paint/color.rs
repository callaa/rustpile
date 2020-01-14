pub type Pixel = [u8;4];

pub const ALPHA_CHANNEL: usize = 0;
pub const RED_CHANNEL: usize = 1;
pub const GREEN_CHANNEL: usize = 2;
pub const BLUE_CHANNEL: usize = 3;
pub const RGB_CHANNELS: std::ops::RangeInclusive<usize> = 1..=3;
pub const ZERO_PIXEL: Pixel = [0, 0, 0, 0];
pub const WHITE_PIXEL: Pixel = [255, 255, 255, 255];

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub fn rgb8(r: u8, g: u8, b: u8) -> Color {
        return Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        };
    }

    pub fn from_argb32(c: u32) -> Color {
        Color {
            r: ((c & 0x00_ff0000) >> 16) as f32 / 255.0,
            g: ((c & 0x00_00ff00) >> 8) as f32 / 255.0,
            b: (c & 0x00_0000ff) as f32 / 255.0,
            a: ((c & 0xff_000000) >> 24) as f32 / 255.0,
        }
    }

    // Get a non-premultiplied pixel value from this color
    pub fn as_argb32(&self) -> u32 {
        ((self.r * 255.0) as u32) << 16
            | ((self.g * 255.0) as u32) << 8
            | ((self.b * 255.0) as u32)
            | ((self.a * 255.0) as u32) << 24
    }

    // Get a color from a premultiplied pixel value
    pub fn from_pixel(p: Pixel) -> Color {
        if p[ALPHA_CHANNEL] == 0 {
            return Color::TRANSPARENT;
        }
        let af = 1.0 / p[ALPHA_CHANNEL] as f32;

        Color {
            r: p[RED_CHANNEL] as f32 * af,
            g: p[GREEN_CHANNEL] as f32 * af,
            b: p[BLUE_CHANNEL] as f32 * af,
            a: p[ALPHA_CHANNEL] as f32 / 255.0,
        }
    }

    // Get a premultiplied pixel value from this color
    pub fn as_pixel(&self) -> Pixel {
        let af = self.a * 255.0;
        [
            (self.a * 255.0) as u8,
            (self.r * af) as u8,
            (self.g * af) as u8,
            (self.b * af) as u8,
        ]
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.as_pixel() == other.as_pixel()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equality() {
        let c1 = Color::rgb8(0, 0, 0);
        let c2 = Color::rgb8(255, 255, 255);
        let c3 = Color::rgb8(255, 255, 254);

        assert!(c1 == c1);
        assert!(c1 != c2);
        assert!(c1 != c3);
        assert!(c2 != c3);
        assert!(
            c1 == Color {
                r: 0.001,
                g: 0.0,
                b: 0.0,
                a: 1.0
            }
        );
    }
}

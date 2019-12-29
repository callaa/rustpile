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

    // Get a color from a premultiplied pixel value
    pub fn from_pixel(p: u32) -> Color {
        let a = p >> 24;
        if a == 0 {
            return Color::TRANSPARENT;
        }
        let af = 1.0 / a as f32;

        Color {
            r: ((p & 0x00ff0000) >> 16) as f32 * af,
            g: ((p & 0x0000ff00) >> 8) as f32 * af,
            b: (p & 0x000000ff) as f32 * af,
            a: a as f32 / 255.0,
        }
    }

    // Get a premultiplied pixel value from this color
    pub fn as_pixel(&self) -> u32 {
        ((self.r * self.a * 255.0) as u32) << 16
            | ((self.g * self.a * 255.0) as u32) << 8
            | ((self.b * self.a * 255.0) as u32)
            | ((self.a * 255.0) as u32) << 24
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

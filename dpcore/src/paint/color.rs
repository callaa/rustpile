// This file is part of Drawpile.
// Copyright (C) 2020 Calle Laakkonen
//
// Drawpile is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// As additional permission under section 7, you are allowed to distribute
// the software through an app store, even if that store has restrictive
// terms and conditions that are incompatible with the GPL, provided that
// the source is also available under the GPL with or without this permission
// through a channel without those restrictive terms and conditions.
//
// Drawpile is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Drawpile.  If not, see <https://www.gnu.org/licenses/>.

pub type Pixel = [u8; 4];

pub const BLUE_CHANNEL: usize = 0;
pub const GREEN_CHANNEL: usize = 1;
pub const RED_CHANNEL: usize = 2;
pub const ALPHA_CHANNEL: usize = 3;

pub const RGB_CHANNELS: std::ops::RangeInclusive<usize> = 0..=2;
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

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub fn rgb8(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    pub fn from_argb32(c: u32) -> Color {
        Color {
            r: ((c & 0x00_ff0000) >> 16) as f32 / 255.0,
            g: ((c & 0x00_00ff00) >> 8) as f32 / 255.0,
            b: (c & 0x00_0000ff) as f32 / 255.0,
            a: ((c & 0xff_000000) >> 24) as f32 / 255.0,
        }
    }

    pub fn argb32_alpha(c: u32) -> u8 {
        ((c & 0xff_000000) >> 24) as u8
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
            (self.b * af) as u8,
            (self.g * af) as u8,
            (self.r * af) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    pub fn is_transparent(&self) -> bool {
        self.a < (1.0 / 255.0)
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

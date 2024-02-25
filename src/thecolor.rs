pub use crate::prelude::*;
use std::ops::{Index, IndexMut};

/// Holds a normalized color value and offers several import and export methods.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TheColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,

    pub name: String,
}

impl Default for TheColor {
    fn default() -> Self {
        Self::gray()
    }
}

impl TheColor {
    /// Creates a color from u8 values.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r,
            g,
            b,
            a,
            name: String::default(),
        }
    }

    /// Creates a color from hsl.
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        fn hue_angle(hue_in: f32, x: f32, y: f32) -> f32 {
            let mut hue = hue_in;

            if hue < 0.0 {
                hue += 1.0;
            } else if hue > 1.0 {
                hue -= 1.0;
            }

            if hue < 1.0 / 6.0 {
                return x + (y - x) * 6.0 * hue;
            }
            if hue < 1.0 / 2.0 {
                return y;
            }
            if hue < 2.0 / 3.0 {
                return x + (y - x) * ((2.0 / 3.0) - hue) * 6.0;
            }

            x
        }

        let (r, g, b) = if s == 0.0 {
            (l, l, l)
        } else {
            let y = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - l * s
            };
            let x = 2.0 * l - y;
            let hue = h / 360.0;

            (
                hue_angle(hue + 1.0 / 3.0, x, y),
                hue_angle(hue, x, y),
                hue_angle(hue - 1.0 / 3.0, x, y),
            )
        };

        Self {
            r,
            g,
            b,
            a: 1.0,
            name: String::default(),
        }
    }

    /// Creates a color from u8 values.
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
            name: String::default(),
        }
    }

    /// Creates a color from u8 values.
    pub fn from_u8_array(color: [u8; 4]) -> Self {
        Self {
            r: color[0] as f32 / 255.0,
            g: color[1] as f32 / 255.0,
            b: color[2] as f32 / 255.0,
            a: color[3] as f32 / 255.0,
            name: String::default(),
        }
    }

    /// Creates a color from an vec3.
    pub fn from_vec3f(color: Vec3f) -> Self {
        Self {
            r: color.x,
            g: color.y,
            b: color.z,
            a: 1.0,
            name: String::default(),
        }
    }

    /// Creates a color from a hex value.
    pub fn from_hex(hex_color: &str) -> Self {
        let mut r = 255;
        let mut g = 255;
        let mut b = 255;
        let mut a = 255;

        if hex_color.len() == 7 || hex_color.len() == 9 {
            if let Ok(rr) = u8::from_str_radix(&hex_color[1..3], 16) {
                r = rr;
            }
            if let Ok(gg) = u8::from_str_radix(&hex_color[3..5], 16) {
                g = gg;
            }
            if let Ok(bb) = u8::from_str_radix(&hex_color[5..7], 16) {
                b = bb;
            }
            if hex_color.len() == 9 {
                if let Ok(aa) = u8::from_str_radix(&hex_color[7..9], 16) {
                    a = aa;
                }
            }
        }

        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
            name: String::default(),
        }
    }

    /// Converts the color to a hexadecimal string.
    pub fn to_hex(&self) -> String {
        // Convert each color component to an integer in the range 0-255
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        let a = (self.a * 255.0).round() as u8;

        // Convert to hexadecimal string. If alpha is fully opaque (255), omit it from the string.
        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    /// Creates a white color.
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
            name: str!("White"),
        }
    }

    /// Creates a gray.
    pub fn gray() -> Self {
        Self {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
            name: str!("Gray"),
        }
    }

    /// Creates a black color.
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
            name: str!("Black"),
        }
    }

    /// Creates a transparent color.
    pub fn transparent() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
            name: str!("Transparent"),
        }
    }

    /// Creates an [f32;4] array
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Creates an [u8;4] array
    pub fn to_u8_array(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    /// Convert the color to Vec3f.
    pub fn to_vec3f(&self) -> Vec3f {
        vec3f(self.r, self.g, self.b)
    }

    pub fn as_srgba(&self) -> TheColor {
        TheColor::new(
            powf(self.r, 0.45),
            powf(self.g, 0.45),
            powf(self.b, 0.45),
            powf(self.a, 0.45),
        )
    }

    /// Convert the color to HSL
    pub fn as_hsl(&self) -> Vec3f {
        let max = self.r.max(self.g.max(self.b));
        let min = self.r.min(self.g.min(self.b));

        let l = (max + min) / 2.0;
        let mut h; // = l;
        let s; // = l;

        if max == min {
            h = 0.0;
            s = 0.0;
        } else {
            let d = max - min;
            s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };

            h = if max == self.r {
                (self.g - self.b) / d + if self.g < self.b { 6.0 } else { 0.0 }
            } else if max == self.g {
                (self.b - self.r) / d + 2.0
            } else {
                (self.r - self.g) / d + 4.0
            };

            h /= 6.0;
        }

        vec3f(h, clamp(s, 0.0, 1.0), clamp(l, 0.0, 1.0))
    }

    /// Lights or darken the color by the given amount.
    pub fn lighten_darken(&self, amount: f32) -> Self {
        let hsl = self.as_hsl();
        let new_l = (hsl.z + amount).max(0.0).min(1.0);

        Self::from_hsl(hsl.x * 360.0, hsl.y, new_l)
    }

    /// Returns a new color as a mix between self and other.
    pub fn mix(&self, other: &TheColor, v: f32) -> TheColor {
        TheColor::new(
            (1.0 - v) * self.r + other.r * v,
            (1.0 - v) * self.g + other.g * v,
            (1.0 - v) * self.b + other.b * v,
            (1.0 - v) * self.a + other.a * v,
        )
    }
}

impl Index<usize> for TheColor {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            3 => &self.a,
            _ => panic!("Index out of bounds!"),
        }
    }
}

impl IndexMut<usize> for TheColor {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            3 => &mut self.a,
            _ => panic!("Index out of bounds!"),
        }
    }
}

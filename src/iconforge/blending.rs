use serde::Serialize;
use std::str::FromStr;

#[derive(Clone)]
pub struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Rgba {
    pub fn into_array(self) -> [u8; 4] {
        [
            self.r.round() as u8,
            self.g.round() as u8,
            self.b.round() as u8,
            self.a.round() as u8,
        ]
    }

    pub fn from_array(rgba: &[u8]) -> Rgba {
        Self {
            r: rgba[0] as f32,
            g: rgba[1] as f32,
            b: rgba[2] as f32,
            a: rgba[3] as f32,
        }
    }

    fn map_each<F, T>(color: &Rgba, color2: &Rgba, rgb_fn: F, a_fn: T) -> Rgba
    where
        F: Fn(f32, f32) -> f32,
        T: Fn(f32, f32) -> f32,
    {
        Rgba {
            r: rgb_fn(color.r, color2.r),
            g: rgb_fn(color.g, color2.g),
            b: rgb_fn(color.b, color2.b),
            a: a_fn(color.a, color2.a),
        }
    }

    fn map_each_a<F, T>(color: &Rgba, color2: &Rgba, rgb_fn: F, a_fn: T) -> Rgba
    where
        F: Fn(f32, f32, f32, f32) -> f32,
        T: Fn(f32, f32) -> f32,
    {
        Rgba {
            r: rgb_fn(color.r, color2.r, color.a, color2.a),
            g: rgb_fn(color.g, color2.g, color.a, color2.a),
            b: rgb_fn(color.b, color2.b, color.a, color2.a),
            a: a_fn(color.a, color2.a),
        }
    }

    /// Takes two [u8; 4]s, converts them to Rgba structs, then blends them according to blend_mode by calling blend().
    pub fn blend_u8(color: &[u8], other_color: &[u8], blend_mode: &BlendMode) -> [u8; 4] {
        Rgba::from_array(color)
            .blend(&Rgba::from_array(other_color), blend_mode)
            .into_array()
    }

    /// Blends two colors according to blend_mode.
    pub fn blend(&self, other_color: &Rgba, blend_mode: &BlendMode) -> Rgba {
        match blend_mode {
            BlendMode::Add => Rgba::map_each(self, other_color, |c1, c2| c1 + c2, f32::min),
            BlendMode::Subtract => Rgba::map_each(self, other_color, |c1, c2| c1 - c2, f32::min),
            BlendMode::Multiply => Rgba::map_each(
                self,
                other_color,
                |c1, c2| c1 * c2 / 255.0,
                |a1: f32, a2: f32| a1 * a2 / 255.0,
            ),
            BlendMode::Overlay => Rgba::map_each_a(
                self,
                other_color,
                |c1, c2, c1_a, c2_a| {
                    if c1_a == 0.0 {
                        return c2;
                    }
                    c1 + (c2 - c1) * c2_a / 255.0
                },
                |a1, a2| {
                    let high = f32::max(a1, a2);
                    let low = f32::min(a1, a2);
                    high + (high * low / 255.0)
                },
            ),
            BlendMode::Underlay => Rgba::map_each_a(
                other_color,
                self,
                |c1, c2, c1_a, c2_a| {
                    if c1_a == 0.0 {
                        return c2;
                    }
                    c1 + (c2 - c1) * c2_a / 255.0
                },
                |a1, a2| {
                    let high = f32::max(a1, a2);
                    let low = f32::min(a1, a2);
                    high + (high * low / 255.0)
                },
            ),
        }
    }
}

// The numbers correspond to BYOND ICON_X blend modes. https://www.byond.com/docs/ref/#/icon/proc/Blend
#[derive(Clone, Hash, Eq, PartialEq, Serialize)]
#[repr(u8)]
pub enum BlendMode {
    Add = 0,
    Subtract = 1,
    Multiply = 2,
    Overlay = 3,
    Underlay = 6,
}

impl BlendMode {
    pub fn from_u8(blend_mode: &u8) -> Result<BlendMode, String> {
        match *blend_mode {
            0 => Ok(BlendMode::Add),
            1 => Ok(BlendMode::Subtract),
            2 => Ok(BlendMode::Multiply),
            3 => Ok(BlendMode::Overlay),
            6 => Ok(BlendMode::Underlay),
            _ => Err(format!("blend_mode '{blend_mode}' is not supported!")),
        }
    }
}

impl FromStr for BlendMode {
    type Err = String;

    fn from_str(blend_mode: &str) -> Result<Self, Self::Err> {
        match blend_mode {
            "add" => Ok(BlendMode::Add),
            "subtract" => Ok(BlendMode::Subtract),
            "multiply" => Ok(BlendMode::Multiply),
            "overlay" => Ok(BlendMode::Overlay),
            "underlay" => Ok(BlendMode::Underlay),
            _ => Err(format!("blend_mode '{blend_mode}' is not supported!")),
        }
    }
}

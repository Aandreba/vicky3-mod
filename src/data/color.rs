use eframe::{egui::{Color32, color_picker::{color_edit_button_rgb, color_edit_button_hsva, color_edit_button_srgb}, Ui, Response, DragValue, Widget}, epaint::{Hsva, Rgba}};
use float_to_int::FloatExt;
use serde::{Serialize, ser::SerializeSeq, Deserialize, de::{Visitor}};
use std::{str::FromStr, alloc::Layout};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[non_exhaustive]
pub enum Color {
    #[serde(rename = "rgb")]
    RgbInt (RgbIntColor),
    #[serde(rename = "")]
    RgbFloat (RgbFloatColor),
    #[serde(rename = "hsv", with = "hsv360_serde")]
    HsvInt (HsvIntColor),
    #[serde(rename = "hsv360", with = "hsv_serde")]
    HsvFloat (HsvFloatColor),
}

impl Color {
    #[inline]
    pub fn render (&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            match self {
                Self::RgbFloat(rgb) => { color_edit_button_rgb(ui, rgb.as_mut_array()); },
                Self::RgbInt(rgb) => { color_edit_button_srgb(ui, rgb.as_mut_array()); },
                _ => {}
            }

            match self {
                Self::RgbFloat(rgb) => {
                    ui.label("Rgb32");

                    DragValue::new(&mut rgb.red)
                        .speed(0.001)
                        .clamp_range(0f32..=1f32)
                        .ui(ui);

                    DragValue::new(&mut rgb.green)
                        .speed(0.001)
                        .clamp_range(0f32..=1f32)
                        .ui(ui);

                    DragValue::new(&mut rgb.blue)
                        .speed(0.001)
                        .clamp_range(0f32..=1f32)
                        .ui(ui);
                },

                Self::RgbInt(rgb) => {
                    ui.label("Rgb");

                    DragValue::new(&mut rgb.red)
                        .clamp_range(0..=255)
                        .ui(ui);

                    DragValue::new(&mut rgb.green)
                        .clamp_range(0..=255)
                        .ui(ui);

                    DragValue::new(&mut rgb.blue)
                        .clamp_range(0..=255)
                        .ui(ui);
                },
                
                Self::HsvInt(hsv) => {
                    ui.label("Hsv360");

                    DragValue::new(&mut hsv.hue)
                        .clamp_range(0..=360)
                        .ui(ui);

                    DragValue::new(&mut hsv.saturation)
                        .clamp_range(0..=255)
                        .ui(ui);

                    DragValue::new(&mut hsv.value)
                        .clamp_range(0..=255)
                        .ui(ui);
                },

                Self::HsvFloat(hsv) => {
                    ui.label("Hsv");

                    DragValue::new(&mut hsv.hue)
                        .speed(0.001)
                        .clamp_range(0f32..=1f32)
                        .ui(ui);

                    DragValue::new(&mut hsv.saturation)
                        .speed(0.001)
                        .clamp_range(0f32..=1f32)
                        .ui(ui);

                    DragValue::new(&mut hsv.value)
                        .speed(0.001)
                        .clamp_range(0f32..=1f32)
                        .ui(ui);
                } 
            };
        });
    }
}

impl From<Color32> for Color {
    #[inline]
    fn from(color: Color32) -> Self {
        Self::RgbInt(RgbIntColor::from(color))
    }
}

impl From<Rgba> for Color {
    #[inline]
    fn from(color: Rgba) -> Self {
        Self::RgbFloat(RgbFloatColor::from(color))
    }
}

impl From<Hsva> for Color {
    #[inline]
    fn from(color: Hsva) -> Self {
        Self::HsvFloat(HsvFloatColor::from(color))
    }
}

impl Into<Color32> for Color {
    #[inline]
    fn into(self) -> Color32 {
        match self {
            Self::RgbInt(x) => x.into(),
            Self::RgbFloat(x) => RgbIntColor::from(x).into(),
            Self::HsvFloat(x) => RgbIntColor::from(x).into(),
            Self::HsvInt(x) => RgbIntColor::from(x).into(),
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {        
        struct Discriminamt;
        impl<'de> Visitor<'de> for Discriminamt {
            type Value = Color;

            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a color")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: serde::de::SeqAccess<'de>, {                                
                return match seq.next_element::<&str>()? {
                    Some("rgb") => seq.next_element::<RgbIntColor>()?
                        .ok_or_else(|| serde::de::Error::custom("color definition not found"))
                        .map(Color::RgbInt),

                    Some("hsv") => {
                        #[derive(Deserialize)]
                        struct Hsv (#[serde(with = "hsv_serde")] pub HsvFloatColor);

                        let Hsv(color) = seq.next_element::<Hsv>()?
                            .ok_or_else(|| serde::de::Error::custom("color definition not found"))?;

                        Ok(Color::HsvFloat(color))
                    },    
                
                    Some("hsv360") => {
                        #[derive(Deserialize)]
                        struct Hsv (#[serde(with = "hsv360_serde")] pub HsvIntColor);

                        let Hsv(color) = seq.next_element::<Hsv>()?
                            .ok_or_else(|| serde::de::Error::custom("color definition not found"))?;

                        Ok(Color::HsvInt(color))
                    },

                    Some(other) => match f32::from_str(other) {
                        Ok(red) => {
                            let green = seq.next_element::<f32>()?
                                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                            
                            let blue = seq.next_element::<f32>()?
                                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                            if red.is_integer() && green.is_integer() && blue.is_integer() {
                                return Ok(Color::RgbInt(RgbIntColor {
                                    red: red as u8,
                                    green: green as u8,
                                    blue: blue as u8
                                }))
                            } else {
                                return Ok(Color::RgbFloat(RgbFloatColor {
                                    red,
                                    green,
                                    blue
                                }))
                            }
                        },

                        Err(e) => Err(serde::de::Error::custom(format!("invalid number '{other}': {e}")))
                    },

                    None => Err(serde::de::Error::custom("color not found"))
                }
            }
        }

        return deserializer.deserialize_seq(Discriminamt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RgbIntColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl RgbIntColor {
    #[inline]
    pub fn as_array (&self) -> &[u8; 3] {
        debug_assert_eq!(Layout::new::<Self>(), Layout::new::<[u8; 3]>());
        return unsafe { &*(self as *const Self as *const [u8; 3]) }
    }

    #[inline]
    pub fn as_mut_array (&mut self) -> &mut [u8; 3] {
        debug_assert_eq!(Layout::new::<Self>(), Layout::new::<[u8; 3]>());
        return unsafe { &mut *(self as *mut Self as *mut [u8; 3]) }
    }
}

impl From<RgbFloatColor> for RgbIntColor {
    #[inline]
    fn from(RgbFloatColor { red, green, blue }: RgbFloatColor) -> Self {
        return Self {
            red: (255f32 * red) as u8,
            green: (255f32 * green) as u8,
            blue: (255f32 * blue) as u8,
        }
    }
}

impl From<HsvFloatColor> for RgbIntColor {
    #[inline]
    fn from(value: HsvFloatColor) -> Self {
        RgbFloatColor::from(value).into()
    }
}

impl From<HsvIntColor> for RgbIntColor {
    #[inline]
    fn from(value: HsvIntColor) -> Self {
        RgbFloatColor::from(value).into()
    }
}

impl Into<Color32> for RgbIntColor {
    #[inline]
    fn into(self) -> Color32 {
        return Color32::from_rgb(self.red, self.green, self.blue)
    }
}

impl From<Color32> for RgbIntColor {
    #[inline]
    fn from(color: Color32) -> Self {
        return Self {
            red: color.r(),
            green: color.g(),
            blue: color.b(),
        }
    }
}

impl Serialize for RgbIntColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&self.red)?;
        ser.serialize_element(&self.green)?;
        ser.serialize_element(&self.blue)?;
        return ser.end()
    }
}

impl<'de> serde::Deserialize<'de> for RgbIntColor {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let [red, green, blue] = <[u8; 3] as serde::Deserialize<'de>>::deserialize(deserializer)?;
        return Ok(Self { red, green, blue })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct RgbFloatColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32
}

impl RgbFloatColor {
    #[inline]
    pub fn as_array (&self) -> &[f32; 3] {
        debug_assert_eq!(Layout::new::<Self>(), Layout::new::<[f32; 3]>());
        return unsafe { &*(self as *const Self as *const [f32; 3]) }
    }

    #[inline]
    pub fn as_mut_array (&mut self) -> &mut [f32; 3] {
        debug_assert_eq!(Layout::new::<Self>(), Layout::new::<[f32; 3]>());
        return unsafe { &mut *(self as *mut Self as *mut [f32; 3]) }
    }
}

impl From<HsvFloatColor> for RgbFloatColor {
    #[inline]
    fn from(HsvFloatColor { hue, saturation, value }: HsvFloatColor) -> Self {
        let hue = 360f32 * hue;

        let c = value * saturation;
        let x: f32 = c * (1f32 - f32::abs(((hue / 60f32) % 2f32) - 1f32));
        let m : f32 = value - c;

        let (r, g, b) = match hue as u16 {
            ..=59 => (c, x, 0f32),
            60..=119 => (x, c, 0f32),
            120..=179 => (0f32, c, x),
            180..=239 => (0f32, x, c),
            240..=299 => (x, 0f32, c),
            _ => (c, 0f32, x)
        };

        return Self {
            red: r + m,
            green: g + m,
            blue: b + m
        }
    }
}

impl From<HsvIntColor> for RgbFloatColor {
    #[inline]
    fn from(value: HsvIntColor) -> Self {
        HsvFloatColor::from(value).into()
    }
}

impl From<RgbIntColor> for RgbFloatColor {
    #[inline]
    fn from(RgbIntColor { red, green, blue }: RgbIntColor) -> Self {
        return Self {
            red: (red as f32) / 255f32,
            green: (green as f32) / 255f32,
            blue: (blue as f32) / 255f32,
        }
    }
}

impl Into<Rgba> for RgbFloatColor {
    #[inline]
    fn into(self) -> Rgba {
        return Rgba::from_rgb(self.red, self.green, self.blue)
    }
}

impl From<Rgba> for RgbFloatColor {
    #[inline]
    fn from(color: Rgba) -> Self {
        return Self {
            red: color.r(),
            green: color.g(),
            blue: color.b(),
        }
    }
}

impl Serialize for RgbFloatColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&f32::from(self.red))?;
        ser.serialize_element(&f32::from(self.green))?;
        ser.serialize_element(&f32::from(self.blue))?;
        return ser.end()
    }
}

impl<'de> serde::Deserialize<'de> for RgbFloatColor {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let [red, green, blue] = <[f32; 3] as serde::Deserialize<'de>>::deserialize(deserializer)?;
        return Ok(Self { 
            red,
            green,
            blue
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HsvIntColor {
    pub hue: u16, // [0, 360]
    pub saturation: u8,
    pub value: u8
}

impl From<HsvFloatColor> for HsvIntColor {
    #[inline]
    fn from(HsvFloatColor { hue, saturation, value }: HsvFloatColor) -> Self {
        return Self {
            hue: (360f32 * hue) as u16,
            saturation: (255f32 * saturation) as u8,
            value: (255f32 * value) as u8,
        }
    }
}

impl From<RgbFloatColor> for HsvIntColor {
    #[inline]
    fn from(value: RgbFloatColor) -> Self {
        HsvFloatColor::from(value).into()
    }
}

impl From<RgbIntColor> for HsvIntColor {
    #[inline]
    fn from(value: RgbIntColor) -> Self {
        HsvFloatColor::from(value).into()
    }
}

impl Serialize for HsvIntColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        return Color::HsvInt(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HsvIntColor {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {        
        let color = <Color as Deserialize<'de>>::deserialize(deserializer)?;
        return match color {
            Color::HsvInt(color) => Ok(color),
            _ => return Err(<D::Error as serde::de::Error>::custom("Extected an HSV360 color, found other color type"))
        }
    }
}

mod hsv360_serde {
    use serde::ser::SerializeSeq;
    use super::HsvIntColor;

    #[inline]
    pub(super) fn serialize<S>(this: &HsvIntColor, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&this.hue)?;
        ser.serialize_element(&this.saturation)?;
        ser.serialize_element(&this.value)?;
        return ser.end()
    }

    #[inline]
    pub(super) fn deserialize<'de, D> (deserializer: D) -> Result<HsvIntColor, D::Error> where D: serde::Deserializer<'de> {        
        let (hue, saturation, value) = <(u16, u8, u8) as serde::Deserialize<'de>>::deserialize(deserializer)?;
        return Ok(HsvIntColor { hue, saturation, value })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HsvFloatColor {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32
}

impl From<RgbFloatColor> for HsvFloatColor {
    fn from(RgbFloatColor { red, green, blue }: RgbFloatColor) -> Self {
        const WEIGHT: f32 = 60f32 / 360f32;

        let c_max = f32::max(f32::max(red, green), blue);
        let c_min = f32::min(f32::min(red, green), blue);
        let delta = c_max - c_min;

        let hue = match (delta, c_max) {
            (0f32, _) => 0f32,
            (_, x) if x == red => WEIGHT * (((green - blue) / delta) % 6f32),
            (_, x) if x == green => WEIGHT * (((blue - red) / delta) + 2f32),
            _ => WEIGHT * (((red - green) / delta) + 4f32),
        };

        let saturation = match c_max {
            0f32 => 0f32,
            other => delta / other
        };

        return Self {
            hue: hue,
            saturation: saturation,
            value: c_max
        }
    }
}

impl From<RgbIntColor> for HsvFloatColor {
    #[inline]
    fn from(value: RgbIntColor) -> Self {
        RgbFloatColor::from(value).into()
    }
}

impl From<HsvIntColor> for HsvFloatColor {
    #[inline]
    fn from(HsvIntColor { hue, saturation, value }: HsvIntColor) -> Self {
        return Self {
            hue: hue as f32 / 360f32,
            saturation: saturation as f32 / 255f32,
            value: value as f32 / 255f32,
        }
    }
}

impl Into<Hsva> for HsvFloatColor {
    #[inline]
    fn into(self) -> Hsva {
        return Hsva {
            h: self.hue,
            s: self.saturation,
            v: self.value,
            a: 1f32
        }
    }
}

impl From<Hsva> for HsvFloatColor {
    #[inline]
    fn from(Hsva { h, s, v, .. }: Hsva) -> Self {
        return Self {
            hue: h,
            saturation: s,
            value: v,
        }
    }
}

impl Serialize for HsvFloatColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        return Color::HsvFloat(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HsvFloatColor {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {        
        let color = <Color as Deserialize<'de>>::deserialize(deserializer)?;
        return match color {
            Color::HsvFloat(color) => Ok(color),
            _ => return Err(<D::Error as serde::de::Error>::custom("Extected an HSV color, found an other color type"))
        }
    }
}

mod hsv_serde {
    use serde::ser::SerializeSeq;
    use super::HsvFloatColor;

    #[inline]
    pub(super) fn serialize<S>(this: &HsvFloatColor, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&f32::from(this.hue))?;
        ser.serialize_element(&f32::from(this.saturation))?;
        ser.serialize_element(&f32::from(this.value))?;
        return ser.end()
    }

    #[inline]
    pub(super) fn deserialize<'de, D> (deserializer: D) -> Result<HsvFloatColor, D::Error> where D: serde::Deserializer<'de> {        
        let [hue, saturation, value] = <[f32; 3] as serde::Deserialize<'de>>::deserialize(deserializer)?;
        return Ok(HsvFloatColor {
            hue,
            saturation,
            value
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::Color;
    
    #[test]
    fn rgb_header () {
        let color = b"1 = rgb{ 255 128 64 } 4 = hsv{ 1 0.5 0.25 } 2 = hsv360{ 255 128 64 } 3 = { 255 128 64 }";
        let text = jomini::text::de::from_utf8_slice::<HashMap<u32, Color>>(color);
        println!("{text:#?}")
    }
}
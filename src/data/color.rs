use eframe::{egui::Color32, epaint::{Hsva, Rgba}};
use float_to_int::FloatExt;
use half::f16;
use serde::{Serialize, ser::SerializeSeq, Deserialize, de::{Visitor}};
use std::{str::FromStr};

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
                                    red: f16::from_f32(red),
                                    green: f16::from_f32(green),
                                    blue: f16::from_f32(blue)
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
pub struct RgbIntColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl From<RgbFloatColor> for RgbIntColor {
    #[inline]
    fn from(RgbFloatColor { red, green, blue }: RgbFloatColor) -> Self {
        return Self {
            red: (255f32 * red.to_f32()) as u8,
            green: (255f32 * green.to_f32()) as u8,
            blue: (255f32 * blue.to_f32()) as u8,
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
pub struct RgbFloatColor {
    pub red: f16,
    pub green: f16,
    pub blue: f16
}

impl From<HsvFloatColor> for RgbFloatColor {
    #[inline]
    fn from(HsvFloatColor { hue, saturation, value }: HsvFloatColor) -> Self {
        let hue = 360f32 * hue.to_f32();
        let saturation = saturation.to_f32();
        let value = value.to_f32();

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
            red: f16::from_f32(r + m),
            green: f16::from_f32(g + m),
            blue: f16::from_f32(b + m)
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
            red: f16::from_f32((red as f32) / 255f32),
            green: f16::from_f32((green as f32) / 255f32),
            blue: f16::from_f32((blue as f32) / 255f32),
        }
    }
}

impl Into<Rgba> for RgbFloatColor {
    #[inline]
    fn into(self) -> Rgba {
        return Rgba::from_rgb(self.red.to_f32(), self.green.to_f32(), self.blue.to_f32())
    }
}

impl From<Rgba> for RgbFloatColor {
    #[inline]
    fn from(color: Rgba) -> Self {
        return Self {
            red: f16::from_f32(color.r()),
            green: f16::from_f32(color.g()),
            blue: f16::from_f32(color.b()),
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
            red: f16::from_f32(red),
            green: f16::from_f32(green),
            blue: f16::from_f32(blue)
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
            hue: (360f32 * hue.to_f32()) as u16,
            saturation: (255f32 * saturation.to_f32()) as u8,
            value: (255f32 * value.to_f32()) as u8,
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
    pub hue: f16,
    pub saturation: f16,
    pub value: f16
}

impl From<RgbFloatColor> for HsvFloatColor {
    fn from(RgbFloatColor { red, green, blue }: RgbFloatColor) -> Self {
        const WEIGHT: f32 = 60f32 / 360f32;

        let red = red.to_f32();
        let green = green.to_f32();
        let blue = blue.to_f32();

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
            hue: f16::from_f32(hue),
            saturation: f16::from_f32(saturation),
            value: f16::from_f32(c_max)
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
            hue: f16::from_f32(hue as f32 / 360f32),
            saturation: f16::from_f32(saturation as f32 / 255f32),
            value: f16::from_f32(value as f32 / 255f32),
        }
    }
}

impl Into<Hsva> for HsvFloatColor {
    #[inline]
    fn into(self) -> Hsva {
        return Hsva {
            h: self.hue.to_f32(),
            s: self.saturation.to_f32(),
            v: self.value.to_f32(),
            a: 1f32
        }
    }
}

impl From<Hsva> for HsvFloatColor {
    #[inline]
    fn from(Hsva { h, s, v, .. }: Hsva) -> Self {
        return Self {
            hue: f16::from_f32(h),
            saturation: f16::from_f32(s),
            value: f16::from_f32(v),
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
    use half::f16;
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
            hue: f16::from_f32(hue),
            saturation: f16::from_f32(saturation),
            value: f16::from_f32(value)
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
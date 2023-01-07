use float_to_int::FloatExt;
use half::f16;
use serde::{Serialize, ser::SerializeSeq, Deserialize, de::{Visitor}};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Color {
    RgbInt (RgbIntColor),
    RgbFloat (RgbFloatColor),
    // todo float rgb 
    #[serde(with = "hsv_serde")]
    Hsv (HsvColor),
    #[serde(with = "hsv360_serde")]
    Hsv360 (HsvIntColor),
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
                        struct Hsv (#[serde(with = "hsv_serde")] pub HsvColor);

                        let Hsv(color) = seq.next_element::<Hsv>()?
                            .ok_or_else(|| serde::de::Error::custom("color definition not found"))?;

                        Ok(Color::Hsv(color))
                    },    
                
                    Some("hsv360") => {
                        #[derive(Deserialize)]
                        struct Hsv (#[serde(with = "hsv360_serde")] pub HsvIntColor);

                        let Hsv(color) = seq.next_element::<Hsv>()?
                            .ok_or_else(|| serde::de::Error::custom("color definition not found"))?;

                        Ok(Color::Hsv360(color))
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

impl Serialize for HsvIntColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        return Color::Hsv360(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HsvIntColor {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {        
        let color = <Color as Deserialize<'de>>::deserialize(deserializer)?;
        return match color {
            Color::Hsv360(color) => Ok(color),
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
pub struct HsvColor {
    pub hue: f16,
    pub saturation: f16,
    pub value: f16
}

impl Serialize for HsvColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        return Color::Hsv(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HsvColor {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {        
        let color = <Color as Deserialize<'de>>::deserialize(deserializer)?;
        return match color {
            Color::Hsv(color) => Ok(color),
            _ => return Err(<D::Error as serde::de::Error>::custom("Extected an HSV color, found an other color type"))
        }
    }
}

mod hsv_serde {
    use half::f16;
    use serde::ser::SerializeSeq;
    use super::HsvColor;

    #[inline]
    pub(super) fn serialize<S>(this: &HsvColor, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&f32::from(this.hue))?;
        ser.serialize_element(&f32::from(this.saturation))?;
        ser.serialize_element(&f32::from(this.value))?;
        return ser.end()
    }

    #[inline]
    pub(super) fn deserialize<'de, D> (deserializer: D) -> Result<HsvColor, D::Error> where D: serde::Deserializer<'de> {        
        let [hue, saturation, value] = <[f32; 3] as serde::Deserialize<'de>>::deserialize(deserializer)?;
        return Ok(HsvColor {
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
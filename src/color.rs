use serde::{Serialize, ser::SerializeSeq, Deserialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Color {
    Rgb (RbgColor),
    #[serde(with = "hsv360_serde")]
    Hsv360 (Hsv360Color),
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "lowercase")]
        enum InnerColor<'a> {
            #[serde(borrow = "'a")]
            Rgb ([&'a str; 3]),
            #[serde(borrow = "'a")]
            Hsv360 ([&'a str; 3])
        }

        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum Inner<'a> {
            #[serde(borrow = "'a")]
            Tagged (InnerColor<'a>),
            #[serde(borrow = "'a")]
            Untagged ([&'a str; 3]),
        }

        let todo = serde_bridge::Value::deserialize(deserializer)?;
        todo!("{todo:?}");

        /*match <Inner as Deserialize>::deserialize(deserializer)? {
            Inner::Untagged(rgb) | Inner::Tagged(InnerColor::Rgb(rgb)) => {
                let red = u8::from_str(rgb[0])
                    .map_err(|e| serde::de::Error::custom(e))?;

                let green = u8::from_str(rgb[1])
                    .map_err(|e| serde::de::Error::custom(e))?;

                let blue = u8::from_str(rgb[2])
                    .map_err(|e| serde::de::Error::custom(e))?;

                return Ok(Color::Rgb(RbgColor { red, green, blue }))
            },

            Inner::Tagged(InnerColor::Hsv360(hsv)) => {
                let hue = u16::from_str(hsv[0])
                    .map_err(|e| serde::de::Error::custom(e))?;

                let saturation = u8::from_str(hsv[1])
                    .map_err(|e| serde::de::Error::custom(e))?;

                let value = u8::from_str(hsv[2])
                    .map_err(|e| serde::de::Error::custom(e))?;

                return Ok(Color::Hsv360(Hsv360Color { hue, saturation, value }))
            }
        }*/
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RbgColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl Serialize for RbgColor {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&self.red)?;
        ser.serialize_element(&self.green)?;
        ser.serialize_element(&self.blue)?;
        return ser.end()
    }
}

impl<'de> serde::Deserialize<'de> for RbgColor {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let [red, green, blue] = <[u8; 3] as serde::Deserialize<'de>>::deserialize(deserializer)?;
        return Ok(Self { red, green, blue })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hsv360Color {
    pub hue: u16, // [0, 360]
    pub saturation: u8,
    pub value: u8
}

impl Serialize for Hsv360Color {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        return Color::Hsv360(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Hsv360Color {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {        
        let color = <Color as Deserialize<'de>>::deserialize(deserializer)?;
        return match color {
            Color::Hsv360(color) => Ok(color),
            _ => return Err(<D::Error as serde::de::Error>::custom("Extected an HSV360 color, found an RGB color"))
        }
    }
}

mod hsv360_serde {
    use serde::ser::SerializeSeq;
    use crate::Hsv360Color;

    #[inline]
    pub(super) fn serialize<S>(this: &Hsv360Color, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {        
        let mut ser = serializer.serialize_seq(Some(3))?;
        ser.serialize_element(&this.hue)?;
        ser.serialize_element(&this.saturation)?;
        ser.serialize_element(&this.value)?;
        return ser.end()
    }

    #[inline]
    pub(super) fn deserialize<'de, D> (deserializer: D) -> Result<Hsv360Color, D::Error> where D: serde::Deserializer<'de> {        
        let (hue, saturation, value) = <(u16, u8, u8) as serde::Deserialize<'de>>::deserialize(deserializer)?;
        return Ok(Hsv360Color { hue, saturation, value })
    }
}

//#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use elor::Either;
    use jomini::binary::Rgb;
    use serde::{Serialize, Deserialize};
    use crate::{RbgColor, Color, Hsv360Color, Str};
    
    #[test]
    fn rgb_header () {
        let color = b"1 = hsv360{ 255 128 64 }";
        let text = jomini::text::de::from_utf8_slice::<HashMap<u32, Color>>(color);
        println!("{text:?}")
    }
}
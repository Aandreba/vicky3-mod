use serde::{Serialize, ser::SerializeSeq, Deserialize, de::{Visitor}};
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
        #[derive(Debug)]
        enum Field {
            Rgb,
            Hsv360
        }
        
        struct Discriminamt;
        impl<'de> Visitor<'de> for Discriminamt {
            type Value = Color;

            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                todo!()
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: serde::de::SeqAccess<'de>, {                
                return match seq.next_element::<&str>()? {
                    Some("rgb") => seq.next_element::<RbgColor>()?
                        .ok_or_else(|| serde::de::Error::custom("color definition not found"))
                        .map(Color::Rgb),

                    Some("hsv360") => {
                        #[derive(Deserialize)]
                        struct Hsv (#[serde(with = "hsv360_serde")] pub Hsv360Color);

                        let Hsv(color) = seq.next_element::<Hsv>()?
                            .ok_or_else(|| serde::de::Error::custom("color definition not found"))?;

                        Ok(Color::Hsv360(color))
                    },

                    Some(other) => match u8::from_str(other) {
                        Ok(red) => {
                            let green = seq.next_element::<u8>()?
                                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                            
                            let blue = seq.next_element::<u8>()?
                                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                        
                            Ok(Color::Rgb(RbgColor { red, green, blue }))
                        },
                        Err(e) => Err(serde::de::Error::custom(e))
                    },

                    None => Err(serde::de::Error::custom("color not found"))
                }
            }
        }

        return deserializer.deserialize_seq(Discriminamt)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::Color;
    
    #[test]
    fn rgb_header () {
        let color = b"1 = rgb{ 255 128 64 } 2 = hsv360{ 255 128 64 } 3 = { 255 128 64 }";
        let text = jomini::text::de::from_utf8_slice::<HashMap<u32, Color>>(color);
        println!("{text:#?}")
    }
}
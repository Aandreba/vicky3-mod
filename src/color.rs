use serde::{Serialize, ser::SerializeSeq, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Color {
    #[serde(rename = "")]
    Rgb (RbgColor),
    #[serde(with = "hsv360_serde", rename = "hsv360")]
    Hsv360 (Hsv360Color),
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
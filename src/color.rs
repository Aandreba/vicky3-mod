use serde::{Serialize, ser::SerializeSeq, Deserialize, de::{Visitor, VariantAccess, DeserializeSeed}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Color {
    Rgb (RbgColor),
    #[serde(with = "hsv360_serde")]
    Hsv360 (Hsv360Color),
}

impl<'de> Deserialize<'de> for Color {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        enum ColorType {
            Rgb,
            Hsv360,
            NoHeaderRgb (RbgColor)
        }

        impl<'de> Deserialize<'de> for ColorType {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
                deserializer.deserialize_unit(VariantVisitor);
                //deserializer.deserialize_identifier(VariantVisitor);
            }
        }
        
        struct VariantVisitor;
        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = ColorType;

            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "nothing, `rgb` or `hsv360`")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: serde::de::SeqAccess<'de>, {
                let red: u8 = seq.next_element()?
                    .ok_or_else(|| <A::Error as serde::de::Error>::missing_field("red"))?;
                
                let green: u8 = seq.next_element()?
                    .ok_or_else(|| <A::Error as serde::de::Error>::missing_field("green"))?;

                let blue: u8 = seq.next_element()?
                    .ok_or_else(|| <A::Error as serde::de::Error>::missing_field("blue"))?;

                return Ok(ColorType::NoHeaderRgb(RbgColor { red, green, blue }))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
                match v {
                    "rgb" => Ok(ColorType::Rgb),
                    "hsv360" => Ok(ColorType::Hsv360),
                    unexp => return Err(E::unknown_variant(unexp, &["rgb", "hsv360"]))
                }
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E> where E: serde::de::Error, {
                return Ok(ColorType::Rgb)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E> where E: serde::de::Error, {
                return self.visit_unit()
            }
        }
        
        struct LocalVisitor;
        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = Color;

            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "an rgb or hsv360 color")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where A: serde::de::EnumAccess<'de>, {
                return match data.variant::<ColorType>()? {
                    (ColorType::NoHeaderRgb(rgb), _) => Ok(Color::Rgb(rgb)),

                    (ColorType::Rgb, variant) => Result::map(
                        variant.newtype_variant::<RbgColor>(),
                        Color::Rgb,
                    ),

                    (ColorType::Hsv360, variant) => {
                        #[repr(transparent)]
                        struct Hsv360 (pub Hsv360Color);
                        impl<'de> Deserialize<'de> for Hsv360 {
                            #[inline]
                            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
                                return hsv360_serde::deserialize(deserializer).map(Self)
                            }
                        }

                        Result::map(
                            variant.newtype_variant::<Hsv360>(),
                            |x| Color::Hsv360(x.0),
                        )
                    },
                }
            }
        }
        
        return deserializer.deserialize_enum(
            "Color",
            &["rgb", "hsv360"],
            LocalVisitor
        )
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
    use crate::{RbgColor, Color};

    #[test]
    fn rgb_header () {
        let color = b"1 = { 255 128 64 }";
        let text = jomini::text::de::from_utf8_slice::<HashMap<u32, Color>>(color);
        println!("{text:?}")
    }
}
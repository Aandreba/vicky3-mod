use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[non_exhaustive]
pub enum IdentKind {
    Country,
    State,
    #[default]
    Unknown
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Ident {
    pub kind: IdentKind,
    pub value: String    
}

impl Ident {
    #[inline]
    pub fn from_str (s: &str) -> Self {
        let (kind, value) = match &s[..2] {
            "c:" => (IdentKind::Country, &s[2..]),
            "s:" => (IdentKind::State, &s[2..]),
            _ => (IdentKind::Unknown, s)
        };
        return Self { kind, value: value.to_string() }
    }

    #[inline]
    pub fn from_string (s: String) -> Self {
        let (kind, value) = match &s[..2] {
            "c:" => (IdentKind::Country, s[2..].to_string()),
            "s:" => (IdentKind::State, s[2..].to_string()),
            _ => (IdentKind::Unknown, s)
        };
        return Self { kind, value }
    }

    #[inline]
    pub fn eq_str (&self, other: &str) -> bool {
        return self.value.eq(other)
    }

    #[inline]
    pub fn eq_name (&self, other: &Ident) -> bool {
        self.eq_str(&other.value)
    }
}

impl Serialize for Ident {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let value = &self.value;
        match self.kind {
            IdentKind::Country => format!("c:{value}").serialize(serializer),
            IdentKind::State => format!("s:{value}").serialize(serializer),
            IdentKind::Unknown => self.value.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Ident {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = <&str as Deserialize>::deserialize(deserializer)?;
        return Ok(Self::from_str(s))
    }
}
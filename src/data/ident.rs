use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[non_exhaustive]
pub enum IdentKind {
    Country,
    State,
    RegionState,
    #[default]
    Unknown
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Ident {
    pub value: String,  
    pub kind: IdentKind,
}

impl Ident {
    #[inline]
    pub fn from_str (s: &str) -> Self {
        let (kind, value) = if s.starts_with("c:") {
            (IdentKind::Country, &s[2..])
        } else if s.starts_with("s:") {
            (IdentKind::State, &s[2..])
        } else if s.starts_with("region_state:") {
            (IdentKind::RegionState, &s[13..])
        } else {
            (IdentKind::Unknown, s)
        };

        return Self { kind, value: value.to_string() }
    }

    #[inline]
    pub fn from_string (s: String) -> Self {
        let (kind, value) = if s.starts_with("c:") {
            (IdentKind::Country, s[2..].to_string())
        } else if s.starts_with("s:") {
            (IdentKind::State, s[2..].to_string())
        } else if s.starts_with("region_state:") {
            (IdentKind::RegionState, s[13..].to_string())
        } else {
            (IdentKind::Unknown, s)
        };

        return Self { kind, value }
    }

    #[inline]
    pub fn eq_name (&self, other: &Ident) -> bool {
        self.value == other.value
    }
}

impl PartialEq<str> for Ident {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

impl PartialEq<Ident> for str {
    #[inline]
    fn eq(&self, other: &Ident) -> bool {
        self == other.value
    }
}

impl Serialize for Ident {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let value = &self.value;
        match self.kind {
            IdentKind::Country => format!("c:{value}").serialize(serializer),
            IdentKind::State => format!("s:{value}").serialize(serializer),
            IdentKind::RegionState => format!("region_state:{value}").serialize(serializer),
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
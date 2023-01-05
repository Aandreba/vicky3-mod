use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[non_exhaustive]
pub enum CountryTier {
    CityState,
    Principality,
    GrandPrincipality,
    Kingdom,
    Empire,
    /// At release in the basegame, only the country of India is a hegemony. Think of it as a megaempire
    Hegemony
}

// todo try implement deser manually
impl<'de> Deserialize<'de> for CountryTier {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        match <&str as Deserialize>::deserialize(deserializer)? {
            "city_state" => Ok(Self::CityState),
            "principality" => Ok(Self::Principality),
            "grand_principality" => Ok(Self::GrandPrincipality),
            "kingdom" => Ok(Self::Kingdom),
            "empire" => Ok(Self::Empire),
            "hegemony" => Ok(Self::Hegemony),
            other => return Err(<D::Error as serde::de::Error>::unknown_variant(other, &[
                "city_state",
                "principality",
                "grand_principality",
                "kingdom",
                "empire",
                "hegemony"
            ]))
        }
    }
}
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
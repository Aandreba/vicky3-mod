use std::{path::Path, collections::HashMap};
use itertools::Itertools;
use jomini::JominiDeserialize;
use serde::Serialize;
use crate::{Result, flat_map_ok, Str};
use super::{NamedCountryRank, CountryRanks};

pub type NamedCountryType<'a> = (&'a Str, &'a CountryType<'a>);
pub type NamedRawCountryType<'a> = (&'a Str, &'a RawCountryType);

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CountryType<'a> {
    pub is_colonizable: bool,
    pub is_unrecognized: bool,
    pub uses_prestige: bool,
    pub has_events: bool,
    pub has_military: bool,
    pub has_economy: bool,
    pub has_politics: bool,
    pub can_research: bool,
    pub default_rank: Option<NamedCountryRank<'a>>
}

impl<'a> CountryType<'a> {
    #[inline]
    pub fn from_raw (raw: RawCountryType, ranks: &'a CountryRanks) -> Result<Self> {
        return Ok(Self {
            default_rank: ranks.get_key_value(&raw.default_rank),
            is_colonizable: raw.is_colonizable,
            is_unrecognized: raw.is_unrecognized,
            uses_prestige: raw.uses_prestige,
            has_events: raw.has_events,
            has_military: raw.has_military,
            has_economy: raw.has_economy,
            has_politics: raw.has_politics,
            can_research: raw.can_research,
        })
    }

    #[inline]
    pub fn from_common (common: &Path, ranks: &'a CountryRanks) -> Result<impl Iterator<Item = Result<(Str, CountryType<'a>)>>> {
        let iter = flat_map_ok(
            RawCountryType::from_common(common)?,
            |(name, raw)| Self::from_raw(raw, ranks).map(|this| (name, this)) 
        );

        return Ok(iter)
    }
}

#[derive(Debug, Serialize, JominiDeserialize)]
#[non_exhaustive]
pub struct RawCountryType {
    pub is_colonizable: bool,
    pub is_unrecognized: bool,
    pub uses_prestige: bool,
    pub has_events: bool,
    pub has_military: bool,
    pub has_economy: bool,
    pub has_politics: bool,
    pub can_research: bool,
    pub default_rank: Str
}

impl RawCountryType {
    #[inline]
    pub fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let data = std::fs::read_to_string(path)?;
        return jomini::text::de::from_utf8_slice(data.as_bytes())
    }

    #[inline]
    pub fn from_common (common: &Path) -> Result<impl Iterator<Item = Result<(Str, Self)>>> {
        let path = common.join("country_types");
        let iter = std::fs::read_dir(path)?
            .filter_ok(|x| x.metadata().unwrap().is_file())
            .map_ok(|x| Self::from_path(x.path()))
            .flatten()
            .flatten_ok();

        return Ok(iter)
    }
}

/*

decentralized = {
	is_colonizable = yes	# yes/no: if a country is colonizable
	is_unrecognized = yes	# yes/no: does non-colonial countries consider this an unrecognized country to be colonized (impacts certain AI decisions)
	uses_prestige = no # yes/no: If no, always has a prestige of 0 and does not display a rank position

	has_events = no

	has_military = yes
	has_economy = no
	has_politics = no
	can_research = no
	
	default_rank = decentralized_power
}

*/
use std::{path::Path, collections::HashMap};
use itertools::*;
use jomini::JominiDeserialize;
use crate::{Result, Str};

pub type NamedCountryRank<'a> = (&'a Str, &'a CountryRank);
pub type CountryRanks = HashMap<Str, CountryRank>;

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct CountryRank {
    /// higher value rank effects take priority over lower ones in being assigned, also determines icon index
    pub rank_value: u8,
    pub icon_index: u8,
    /// if yes, this rank is invalid for subjects with a subject type where overlord needs to have higher rank if overlord has that rank or higher
    #[jomini(default)]
    pub enforce_subject_rank_check: bool,
    /// multiple of average country prestige
    #[jomini(default)]
    pub prestige_average_threshold: f32,
    /// relative to highest prestige country
    #[jomini(default)]
    pub prestige_relative_threshold: f32,
    /// minimum amount of generals in the country; if below, game will auto-generate
    #[jomini(default)]
    pub min_generals: Option<u32>,
    /// max ranks when auto-generating commander rank
    #[jomini(default)]
    pub max_commander_rank_random: Option<u32>,
    /// min ranks when auto-generating commander rank
    #[jomini(default)]
    pub min_commander_rank_random: Option<u32>,
    /*
    /// {} must evaluate to true for rank to be able to be assigned
    pub possible: bool,
    */
    /// whether a country of this rank can colonize
    #[jomini(default = "default_true")]
    pub can_colonize: bool,
    /// Diplomatic pacts with country of this rank have their cost multiplied by 1 + this amount
    #[jomini(default)]
    pub diplo_pact_cost: f32,

}

impl CountryRank {
    #[inline]
    pub fn from_path (path: impl AsRef<Path>) -> Result<CountryRanks> {
        let data = std::fs::read_to_string(path)?;
        return jomini::text::de::from_utf8_slice::<CountryRanks>(data.as_bytes());
    }

    #[inline]
    pub fn from_common (common: impl AsRef<Path>) -> Result<impl Iterator<Item = Result<(Str, Self)>>> {
        let ranks = common.as_ref().join("country_ranks");
        let iter = std::fs::read_dir(ranks)?
            .filter_ok(|x| x.metadata().unwrap().is_file())
            .map_ok(|x| Self::from_path(x.path()))
            .flatten()
            .flatten_ok();

        return Ok(iter)
    }
}

#[inline(always)]
const fn default_true () -> bool { true }
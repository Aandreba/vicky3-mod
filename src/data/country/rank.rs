use std::{path::Path, collections::HashMap};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use crate::{data::read_to_string, Result, Str, utils::{ReadDirStream, FlattenOkIter}};

pub type NamedCountryRank<'a> = (&'a str, &'a CountryRank);

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
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let data = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(data.as_bytes())).await.unwrap();
    }

    #[inline]
    pub async fn from_common (common: impl AsRef<Path>) -> Result<impl Stream<Item = Result<(Str, Self)>>> {
        let ranks = common.as_ref().join("country_ranks");
        let iter = ReadDirStream::new(tokio::fs::read_dir(ranks).await?)
            .map_err(<jomini::Error as From<std::io::Error>>::from)
            .try_filter_map(|x: tokio::fs::DirEntry| async move {
                if x.metadata().await.map_err(jomini::Error::from)?.is_file() {
                    return Ok(Some(Self::from_path(x.path()).await?))
                } else {
                    return Ok(None)
                }
            });

        return Ok(FlattenOkIter::new(iter))
    }
}

#[inline(always)]
const fn default_true () -> bool { true }
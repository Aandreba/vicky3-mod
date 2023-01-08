use std::{path::Path, collections::HashMap};
use futures::{TryStreamExt, Stream};
use jomini::JominiDeserialize;
use serde::Serialize;
use tokio::task::spawn_blocking;
use crate::{Str, Result, utils::{ReadDirStream, FlattenOkIter}, data::read_to_string};

#[derive(Debug, Serialize, JominiDeserialize)]
#[non_exhaustive]
pub struct CountryType {
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

impl CountryType {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let data = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(data.as_bytes())).await.unwrap()
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(Str, Self)>>> {
        let path = common.join("country_types");
        let iter = ReadDirStream::new(tokio::fs::read_dir(path).await?)
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
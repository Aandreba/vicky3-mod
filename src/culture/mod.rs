use std::{collections::HashMap, path::Path};
use futures::{Stream, StreamExt, TryStreamExt};
use itertools::Itertools;
use jomini::JominiDeserialize;
use rayon::prelude::{ParallelIterator};
use tokio_stream::wrappers::ReadDirStream;
use crate::{Color, Result, Str, read_to_string};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct RawCulture {
    pub color: Color,
    pub religion: Str,
    #[jomini(default)]
    pub traits: Vec<Str>,
    #[jomini(default)]
    pub male_common_first_names: Vec<Str>,
    #[jomini(default)]
    pub female_common_first_names: Vec<Str>,
    #[jomini(default)]
    pub noble_last_names: Vec<Str>,
    #[jomini(default)]
    pub common_last_names: Vec<Str>,
    #[jomini(default)]
    pub male_regal_first_names: Vec<Str>,
    #[jomini(default)]
    pub female_regal_first_names: Vec<Str>,
    pub graphics: Str,
    pub ethnicities: HashMap<u32, Str>
}

impl RawCulture {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let data = read_to_string(path).await?;
        return jomini::text::de::from_utf8_slice(data.as_bytes())
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(Str, Self)>>> {
        let path = common.join("cultures");
        let iter = ReadDirStream::new(tokio::fs::read_dir(path).await?)
            .map_err(<jomini::Error as From<std::io::Error>>::from)
            .try_filter_map(|x: tokio::fs::DirEntry| async move {
                if x.metadata().await.map_err(jomini::Error::from)?.is_file() {
                    return Ok(Some(Self::from_path(x.path()).await?))
                } else {
                    return Ok(None)
                }
            });

        return Ok(iter)
    }
}
use std::{path::Path, collections::HashMap};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use super::{CountryTier};
use crate::{Result, Str, utils::{ReadDirStream, FlattenOkIter}, data::{Color, read_to_string}};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct CountryDefinition {
    pub color: Color,
    pub country_type: Str,
    pub tier: CountryTier,
    pub cultures: Box<[Str]>,
    pub capital: Option<Str>,
    #[jomini(default)]
    pub is_named_from_capital: bool
}

impl CountryDefinition {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let contents = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(contents.as_bytes())).await.unwrap();
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(Str, Self)>>> {
        let path = common.join("country_definitions");
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
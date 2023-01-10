use std::{path::Path, collections::HashMap};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use super::{CountryTier};
use crate::{Result, utils::{ReadDirStream, FlattenOkIter}, data::{Color, read_to_string, GamePaths, Ident}};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct CountryDefinition {
    pub color: Color,
    pub country_type: String,
    pub tier: CountryTier,
    pub cultures: Box<[String]>,
    pub capital: Option<Ident>,
    #[jomini(default)]
    pub is_named_from_capital: bool
}

impl CountryDefinition {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Ident, Self>> {
        let contents = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(contents.as_bytes())).await.unwrap();
    }

    #[inline]
    pub async fn from_game (game: &GamePaths) -> Result<impl Stream<Item = Result<(Ident, Self)>>> {
        let path = game.common().join("country_definitions");
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
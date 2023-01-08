use std::{path::{Path}, collections::HashMap};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use crate::{Str, Result};
use crate::utils::{ReadDirStream, FlattenOkIter};
use super::{Color, read_to_string};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct Religion {
    pub texture: Box<Path>,
    // religion traits, different from other kinds of traits
    pub traits: Box<[Str]>,
    pub color: Color,
    #[jomini(default)]
    pub taboos: Box<[Str]>
}

impl Religion {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let data = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(data.as_bytes())).await.unwrap()
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(Str, Self)>>> {
        let path = common.join("religions");
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
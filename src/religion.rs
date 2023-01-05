use std::{path::{Path}, collections::HashMap};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use crate::{Str, Color, Result, read_to_string, utils::{ReadDirStream, FlattenOkIter}};

pub type NamedReligion<'a> = (&'a str, &'a Religion);

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct Religion {
    texture: Box<Path>,
    // religion traits, different from other kinds of traits
    // traits: Vec<Str>,
    color: Color,
    // taboos: Vec<Str>
}

impl Religion {
    #[inline]
    pub fn from_raw (raw: RawReligion) -> Self {
        return Self {
            texture: raw.texture,
            color: raw.color,
        }
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(Str, Self)>>> {
        return Ok(
            RawReligion::from_common(common)
                .await?
                .map_ok(|(name, raw)| (name, Self::from_raw(raw)))
        )
    }
}

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct RawReligion {
    texture: Box<Path>,
    // religion traits, different from other kinds of traits
    traits: Box<[Str]>,
    color: Color,
    #[jomini(default)]
    taboos: Box<[Str]>
}

impl RawReligion {
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
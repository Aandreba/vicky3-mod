use std::{path::Path, collections::HashMap};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use super::{NamedCountryType, CountryType, CountryTier};
use crate::{Result, Str, utils::{GetStr, ReadDirStream, FlattenOkIter, stream_and_then}, data::{Color, culture::{NamedCulture, Culture}, try_collect, read_to_string}};

pub type DefinitionRef<'a> = &'a Definition<'a>;
pub type NamedDefinition<'a> = (&'a str, DefinitionRef<'a>);

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Definition<'a> {
    pub color: Color,
    pub country_type: NamedCountryType<'a>,
    pub tier: CountryTier,
    pub cultures: Box<[NamedCulture<'a>]>,
    // pub capital: Str // todo states
    pub is_named_from_capital: bool
}

impl<'a> Definition<'a> {
    pub fn from_raw (
        raw: RawDefinition,
        tys: &'a HashMap<Str, CountryType<'a>>,
        cultures: &'a HashMap<Str, Culture<'a>>
    ) -> Result<Self> {
        let cultures = raw.cultures.iter()
            .map(|x| cultures.try_get_str_value(x));

        return Ok(Self {
            cultures: try_collect(cultures)?,
            country_type: tys.try_get_str_value(&raw.country_type)?,
            tier: raw.tier,
            color: raw.color,
            is_named_from_capital: raw.is_named_from_capital
        })
    }

    #[inline]
    pub async fn from_common (
        common: &Path,
        tys: &'a HashMap<Str, CountryType<'a>>,
        cultures: &'a HashMap<Str, Culture<'a>>
    ) -> Result<impl Stream<Item = Result<(Str, Definition<'a>)>>> {
        let iter = RawDefinition::from_common(common).await?;
        return Ok(stream_and_then(
            iter,
            |(name, raw)| Ok((name, Self::from_raw(raw, tys, cultures)?)))
        );
    }
}


#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct RawDefinition {
    pub color: Color,
    pub country_type: Str,
    pub tier: CountryTier,
    pub cultures: Box<[Str]>,
    pub capital: Option<Str>,
    #[jomini(default)]
    pub is_named_from_capital: bool
}

impl RawDefinition {
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